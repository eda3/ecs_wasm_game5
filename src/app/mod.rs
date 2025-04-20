// src/app/mod.rs
//! アプリケーション全体に関わるモジュール（初期化、状態管理など）をまとめるよ！

pub mod game_app;     // GameApp 構造体とその実装
pub mod init_handler; // 初期化処理
pub mod network_handler; // ネットワーク関連のハンドラー
pub mod event_handler; // イベント処理
pub mod renderer;    // 描画処理
pub mod state_handler;   // ★追加
// pub mod init_handler;    // 今後追加予定 
pub mod drag_handler;
pub mod layout_calculator;
pub mod state_getter;
pub mod browser_event_manager; // ★追加 