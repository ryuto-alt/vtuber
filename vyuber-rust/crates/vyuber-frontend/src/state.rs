use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChatUser {
    Me,             // 自分（音声入力）
    Ai(String),     // AI視聴者（名前付き）
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub id: usize,
    pub user: ChatUser,
    pub text: String,
}

// アプリ全体で共有するステート（状態）
#[derive(Clone, Copy)]
pub struct GlobalState {
    pub messages: ReadSignal<Vec<ChatMessage>>,
    pub set_messages: WriteSignal<Vec<ChatMessage>>,
    pub unique_ai_users: ReadSignal<HashSet<String>>,
    pub set_unique_ai_users: WriteSignal<HashSet<String>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let (messages, set_messages) = signal(Vec::new());
        let (unique_ai_users, set_unique_ai_users) = signal(HashSet::new());
        Self {
            messages,
            set_messages,
            unique_ai_users,
            set_unique_ai_users,
        }
    }

    // メッセージを追加する関数
    pub fn add_message(&self, user: ChatUser, text: String) {
        self.set_messages.update(|msgs| {
            let id = msgs.len();
            msgs.push(ChatMessage { id, user: user.clone(), text });
        });

        // AI視聴者の場合、ユニークカウントに追加
        if let ChatUser::Ai(name) = user {
            self.set_unique_ai_users.update(|users| {
                users.insert(name);
            });
        }
    }

    // デモ用：AI視聴者のコメントを追加
    #[allow(dead_code)]
    pub fn add_demo_ai_comment(&self) {
        let viewers = vec!["初見さん", "古参ファン", "通りすがり"];
        let comments = vec!["こんにちは！", "wktk", "草", "888888", "初見です"];

        // ランダムっぽく選択（簡易実装）
        let v_idx = (js_sys::Math::random() * viewers.len() as f64) as usize;
        let c_idx = (js_sys::Math::random() * comments.len() as f64) as usize;

        self.add_message(
            ChatUser::Ai(viewers[v_idx].to_string()),
            comments[c_idx].to_string()
        );
    }
}