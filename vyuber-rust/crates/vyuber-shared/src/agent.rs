use serde::{Deserialize, Serialize};

/// エージェントの知能レベル（階層）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentTier {
    /// 70B: 文脈を理解し、会話を主導する「固定ファン」
    Anchor,
    /// 8B: 反射的に短文を投下する「ガヤ」
    Swarm,
}

impl AgentTier {
    /// バッジのラベル
    pub fn label(&self) -> &'static str {
        match self {
            AgentTier::Anchor => "70B",
            AgentTier::Swarm => "8B",
        }
    }

    /// UI表示用のバッジカラー (Tailwind CSS)
    pub fn badge_style(&self) -> &'static str {
        match self {
            AgentTier::Anchor => "bg-blue-500/10 text-blue-400 border-blue-500/20",
            AgentTier::Swarm => "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
        }
    }
}

/// 共通のエージェント定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub tier: AgentTier, // ★厳密な区別
    
    // Anchor用詳細プロフィール（Swarmの場合は空文字や無視でOK）
    pub bio: String,     // 年齢、職業、関係性などをまとめた文字列
    
    // 口調・トーン（全エージェント必須）
    pub tone: String,
    
    // テーマカラー（Tailwind class）
    pub color_class: String,
}