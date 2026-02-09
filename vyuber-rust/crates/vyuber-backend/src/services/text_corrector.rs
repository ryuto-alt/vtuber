use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;

/// IPADIC 詳細情報のインデックス
const IPADIC_READING: usize = 7; // 読み (カタカナ)

/// 補完対象とする未知語の最小文字数（短い語は誤補完しやすいためスキップ）
const MIN_CORRECTION_LENGTH: usize = 3;

/// 音韻候補のマッチに許容する最大編集距離
const MAX_EDIT_DISTANCE: usize = 1;

/// VTuber / 配信ドメイン固有の用語辞書
/// (カタカナ読み, 正しい表記)
const DOMAIN_CORRECTIONS: &[(&str, &str)] = &[
    ("スパチャ", "スパチャ"),
    ("スーパーチャット", "スーパーチャット"),
    ("メンシ", "メンシ"),
    ("メンバシップ", "メンバーシップ"),
    ("ブイチューバー", "VTuber"),
    ("ブイチューバ", "VTuber"),
    ("コラボ", "コラボ"),
    ("マシュマロ", "マシュマロ"),
    ("ウタワク", "歌枠"),
    ("ザツダン", "雑談"),
    ("ドウジシチョウ", "同時視聴"),
    ("アーカイブ", "アーカイブ"),
    ("クリップ", "クリップ"),
    ("エモート", "エモート"),
    ("サブスク", "サブスク"),
    ("チャンネルトウロク", "チャンネル登録"),
    ("ゲームジッキョウ", "ゲーム実況"),
    ("スリーディー", "3D"),
];

pub struct TextCorrector {
    tokenizer: Tokenizer,
}

impl TextCorrector {
    pub fn new() -> Result<Self, String> {
        let dictionary = load_dictionary("embedded://ipadic")
            .map_err(|e| format!("Failed to load IPADIC dictionary: {}", e))?;
        let segmenter = Segmenter::new(Mode::Normal, dictionary, None);
        let tokenizer = Tokenizer::new(segmenter);

        Ok(Self { tokenizer })
    }

    /// Whisper出力テキストを後処理して補完する
    pub fn correct(&self, text: &str) -> Result<String, String> {
        if text.is_empty() {
            return Ok(String::new());
        }

        let mut tokens = self
            .tokenizer
            .tokenize(text)
            .map_err(|e| format!("Tokenization failed: {}", e))?;

        let mut result = String::with_capacity(text.len());

        for token in &mut tokens {
            let surface: String = token.surface.to_string();

            // 未知語判定: 読みフィールドが "*" または欠落
            if Self::is_unknown_token(token) && surface.chars().count() >= MIN_CORRECTION_LENGTH {
                if let Some(corrected) = self.try_correct(&surface) {
                    tracing::debug!("Text correction: '{}' -> '{}'", surface, corrected);
                    result.push_str(&corrected);
                    continue;
                }
            }

            result.push_str(&surface);
        }

        Ok(result)
    }

    /// トークンが辞書に存在しない未知語かどうかを判定
    fn is_unknown_token(token: &mut lindera::token::Token) -> bool {
        let details = token.details();
        if details.len() <= IPADIC_READING {
            return true;
        }
        details[IPADIC_READING] == "*"
    }

    /// 未知語トークンの補完を試行
    fn try_correct(&self, surface: &str) -> Option<String> {
        let kana = Self::to_katakana(surface);

        // 1. ドメイン辞書から完全一致を検索
        for &(reading, correct_form) in DOMAIN_CORRECTIONS {
            if kana == reading {
                return Some(correct_form.to_string());
            }
        }

        // 2. ドメイン辞書から近似一致を検索 (編集距離 <= MAX_EDIT_DISTANCE)
        for &(reading, correct_form) in DOMAIN_CORRECTIONS {
            if reading.chars().count() >= MIN_CORRECTION_LENGTH {
                let dist = strsim::levenshtein(&kana, reading);
                if dist <= MAX_EDIT_DISTANCE {
                    return Some(correct_form.to_string());
                }
            }
        }

        // 3. 音韻候補を生成し、lindera辞書で検証
        let candidates = Self::generate_phonetic_candidates(&kana);
        for candidate in &candidates {
            if let Some(known) = self.validate_as_known_word(candidate) {
                return Some(known);
            }
        }

        None
    }

    /// 候補文字列をlinderaでトークン化し、単一の既知語として認識されれば採用
    fn validate_as_known_word(&self, candidate: &str) -> Option<String> {
        let mut tokens = self.tokenizer.tokenize(candidate).ok()?;
        // 単一トークンかつ既知語の場合のみ採用
        if tokens.len() == 1 && !Self::is_unknown_token(&mut tokens[0]) {
            Some(tokens[0].surface.to_string())
        } else {
            None
        }
    }

    /// 音韻的に類似した候補文字列を生成
    fn generate_phonetic_candidates(kana: &str) -> Vec<String> {
        let chars: Vec<char> = kana.chars().collect();
        let mut candidates = Vec::new();

        // 長音(ー)の挿入: 各文字の後に ー を挿入
        for i in 0..chars.len() {
            let mut c = chars.clone();
            c.insert(i + 1, 'ー');
            candidates.push(c.into_iter().collect());
        }

        // 長音(ー)の削除
        for i in 0..chars.len() {
            if chars[i] == 'ー' {
                let mut c = chars.clone();
                c.remove(i);
                candidates.push(c.into_iter().collect());
            }
        }

        // 促音(ッ)の挿入
        for i in 0..chars.len() {
            let mut c = chars.clone();
            c.insert(i, 'ッ');
            candidates.push(c.into_iter().collect());
        }

        // 促音(ッ/っ)の削除
        for i in 0..chars.len() {
            if chars[i] == 'ッ' || chars[i] == 'っ' {
                let mut c = chars.clone();
                c.remove(i);
                candidates.push(c.into_iter().collect());
            }
        }

        // 類似モーラの置換
        let substitutions: &[(char, char)] = &[
            ('ヂ', 'ジ'),
            ('ヅ', 'ズ'),
            ('ヲ', 'オ'),
            ('ワ', 'ハ'),
            ('エ', 'ヘ'),
        ];
        for i in 0..chars.len() {
            for &(from, to) in substitutions {
                if chars[i] == from {
                    let mut c = chars.clone();
                    c[i] = to;
                    candidates.push(c.into_iter().collect());
                }
                if chars[i] == to {
                    let mut c = chars.clone();
                    c[i] = from;
                    candidates.push(c.into_iter().collect());
                }
            }
        }

        candidates
    }

    /// ひらがな → カタカナ変換（漢字・カタカナはそのまま）
    fn to_katakana(text: &str) -> String {
        text.chars()
            .map(|c| {
                if ('\u{3041}'..='\u{3096}').contains(&c) {
                    char::from_u32(c as u32 + 0x60).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let corrector = TextCorrector::new().unwrap();
        assert_eq!(corrector.correct("").unwrap(), "");
    }

    #[test]
    fn test_known_words_unchanged() {
        let corrector = TextCorrector::new().unwrap();
        let result = corrector.correct("今日はいい天気ですね").unwrap();
        assert_eq!(result, "今日はいい天気ですね");
    }

    #[test]
    fn test_to_katakana() {
        assert_eq!(TextCorrector::to_katakana("ひらがな"), "ヒラガナ");
        assert_eq!(TextCorrector::to_katakana("カタカナ"), "カタカナ");
        assert_eq!(TextCorrector::to_katakana("漢字"), "漢字");
    }

    #[test]
    fn test_correction_performance() {
        let corrector = TextCorrector::new().unwrap();
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = corrector.correct("今日の配信はゲーム実況をやります");
        }
        let elapsed = start.elapsed();
        // 100回で5秒以内（1回50ms以内）
        assert!(elapsed.as_millis() < 5000, "Too slow: {:?}", elapsed);
    }
}
