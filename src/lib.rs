// src/lib.rs

// wasm-bindgen クレートを使う宣言！RustとJavaScriptを繋ぐ魔法！🪄
use wasm_bindgen::prelude::*;

// --- ECS Core Modules --- 
// Entity, Component (trait), World, System (trait) の基本的な部品！
pub mod entity;
pub mod component; // Component トレイトと ComponentStorage
pub mod world;
pub mod system;

// --- Game Specific Components --- 
// components/ ディレクトリ以下をモジュールとして宣言！
pub mod components;

// --- Game Specific Systems --- 
// systems/ ディレクトリ以下をモジュールとして宣言！
pub mod systems;

// デバッグ用に、パニック（エラー）が起きた時にコンソールのエラー出力に詳細を出す設定！
// これを最初に一回呼んでおくと、何か問題が起きた時に原因を突き止めやすくなるよ！👍
#[wasm_bindgen(start)] // Wasmが読み込まれた時に最初に実行される関数に指定！
pub fn set_panic_hook() {
    // ... existing code ...
}

// JavaScript の console.log を Rust から呼び出すためのヘルパー関数！
// ... existing code ...
#[wasm_bindgen]
extern "C" {
    // ... existing code ...
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // 文字列を引数に取るよ。
}

// 簡単なテスト用の関数！これもJavaScriptから呼べるように `#[wasm_bindgen]` をつけるよ。
// ... existing code ...
#[wasm_bindgen]
pub fn greet(name: &str) {
    // ... existing code ...
    log(&format!("Hello from Rust, {}!", name));
}

// ここから下は、WorldとかSystemとか作った時に追加していく予定！
// 今は、cargo check が通るようにするための最小限の構成だよ！😊