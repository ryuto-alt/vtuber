use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

use super::text_corrector::TextCorrector;

/// no_speech_probability がこの値を超えたセグメントは幻覚とみなして無視
const NO_SPEECH_THRESHOLD: f32 = 0.5;

/// 推論に使用するスレッド数（GPU使用時は無視される）
const N_THREADS: i32 = 4;

/// Beam探索のビーム幅（GPU使用時は高精度モード）
/// CPU: 5-6, GPU: 8-10 推奨
const BEAM_SIZE_CPU: i32 = 6;
const BEAM_SIZE_GPU: i32 = 8;

/// best_of: 複数候補から最良を選択（精度向上）
const BEST_OF: i32 = 5;

/// Whisper が沈黙時に出力する既知のハルシネーション（部分一致）
const HALLUCINATION_PATTERNS: &[&str] = &[
    "ご視聴ありがとうございました",
    "ご視聴いただきありがとうございます",
    "チャンネル登録",
    "お願いします",
    "おやすみなさい",
    "お疲れ様でした",
    "ありがとうございました",
    "ありがとうございます",
    "動画見てくれて",
    "動画を見てくれて",
    "高評価",
    "グッドボタン",
    "コメント欄",
    "字幕",
    "Subtitles",
    "Subscribe",
    "Thank you",
    "Thanks for watching",
    "MorinagaYuki",
    "Amara.org",
    "www.",
    "http",
    "字幕作成",
    "翻訳",
];

/// 日本語モードで英語テキストが出た場合のハルシネーション判定に使う
/// ASCII英字の割合がこの値を超えたら英語ハルシネーションとみなす
const ASCII_ALPHA_RATIO_THRESHOLD: f32 = 0.5;

#[derive(Clone)]
pub struct WhisperService {
    ctx: Arc<WhisperContext>,
    /// State を再利用して初期化コストを削減
    state: Arc<Mutex<Option<WhisperState>>>,
    /// 形態素解析ベースのテキスト補完
    corrector: Arc<TextCorrector>,
    /// GPU利用可能フラグ（CUDA検出）
    use_gpu: bool,
}

// WhisperState は Send ではないが、Mutex で排他アクセスするため安全
unsafe impl Send for WhisperService {}
unsafe impl Sync for WhisperService {}

impl WhisperService {
    pub fn new(model_path: &str) -> Result<Self, String> {
        let path = PathBuf::from(model_path);
        if !path.exists() {
            return Err(format!(
                "Whisper model not found: {}\nRun download-model.bat to download it.",
                model_path
            ));
        }

        // GPU (CUDA) の利用可能性を検出
        let mut params = WhisperContextParameters::default();
        let use_gpu = if cfg!(feature = "cuda") {
            // CUDA有効化を試行
            params.use_gpu(true);
            tracing::info!("CUDA feature enabled, attempting to use GPU");
            true
        } else {
            tracing::info!("CUDA feature disabled, using CPU");
            false
        };

        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        if use_gpu {
            tracing::info!("✓ Whisper model loaded on GPU: {}", model_path);
        } else {
            tracing::info!("✓ Whisper model loaded on CPU: {}", model_path);
        }

        let corrector = TextCorrector::new()
            .map_err(|e| format!("Failed to init text corrector: {}", e))?;
        tracing::info!("✓ Text corrector initialized (lindera + IPADIC)");

        Ok(Self {
            ctx: Arc::new(ctx),
            state: Arc::new(Mutex::new(None)),
            corrector: Arc::new(corrector),
            use_gpu,
        })
    }

    /// State を取得（既存があれば再利用、なければ新規作成）
    fn get_or_create_state(&self) -> Result<WhisperState, String> {
        let mut guard = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(state) = guard.take() {
            Ok(state)
        } else {
            self.ctx
                .create_state()
                .map_err(|e| format!("Failed to create Whisper state: {}", e))
        }
    }

    /// State をプールに返却
    fn return_state(&self, state: WhisperState) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = Some(state);
        }
    }

    /// ハルシネーション判定
    fn is_hallucination(text: &str) -> bool {
        let trimmed = text.trim();

        // 短すぎるテキスト（1〜2文字）は信頼性が低い
        if trimmed.chars().count() <= 2 {
            return true;
        }

        // 同じ文字の繰り返し
        let chars: Vec<char> = trimmed.chars().collect();
        if chars.len() >= 3 && chars.windows(2).all(|w| w[0] == w[1]) {
            return true;
        }

        // 日本語モードなのにASCII英字が過半数 → 英語ハルシネーション
        // 例: "whilethatpersonistryingtostopthegame"
        let total_chars = chars.len();
        if total_chars >= 5 {
            let ascii_alpha_count = chars.iter().filter(|c| c.is_ascii_alphabetic()).count();
            let ratio = ascii_alpha_count as f32 / total_chars as f32;
            if ratio > ASCII_ALPHA_RATIO_THRESHOLD {
                return true;
            }
        }

        // 既知のハルシネーションパターンに一致
        for pattern in HALLUCINATION_PATTERNS {
            if trimmed.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// PCM f32 (16kHz mono) を受け取って文字起こしする
    pub fn transcribe(&self, pcm_data: &[f32]) -> Result<String, String> {
        let mut state = self.get_or_create_state()?;

        // BeamSearch: GPU使用時はbeam_sizeを増やして精度向上（10-30倍高速なため許容）
        let beam_size = if self.use_gpu { BEAM_SIZE_GPU } else { BEAM_SIZE_CPU };
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size,
            patience: 1.3  // より慎重に探索
        });

        params.set_language(Some("ja"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // 長い発話に対応: 複数セグメントを許可
        params.set_single_segment(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);

        // セグメント間のエラー伝播を防ぐ（ストリーミングでは各発話が独立）
        params.set_no_context(true);
        // 翻訳モードを明示的に無効化
        params.set_translate(false);

        // スレッド数（GPU使用時は無視される）
        if !self.use_gpu {
            params.set_n_threads(N_THREADS);
        }

        // 温度設定: 確定的デコード → 不確実時に温度を上げてリトライ
        params.set_temperature(0.0);
        params.set_temperature_inc(0.15);

        // エントロピー閾値: 日本語は文字種が多くentropy 3.0-3.5が普通
        // 低すぎると全セグメントが温度フォールバックに入り精度悪化
        params.set_entropy_thold(3.0);

        // 低信頼度セグメントの再デコード閾値
        params.set_logprob_thold(-0.8);

        // best_of はGreedy戦略でのみ有効（BeamSearch使用時は不要）

        // 初期プロンプト: 純粋な日本語のみ（英語を混ぜるとモデルが英語に引っ張られる）
        params.set_initial_prompt(
            "こんにちは、今日も配信を始めていきたいと思います。\
             雑談しながらやっていきましょう。\
             皆さんのコメントも読んでいきますね。"
        );

        let result = state.full(params, pcm_data);
        if let Err(e) = result {
            self.return_state(state);
            return Err(format!("Whisper inference failed: {}", e));
        }

        let mut text = String::new();
        let num_segments = state.full_n_segments();

        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if segment.no_speech_probability() > NO_SPEECH_THRESHOLD {
                    tracing::debug!(
                        "Segment {} skipped (no_speech_prob: {:.2})",
                        i,
                        segment.no_speech_probability()
                    );
                    continue;
                }
                if let Ok(s) = segment.to_str_lossy() {
                    text.push_str(&s);
                }
            }
        }

        let result = text.trim().to_string();

        // ハルシネーションフィルタ
        if Self::is_hallucination(&result) {
            tracing::debug!("Hallucination filtered: {:?}", result);
            self.return_state(state);
            return Ok(String::new());
        }

        self.return_state(state);

        // テキスト補完: 未知語を形態素解析ベースで補正
        let corrected = self.corrector.correct(&result)?;
        if corrected != result {
            tracing::info!("Text corrected: '{}' -> '{}'", result, corrected);
        }

        Ok(corrected)
    }
}
