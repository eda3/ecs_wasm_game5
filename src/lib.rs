// src/lib.rs

// WASM と JavaScript を繋ぐための基本！
use wasm_bindgen::prelude::*;
// ★復活！ JsCast トレイトを使う！★
// use wasm_bindgen::JsCast; // game_app.rs に移動

// ★修正: 未使用の型をごっそり削除！ Event, window, HtmlCanvasElement, CanvasRenderingContext2d は残す★
// use web_sys::{window, Event, HtmlCanvasElement, CanvasRenderingContext2d}; // game_app.rs に移動

// 標準ライブラリから、スレッドセーフな共有ポインタとミューテックスを使うよ。
// 非同期のコールバック関数からでも安全にデータを共有・変更するために必要！
// use std::sync::{Arc, Mutex}; // game_app.rs に移動
// メッセージキュー（受信メッセージを一時的に溜めておく場所）のために VecDeque を使うよ。
// use std::collections::VecDeque; // game_app.rs に移動

// 自分で作ったモジュールたち！ これでコードを整理してるんだ。
pub mod entity;
pub mod component;
pub mod world;
pub mod system;
pub mod components;
pub mod systems;
pub mod network;
pub mod protocol;
pub mod logic;
pub mod app;
pub mod config;

// ★追加: GameApp を lib.rs のスコープに公開！
pub use app::game_app::GameApp;

// 各モジュールから必要な型をインポート！
// use crate::network::NetworkManager; // game_app.rs に移動
// use crate::protocol::{ClientMessage, ServerMessage, GameStateData, CardData, PositionData, PlayerId}; // game_app.rs に移動
// use crate::components::stack::StackType; // game_app.rs に移動
// use crate::entity::Entity; // game_app.rs に移動
// use serde_json; // game_app.rs に移動
// use crate::network::ConnectionStatus; // game_app.rs に移動
// systems モジュールと、その中の DealInitialCardsSystem を使う宣言！
// use wasm_bindgen::closure::Closure; // game_app.rs に移動
// use crate::components::dragging_info::DraggingInfo; // game_app.rs に移動
// use crate::world::World; // game_app.rs に移動
// use crate::systems::deal_system::DealInitialCardsSystem; // game_app.rs に移動

// components/ 以下の主要なコンポーネントを use 宣言！
// (ここで use したものは、このファイル内では直接型名で参照できる！)
// use crate::components::{ // game_app.rs に移動
//     card::Card,
//     position::Position,
//     player::Player,
//     stack::{StackInfo},
// };

// use crate::logic::auto_move::find_automatic_foundation_move; // game_app.rs に移動

// systems/ 以下のシステムを use 宣言！
// ★ 空の use ブロックは削除 ★

// network と protocol 関連

// JavaScript の console.log を Rust から呼び出すための準備 (extern ブロック)。
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    // ★追加: console.error も使えるようにしておく！★
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// main 関数の代わりに、Wasm がロードされた時に最初に実行される関数だよ。
#[wasm_bindgen(start)]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
    log("Panic hook set!");
}

// 簡単なテスト用の関数 (これはマルチプレイには直接関係ない)
#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello from Rust, {}!", name));
}

// --- GameApp 関連のコードは src/app/game_app.rs に移動しました ---