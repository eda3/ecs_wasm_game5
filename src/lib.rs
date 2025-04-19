// src/lib.rs

// wasm-bindgen クレートを使う宣言！RustとJavaScriptを繋ぐ魔法！🪄
use wasm_bindgen::prelude::*;

// entity.rs と component.rs をこのライブラリクレートのモジュールとして使えるようにするよ！
// これで、lib.rs から Entity や ComponentStorage とかにアクセスできるようになるんだ✨
pub mod entity;
pub mod component;

// デバッグ用に、パニック（エラー）が起きた時にコンソールのエラー出力に詳細を出す設定！
// これを最初に一回呼んでおくと、何か問題が起きた時に原因を突き止めやすくなるよ！👍
#[wasm_bindgen(start)] // Wasmが読み込まれた時に最初に実行される関数に指定！
pub fn set_panic_hook() {
    // `console_error_panic_hook` クレートの `set_once` 関数を呼ぶだけ！
    // これで、パニックが発生したら自動的に console.error に情報が出るようになるよ。超便利！💖
    console_error_panic_hook::set_once();
    // ちゃんと設定されたか、コンソールにメッセージを出してみる！(これは無くてもOK！) 
    log("RUST Main: panic hook set!");
}

// JavaScript の console.log を Rust から呼び出すためのヘルパー関数！
// `#[wasm_bindgen]` をつけると、この関数がJavaScript側からも呼べるようになるよ！すごい！🤩
// `extern "C"` は、C言語の関数呼び出し規約を使うっていう意味だけど、今は気にしなくてOK！
#[wasm_bindgen]
extern "C" {
    // JavaScript側の `console.log` 関数を Rust で表現してるよ。
    // `#[wasm_bindgen(js_namespace = console)]` で `console` オブジェクトの `log` 関数だよ、って教えてるんだ。
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str); // 文字列を引数に取るよ。
}

// 簡単なテスト用の関数！これもJavaScriptから呼べるように `#[wasm_bindgen]` をつけるよ。
// これをJavaScriptから呼んで、ちゃんと "Hello from Rust!" ってコンソールに出れば、
// RustとJavaScriptの連携はうまくいってるってこと！🎉
#[wasm_bindgen]
pub fn greet(name: &str) {
    // さっき定義した log 関数を使って、メッセージをコンソールに出力！
    // format! マクロは、文字列の中に変数を埋め込むのに使うよ。
    log(&format!("Hello from Rust, {}!", name));
}

// ここから下は、WorldとかSystemとか作った時に追加していく予定！
// 今は、cargo check が通るようにするための最小限の構成だよ！😊 