// src/network.rs

// このファイルは、WebSocketサーバーとの通信を担当するモジュールだよ！📡
// ブラウザのWebSocket APIを使うために、`web_sys`クレートの機能と、
// RustとJavaScriptの間でやり取りするための`wasm-bindgen`クレートの機能を使うよ。
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; // JavaScriptの型とRustの型を変換するために使う
use web_sys::{ErrorEvent, MessageEvent, WebSocket}; // WebSocket関連の型
use std::sync::{Arc, Mutex}; // スレッドセーフな共有状態を扱うため (後で使うかも？)
use crate::log; // lib.rs で定義した console.log を使う

// WebSocket接続の状態を表すenumだよ。今はシンプルにConnectedとDisconnectedだけ。
// 将来的には、Connecting（接続中）とかError（エラー発生）とか追加するともっと親切かもね！
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting, // 接続試行中
    Error,      // 何らかのエラーが発生
}

// WebSocketの接続を管理する構造体だよ。
pub struct NetworkManager {
    // WebSocketのインスタンスを保持するフィールド。
    // Option型なのは、接続が確立される前や切断後はNoneになる可能性があるからだよ。
    ws: Option<WebSocket>,
    // 接続状態を保持するフィールド。
    // TODO: Arc<Mutex<>> とかでスレッドセーフにする必要があるかも？
    //       メインループや他のコールバックから状態を共有・変更する場合。
    //       今のところはシンプルに持つ。
    status: ConnectionStatus,
    // サーバーのURLを保持するフィールド。
    server_url: String,
    // TODO: 受信メッセージを処理するコールバック関数を保持するフィールドを追加する？
    //       例: on_message_callback: Option<Box<dyn FnMut(String)>>,
}

impl NetworkManager {
    /// 新しいNetworkManagerインスタンスを作成するよ。
    ///
    /// # 引数
    /// * `server_url` - 接続先のWebSocketサーバーのURL (例: "ws://127.0.0.1:8101")
    ///
    /// # 戻り値
    /// * 新しい`NetworkManager`インスタンス。初期状態は`Disconnected`だよ。
    pub fn new(server_url: String) -> Self {
        log(&format!("NetworkManager: Initializing with server URL: {}", server_url));
        Self {
            ws: None, // 最初はWebSocket接続はまだない
            status: ConnectionStatus::Disconnected, // 初期状態は「切断」
            server_url, // サーバーURLを保存
        }
    }

    /// WebSocketサーバーへの接続を開始するよ。
    ///
    /// すでに接続中や接続済みだったら何もしないよ。
    /// 接続試行中にエラーが起きたら、ステータスをErrorにするよ。
    pub fn connect(&mut self) {
        // すでに接続済み、または接続試行中なら何もしない
        if self.status == ConnectionStatus::Connected || self.status == ConnectionStatus::Connecting {
            log("NetworkManager: Already connected or connecting.");
            return;
        }

        log(&format!("NetworkManager: Attempting to connect to {}...", self.server_url));
        self.status = ConnectionStatus::Connecting; // 状態を「接続中」に更新

        // WebSocketオブジェクトを作成！ web_sys::WebSocket::new() を使うよ。
        // URLが不正だったりするとエラー (Result<WebSocket, JsValue>) が返るから、ちゃんと処理する。
        match WebSocket::new(&self.server_url) {
            Ok(ws) => {
                // 成功！ WebSocketインスタンスを保持する。
                log("NetworkManager: WebSocket object created successfully.");

                // WebSocketはバイナリデータも送受信できるけど、今回はJSON文字列を使う想定だから、
                // バイナリタイプを "arraybuffer" (または "blob") に設定しておく。
                // (web-sysがデフォルトでどっちを期待するかによるかも？要確認！)
                // ここではとりあえずコメントアウト。必要なら設定する。
                // ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                // --- WebSocketイベントハンドラの設定 ---
                // ここで、接続が開いた時、メッセージを受け取った時、エラーが起きた時、接続が閉じた時に
                // それぞれ実行される処理（コールバック関数）を設定していくよ！

                // (1) 接続成功時 (onopen) のコールバック
                let onopen_callback = Closure::wrap(Box::new(|_| {
                    log("NetworkManager: WebSocket connection opened successfully! 🎉");
                    // TODO: ここで status を Connected に更新する必要がある！
                    //       そのためには、status を Closure にキャプチャさせる必要があるけど、
                    //       `&mut self` をキャプチャできない。Arc<Mutex<ConnectionStatus>> が必要そう。
                }) as Box<dyn FnMut(JsValue)>);
                // 作成したコールバックを onopen プロパティに設定！
                ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                // コールバックがRust側で破棄されないように「忘れる」必要があるよ (メモリリーク注意！)
                onopen_callback.forget();

                // (2) メッセージ受信時 (onmessage) のコールバック
                let onmessage_callback = Closure::wrap(Box::new(|e: MessageEvent| {
                    // MessageEventから実際のメッセージデータを取り出す！
                    // データはテキスト (String) のはずなので、as_string() で変換を試みる。
                    if let Some(message) = e.data().as_string() {
                        log(&format!("NetworkManager: Message received: {}", message));
                        // TODO: 受信したメッセージを処理するロジックを呼び出す！
                        //       例えば、上で定義した on_message_callback を実行するとか。
                    } else {
                        log("NetworkManager: Received non-string message data.");
                        // TODO: テキスト以外のデータ (バイナリとか) を受信した場合の処理も必要かも？
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();

                // (3) エラー発生時 (onerror) のコールバック
                let onerror_callback = Closure::wrap(Box::new(|e: ErrorEvent| {
                    // エラーイベントの詳細を出力する (内容はブラウザ依存かも)
                    log(&format!("NetworkManager: WebSocket error occurred: {:?}", e.message()));
                    // TODO: status を Error に更新する必要がある！ (これも Arc<Mutex<>> が必要)
                }) as Box<dyn FnMut(ErrorEvent)>);
                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();

                // (4) 接続切断時 (onclose) のコールバック
                let onclose_callback = Closure::wrap(Box::new(|_| {
                    log("NetworkManager: WebSocket connection closed.");
                    // TODO: status を Disconnected に更新する必要がある！ (これも Arc<Mutex<>> が必要)
                    // TODO: 再接続ロジックとかを入れる？
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                onclose_callback.forget();

                // 接続が確立されたWebSocketインスタンスを self.ws に保存する
                self.ws = Some(ws);
                // TODO: ↑でコメントしたように、この時点ではまだConnectedじゃない！
                //       onopenコールバックが呼ばれたらConnectedになる。

            }
            Err(e) => {
                // WebSocketオブジェクトの作成に失敗した場合
                log(&format!("NetworkManager: Failed to create WebSocket: {:?}", e));
                self.status = ConnectionStatus::Error; // 状態を「エラー」に更新
            }
        }
    }

    /// WebSocketサーバーにメッセージを送信するよ。
    ///
    /// # 引数
    /// * `message` - 送信する文字列メッセージ。JSON形式の文字列を想定してるよ。
    ///
    /// # 戻り値
    /// * `Ok(())` - 送信に成功した場合 (非同期なので、実際に送信されたかは保証しない)。
    /// * `Err(&str)` - 接続されていない、または送信中にエラーが発生した場合。
    pub fn send_message(&self, message: &str) -> Result<(), &'static str> {
        // まず、WebSocket接続が存在するか (`self.ws` が `Some` か) を確認する。
        // `if let Some(ref ws) = self.ws` は、`self.ws` が `Some` の場合に中身 (ws) を取り出す構文だよ。
        if let Some(ref ws) = self.ws {
            // 接続状態を確認する。`web_sys::WebSocket` の `ready_state()` メソッドを使うよ。
            // `OPEN` (値は1) だったら送信可能！
            if ws.ready_state() == WebSocket::OPEN {
                // `send_with_str()` メソッドで文字列メッセージを送信する。
                // このメソッドもエラー (Result<(), JsValue>) を返す可能性があるから、`match` で処理する。
                match ws.send_with_str(message) {
                    Ok(_) => {
                        // 送信処理の呼び出し成功！
                        log(&format!("NetworkManager: Message sent: {}", message));
                        Ok(()) // 成功を返す
                    }
                    Err(e) => {
                        // 送信処理の呼び出し失敗！
                        log(&format!("NetworkManager: Failed to send message: {:?}", e));
                        Err("Failed to send message") // エラーメッセージを返す
                    }
                }
            } else {
                // WebSocket接続が開いていない (接続中、閉じている、など) 場合
                log("NetworkManager: Cannot send message, WebSocket is not open.");
                Err("WebSocket connection is not open") // エラーメッセージを返す
            }
        } else {
            // WebSocket接続自体が存在しない場合
            log("NetworkManager: Cannot send message, not connected.");
            Err("Not connected to WebSocket server") // エラーメッセージを返す
        }
    }

    /// 現在の接続状態を取得するよ。
    pub fn get_status(&self) -> ConnectionStatus {
        // TODO: コールバックから状態を更新できるように Arc<Mutex<>> を使うようになったら、
        //       Mutexをロックして値のクローンを返す必要がある。
        self.status.clone()
    }

    // TODO: 切断処理 (disconnect) メソッドも必要だね！
    // pub fn disconnect(&mut self) { ... }
}

// Wasm-bindgenの制約で、Closure内で`self` (特に`&mut self`) を直接キャプチャできない問題がある。
// そのため、状態(status)の更新や、受信メッセージの処理をNetworkManagerの外部 (例えばメインループとか)
// に通知する仕組みが必要になる。
// 方法としては：
// 1. Arc<Mutex<>> で状態を共有する (コールバック内でもロックして変更できるようにする)。
// 2. 受信メッセージや状態変化をキューやチャンネルに入れて、メインループ側でポーリングして処理する。
// 3. コールバック関数をNetworkManagerの初期化時に外部から渡してもらう。
//
// 今回はまず基本的な接続と送信の骨組みを作って、状態更新やメッセージ処理の連携は
// 次のステップ以降で考えていくことにするよ！💪 