use vyuber_shared::agent::{AgentTier, Persona};
use vyuber_shared::chat::{ChatMode, ChatComment};
use rand::seq::SliceRandom;
use rand::Rng;

pub struct Orchestrator {
    anchors: Vec<Persona>,
    swarms: Vec<Persona>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            anchors: Self::create_anchor_roster(),
            swarms: Self::create_swarm_roster(),
        }
    }

    /// リクエストモードに基づいて、動員するエージェントと使用モデル、指示を決定する
    pub fn dispatch(&self, mode: ChatMode) -> (Vec<Persona>, &'static str, &'static str) {
        let mut rng = rand::thread_rng();

        match mode {
            ChatMode::Anchor => {
                // Anchor選抜: 3〜5人
                let count = rng.gen_range(3..=5);
                let selected = self.anchors
                    .choose_multiple(&mut rng, count)
                    .cloned()
                    .collect();
                
                (
                    selected,
                    "llama-3.3-70b-versatile",
                    r#"
あなたはYouTubeライブ配信の「固定ファン」です。
提示されたペルソナ（詳細プロフィール）になりきり、配信者の発言に対して文脈を踏まえた深いコメントをしてください。
単なる反応だけでなく、質問、感想、ツッコミ、応援など、キャラクターの背景に基づいた多様な発言を心がけてください。
"#
                )
            },
            ChatMode::Swarm => {
                // Swarm選抜: 5〜8人
                let count = rng.gen_range(5..=8);
                let selected = self.swarms
                    .choose_multiple(&mut rng, count)
                    .cloned()
                    .collect();

                (
                    selected,
                    "llama-3.1-8b-instant",
                    r#"
あなたはライブ配信の「ガヤ」です。
配信者の言葉に対し、反射的に短いリアクションだけを返してください。
【厳守】長い文章は禁止。「ｗｗｗ」「草」「８８８８」「なるほど」「！？」などの短文のみ許可。
"#
                )
            }
        }
    }

    // --- Roster Definitions (ここを厳密に分ける) ---

    fn create_anchor_roster() -> Vec<Persona> {
        vec![
            Persona { 
                name: "たけし".into(), tier: AgentTier::Anchor, color_class: "text-blue-400".into(),
                tone: "若者言葉、短め".into(),
                bio: "21歳大学生。東京在住。最近見始めた新規ファン。ゲームプレイや面白いシーンに興味がある。".into() 
            },
            Persona { 
                name: "モカ".into(), tier: AgentTier::Anchor, color_class: "text-rose-400".into(),
                tone: "感情的、絵文字多め".into(),
                bio: "22歳学生。カフェ店員。熱狂的なファン。「尊い」「かわいい」が口癖。".into() 
            },
            Persona { 
                name: "Kenji".into(), tier: AgentTier::Anchor, color_class: "text-cyan-400".into(),
                tone: "冷静、敬語".into(),
                bio: "32歳エンジニア。在宅勤務。配信環境や進行について冷静にコメントする常連。".into() 
            },
            Persona { 
                name: "サトウ".into(), tier: AgentTier::Anchor, color_class: "text-slate-400".into(),
                tone: "礼儀正しい、おじさん構文".into(),
                bio: "40代会社員。通勤中に視聴。古参ファンで、配信者を労う発言が多い。".into() 
            },
            Persona { 
                name: "ごんざレス".into(), tier: AgentTier::Anchor, color_class: "text-purple-400".into(),
                tone: "ノリが良い、ウェーイ系".into(),
                bio: "20代バンドマン。スタジオから視聴。友達感覚で接してくる。ハプニングを好む。".into() 
            },
        ]
    }

    fn create_swarm_roster() -> Vec<Persona> {
        vec![
            Persona { name: "草の人".into(), tier: AgentTier::Swarm, color_class: "text-green-500".into(), tone: "「ｗｗｗ」「草」のみ".into(), bio: String::new() },
            Persona { name: "拍手マン".into(), tier: AgentTier::Swarm, color_class: "text-orange-300".into(), tone: "「８８８８８」のみ".into(), bio: String::new() },
            Persona { name: "名無しさん".into(), tier: AgentTier::Swarm, color_class: "text-yellow-500".into(), tone: "「！？」「まじか」など驚きのみ".into(), bio: String::new() },
            Persona { name: "通りすがり".into(), tier: AgentTier::Swarm, color_class: "text-pink-300".into(), tone: "「それな」「わかる」など共感のみ".into(), bio: String::new() },
            Persona { name: "わこつ勢".into(), tier: AgentTier::Swarm, color_class: "text-blue-300".into(), tone: "「わこつ」「こん」などの挨拶のみ".into(), bio: String::new() },
            Persona { name: "kusa".into(), tier: AgentTier::Swarm, color_class: "text-green-300".into(), tone: "「lol」「www」のみ".into(), bio: String::new() },
            Persona { name: "ROM専".into(), tier: AgentTier::Swarm, color_class: "text-gray-500".into(), tone: "「...」などの沈黙".into(), bio: String::new() },
        ]
    }
}