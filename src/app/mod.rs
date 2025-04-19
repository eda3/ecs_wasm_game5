// src/app/mod.rs
//! GameApp の内部ロジックを役割ごとに分割して置くモジュールだよ！

pub mod event_handler;
pub mod state_handler;   // ★追加
pub mod network_handler; // ★追加
pub mod init_handler;    // ★追加
pub mod renderer;      // ★追加
// pub mod init_handler;    // 今後追加予定 