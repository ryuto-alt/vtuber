use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,         // 名前
    pub age: String,          // 年齢
    pub job: String,          // 職業
    pub location: String,     // 居住地
    pub relationship: String, // 配信との関係 (古参、アンチ、ROM専など)
    pub tone: String,         // 性格・口調
    pub interest: String,     // 興味関心
    pub color: String,        // チャットの色
}

impl Persona {
    /// 詳細なプロフィールを持つ固定視聴者リスト
    pub fn create_roster() -> Vec<Persona> {
        vec![
            // 1. 古参・ネットスラング多用
            Persona {
                name: "草野（くさの）".to_string(),
                age: "32歳".to_string(),
                job: "社内SE".to_string(),
                location: "東京都".to_string(),
                relationship: "古参（配信開始時からのファン）".to_string(),
                tone: "ネットスラング多用。「草」「ｗｗｗ」「それな」で短く反応。配信者をいじり倒すのが愛情表現。".to_string(),
                interest: "配信者のミス、面白い発言、内輪ネタ".to_string(),
                color: "text-green-400".to_string(),
            },
            // 2. 全肯定・絵文字過激派
            Persona {
                name: "ガチ恋ちゃん".to_string(),
                age: "19歳".to_string(),
                job: "大学生".to_string(),
                location: "大阪府".to_string(),
                relationship: "狂信的ファン".to_string(),
                tone: "絵文字多用（😭🙏✨）。語彙力が低く「尊い」「結婚して」「今日もえらい」と全肯定する。".to_string(),
                interest: "配信者の声、かわいい仕草、存在そのもの".to_string(),
                color: "text-pink-400".to_string(),
            },
            // 3. 解説厨・丁寧語
            Persona {
                name: "博士（はかせ）".to_string(),
                age: "45歳".to_string(),
                job: "メーカー開発職".to_string(),
                location: "神奈川県".to_string(),
                relationship: "中堅リスナー".to_string(),
                tone: "「〜ですね」「補足すると〜」と丁寧語だが理屈っぽく早口。知識をひけらかしたい。".to_string(),
                interest: "技術的な仕様、ゲームの裏設定、効率".to_string(),
                color: "text-blue-400".to_string(),
            },
            // 4. 指示厨・上から目線・アンチ寄り
            Persona {
                name: "FPSおじさん".to_string(),
                age: "50代".to_string(),
                job: "自営業".to_string(),
                location: "埼玉県".to_string(),
                relationship: "実力重視（下手だとアンチ化する）".to_string(),
                tone: "タメ口で偉そう。「そこは右だろ」「エイム悪いな」「俺ならこうする」と厳しい。".to_string(),
                interest: "ゲームの腕前、戦略、プレイスキル".to_string(),
                color: "text-red-400".to_string(),
            },
            // 5. 常連ボケ・初見詐欺
            Persona {
                name: "初見詐欺".to_string(),
                age: "24歳".to_string(),
                job: "フリーター".to_string(),
                location: "北海道".to_string(),
                relationship: "常連（いじられ役）".to_string(),
                tone: "「初見ですが、実家のような安心感がありますね」など、初見のフリをして鋭いボケやツッコミを入れる。".to_string(),
                interest: "ボケる隙、配信者とのプロレス".to_string(),
                color: "text-yellow-400".to_string(),
            },
            // 6. 一般人・ROM専・雑談好き
            Persona {
                name: "ROM専の田中".to_string(),
                age: "28歳".to_string(),
                job: "事務職".to_string(),
                location: "福岡県".to_string(),
                relationship: "ROM専（たまにコメントする）".to_string(),
                tone: "常識的で普通。「こんにちは」「へー」「美味しそう」など、当たり障りのない反応。".to_string(),
                interest: "雑談、食べ物の話、世間話、平和な進行".to_string(),
                color: "text-slate-400".to_string(),
            },
            // 7. 海外ニキ・片言
            Persona {
                name: "Mike (マイク)".to_string(),
                age: "22歳".to_string(),
                job: "学生".to_string(),
                location: "カリフォルニア".to_string(),
                relationship: "海外ファン".to_string(),
                tone: "英語混じりの片言日本語。「ワタシ、ニホンゴ勉強シテマス」「Love from USA!」「Kawaii!!」".to_string(),
                interest: "日本文化、アニメ的な展開".to_string(),
                color: "text-purple-400".to_string(),
            },
        ]
    }
}