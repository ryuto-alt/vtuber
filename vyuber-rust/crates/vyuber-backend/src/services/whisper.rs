use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

/// no_speech_probability がこの値を超えたセグメントは幻覚とみなして無視
const NO_SPEECH_THRESHOLD: f32 = 0.5;

/// 推論に使用するスレッド数
const N_THREADS: i32 = 4;

/// Whisper が沈黙時に出力する既知のハルシネーション（部分一致）
const HALLUCINATION_PATTERNS: &[&str] = &[
    "ご視聴ありがとうございました",
    "ご視聴いただきありがとうございます",
    "チャンネル登録",
    "お願いします",
    "おやすみなさい",
    "お疲れ様でした",
    "ありがとうございました",
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

#[derive(Clone)]
pub struct WhisperService {
    ctx: Arc<WhisperContext>,
    /// State を再利用して初期化コストを削減
    state: Arc<Mutex<Option<WhisperState>>>,
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

        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        tracing::info!("Whisper model loaded: {}", model_path);

        Ok(Self {
            ctx: Arc::new(ctx),
            state: Arc::new(Mutex::new(None)),
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

        // BeamSearch: Greedy より高精度（beam_size=3 で速度と精度のバランス）
        let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 3, patience: 1.0 });
        params.set_language(Some("ja"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        // 長い発話に対応: 複数セグメントを許可
        params.set_single_segment(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_n_threads(N_THREADS);

        // 日本語認識の精度向上: 初期プロンプトで言語ヒントを与える
        params.set_initial_prompt("こんにちは、配信を始めます。");

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
        Ok(result)
    }
}
