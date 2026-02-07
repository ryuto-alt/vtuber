use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub age: String,
    pub job: String,
    pub location: String,
    pub relationship: String,
    pub tone: String,
    pub interest: String,
    pub color: String,
}

impl Persona {
    /// メインで喋る視聴者リスト（普通の反応をする人たち）
    pub fn create_main_roster() -> Vec<Persona> {
        vec![
            Persona { name: "たけし".into(), age: "21歳".into(), job: "大学生".into(), location: "東京".into(), relationship: "最近見始めた".into(), tone: "「おー」「なるほど」など、若者らしい短めの反応。タメ口混じり。".into(), interest: "ゲームプレイ、面白いシーン".into(), color: "text-blue-400".into() },
            Persona { name: "あや".into(), age: "24歳".into(), job: "事務職".into(), location: "神奈川".into(), relationship: "ファン".into(), tone: "「こんにちは！」「楽しみです✨」と明るく丁寧。絵文字を少し使う。".into(), interest: "雰囲気、雑談".into(), color: "text-pink-400".into() },
            Persona { name: "Kenji".into(), age: "32歳".into(), job: "エンジニア".into(), location: "自宅".into(), relationship: "常連".into(), tone: "「音質問題ないです」「画質きれい」など、状況を冷静に報告する。".into(), interest: "配信環境、進行".into(), color: "text-cyan-400".into() },
            Persona { name: "ユウ".into(), age: "17歳".into(), job: "高校生".into(), location: "自室".into(), relationship: "憧れ".into(), tone: "「うまっ」「すげえ」と素直に反応する。".into(), interest: "プレイスキル".into(), color: "text-green-400".into() },
            Persona { name: "みかん".into(), age: "28歳".into(), job: "パート".into(), location: "リビング".into(), relationship: "ながら見".into(), tone: "「家事しながら見てます〜」「平和ですね」と落ち着いた口調。".into(), interest: "世間話、まったり進行".into(), color: "text-orange-400".into() },
            Persona { name: "サトウ".into(), age: "40代".into(), job: "会社員".into(), location: "通勤中".into(), relationship: "古参".into(), tone: "「お疲れ様です」「今日もよろしく」と礼儀正しい。".into(), interest: "挨拶、労い".into(), color: "text-slate-400".into() },
            Persona { name: "Ryo".into(), age: "20代".into(), job: "フリーター".into(), location: "PC前".into(), relationship: "ゲーマー".into(), tone: "「そこ右行ったほうがいいかも」「ナイス」とゲーム内容に即したコメント。".into(), interest: "攻略、効率".into(), color: "text-red-400".into() },
            Persona { name: "ななし".into(), age: "不明".into(), job: "不明".into(), location: "ネット".into(), relationship: "初見".into(), tone: "「初見です」「なんのゲーム？」と素朴な疑問を投げる。".into(), interest: "概要、状況把握".into(), color: "text-gray-400".into() },
            Persona { name: "モカ".into(), age: "22歳".into(), job: "学生".into(), location: "カフェ".into(), relationship: "ファン".into(), tone: "「かわいい！」「尊い...」と感情豊かに反応する。".into(), interest: "ビジュアル、声".into(), color: "text-rose-400".into() },
            Persona { name: "T_K".into(), age: "30代".into(), job: "営業".into(), location: "出先".into(), relationship: "ROM専".into(), tone: "「移動中に見てる」「アーカイブ助かる」など自分語り少なめ。".into(), interest: "視聴環境".into(), color: "text-indigo-400".into() },
            Persona { name: "ハル".into(), age: "10代".into(), job: "中学生".into(), location: "自宅".into(), relationship: "新規".into(), tone: "「わこつ」「これマジ？」などネットスラングを軽く使う。".into(), interest: "盛り上がり".into(), color: "text-yellow-400".into() },
            Persona { name: "ごんざレス".into(), age: "20代".into(), job: "バンドマン".into(), location: "スタジオ".into(), relationship: "友達感覚".into(), tone: "「ウェイｗｗ」「うける」とノリが良い。".into(), interest: "笑い、ハプニング".into(), color: "text-purple-400".into() },
            Persona { name: "書記".into(), age: "20代".into(), job: "学生".into(), location: "自宅".into(), relationship: "支援".into(), tone: "配信者が言ったことを要約したり「了解です」と肯定する。".into(), interest: "伝達".into(), color: "text-teal-400".into() },
            Persona { name: "Mike".into(), age: "20代".into(), job: "海外".into(), location: "USA".into(), relationship: "海外勢".into(), tone: "「Hi」「Cool!」など英語や簡単な日本語。".into(), interest: "国際交流".into(), color: "text-blue-500".into() },
            Persona { name: "猫好き".into(), age: "30代".into(), job: "在宅".into(), location: "自宅".into(), relationship: "常連".into(), tone: "「ｗｗｗ」「それな」など、適度な相槌を打つ。".into(), interest: "共感".into(), color: "text-amber-400".into() },
        ]
    }

    /// ガヤ専用リスト（短文リアクション要員）
    pub fn create_gaya_roster() -> Vec<Persona> {
        vec![
            Persona { name: "草の人".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「ｗｗｗ」「草」「大草原」のみ。".into(), interest: "笑い".into(), color: "text-green-500".into() },
            Persona { name: "拍手係".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「８８８８８」「パチパチ」のみ。".into(), interest: "賞賛".into(), color: "text-orange-300".into() },
            Persona { name: "驚き係".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「！？」「おおお」「まじか」のみ。".into(), interest: "驚き".into(), color: "text-yellow-500".into() },
            Persona { name: "肯定係".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「それな」「わかる」「たしかに」のみ。".into(), interest: "共感".into(), color: "text-pink-300".into() },
            Persona { name: "挨拶係".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「わこつ」「こん」「きた」のみ。".into(), interest: "開始、登場".into(), color: "text-blue-300".into() },
            Persona { name: "kusa".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「lol」「www」のみ。".into(), interest: "笑い".into(), color: "text-green-300".into() },
            Persona { name: "ROM".into(), age: "-".into(), job: "-".into(), location: "-".into(), relationship: "-".into(), tone: "「...」「(笑)」など控えめな反応。".into(), interest: "観察".into(), color: "text-gray-500".into() },
        ]
    }
}