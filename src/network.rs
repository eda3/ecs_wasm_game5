// src/network.rs

// このファイルは、WebSocketサーバーとの通信を担当するモジュールだよ！📡
// ブラウザのWebSocket APIを使うために、`web_sys`クレートの機能と、
// RustとJavaScriptの間でやり取りするための`wasm-bindgen`クレートの機能を使うよ。
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; // JavaScriptの型とRustの型を変換するために使う
use web_sys::{ErrorEvent, MessageEvent, WebSocket}; // WebSocket関連の型
// スレッドセーフな共有状態を扱うための Arc と Mutex を使う！
// Arc: アトミック参照カウント。複数の所有者を可能にするスマートポインタ。
// Mutex: 相互排他ロック。複数のスレッド/コンテキストからデータを安全に変更可能にする。
use std::sync::{Arc, Mutex};
// メッセージキュー用の VecDeque も使うよ。
use std::collections::VecDeque;
// protocol モジュールから ServerMessage 型をインポート
use crate::protocol::ServerMessage; // onmessage で使う！
// JSON パースのために serde_json をインポート
use serde_json;
use crate::log; // lib.rs で定義した console.log を使う

// WebSocket接続の状態を表すenumだよ。
#[derive(Debug, Clone, PartialEq)] // derive で便利なトレイトを自動実装！
pub enum ConnectionStatus {
    Connected,    // 接続成功！
    Disconnected, // 切断された
    Connecting,   // 接続試行中…
    Error,        // 何らかのエラーが発生した
}

// WebSocketの接続を管理する構造体だよ。
// 今回の修正で、状態(status)やメッセージキューは外部(GameApp)から
// Arc<Mutex<>> で渡されるようになった！
pub struct NetworkManager {
    // WebSocketのインスタンスを保持するフィールド。
    // 接続が確立される前や切断後はNoneになる。
    ws: Option<WebSocket>,
    // 接続状態を保持するための共有ポインタ（GameApp と共有！）
    // コールバック関数の中からでも安全に状態を変更できるようにする。
    status_arc: Arc<Mutex<ConnectionStatus>>,
    // 受信メッセージを溜めておくキューへの共有ポインタ（GameApp と共有！）
    // onmessage コールバックがここにメッセージを追加する。
    message_queue_arc: Arc<Mutex<VecDeque<ServerMessage>>>,
    // サーバーのURLは NetworkManager が固有に持つ。
    server_url: String,
}

impl NetworkManager {
    /// 新しいNetworkManagerインスタンスを作成するよ。
    /// 外部から共有状態への参照 (`Arc<Mutex<>>`) を受け取るように変更！
    ///
    /// # 引数
    /// * `server_url` - 接続先のWebSocketサーバーのURL
    /// * `status_arc` - 接続状態を共有するための Arc<Mutex<ConnectionStatus>>
    /// * `message_queue_arc` - 受信メッセージキューを共有するための Arc<Mutex<VecDeque<ServerMessage>>>
    ///
    /// # 戻り値
    /// * 新しい`NetworkManager`インスタンス。
    pub fn new(
        server_url: String,
        status_arc: Arc<Mutex<ConnectionStatus>>,
        message_queue_arc: Arc<Mutex<VecDeque<ServerMessage>>>,
    ) -> Self {
        log(&format!("NetworkManager: Initializing with server URL: {}", server_url));
        // 初期状態は Disconnected に設定しておく。
        // `lock()` で MutexGuard を取得し、`*` で中の値にアクセスして書き換える。
        // expect はロック失敗時にパニックする。初期化時なので、通常は問題ないはず。
        *status_arc.lock().expect("Failed to lock status on init") = ConnectionStatus::Disconnected;
        Self {
            ws: None, // 最初はWebSocket接続はまだない
            status_arc, // 渡された Arc を保持
            message_queue_arc, // 渡された Arc を保持
            server_url, // サーバーURLを保存
        }
    }

    /// WebSocketサーバーへの接続を開始するよ。
    /// 接続状態の変更やメッセージの受信は、引数で受け取った Arc<Mutex<>> を介して行う！
    /// シグネチャを `&self` に変更！ 内部状態 ws 以外は Arc<Mutex<>> 経由で変更するため。
    /// (ws の変更があるため、やはり &mut self が必要。あるいは ws も Arc<Mutex<Option<WebSocket>>> に？)
    /// いや、ws は connect / disconnect でのみ変更される想定なら &mut self のままで良さそう。
    /// → connect が成功した場合のみ ws が Some になり、それは connect 関数内で行うため、
    ///   ws を変更する connect 関数自体は &mut self が必要。
    pub fn connect(&mut self) {
        // 既存の接続があれば一旦閉じる (エラー処理は省略)
        if let Some(ws) = self.ws.take() {
            let _ = ws.close(); // close() は Result を返すけど、ここでは無視
            log("NetworkManager: Closed existing WebSocket connection before reconnecting.");
        }

        // 現在の接続状態を確認 (ロックして読み取る)
        let current_status = self.status_arc.lock().expect("Failed to lock status for connect check").clone();
        if current_status == ConnectionStatus::Connecting {
            log("NetworkManager: Already attempting to connect.");
            return;
        }

        log(&format!("NetworkManager: Attempting to connect to {}...", self.server_url));
        // 状態を「接続中」に更新！
        *self.status_arc.lock().expect("Failed to lock status for Connecting") = ConnectionStatus::Connecting;

        // WebSocketオブジェクトを作成！
        match WebSocket::new(&self.server_url) {
            Ok(ws) => {
                log("NetworkManager: WebSocket object created successfully.");

                // --- コールバックに渡すための Arc をクローン！ ---
                // これが重要！ Arc をクローンすると参照カウントが増えるだけで、中身は同じものを指す。
                // このクローンした Arc をクロージャに `move` で渡すことで、
                // クロージャが実行される時にも安全に共有状態にアクセスできる！
                let status_arc_clone_open = Arc::clone(&self.status_arc);
                let status_arc_clone_error = Arc::clone(&self.status_arc);
                let status_arc_clone_close = Arc::clone(&self.status_arc);
                let queue_arc_clone_message = Arc::clone(&self.message_queue_arc);

                // (1) 接続成功時 (onopen) のコールバック
                // `move` キーワードで、クロージャが使う外部変数 (status_arc_clone_open) の
                // 所有権をクロージャ内に移動させる。
                let onopen_callback = Closure::wrap(Box::new(move |_| {
                    log("NetworkManager: WebSocket connection opened successfully! 🎉");
                    // 共有状態の status を Connected に更新！
                    let mut status = status_arc_clone_open.lock().expect("Failed to lock status on open");
                    *status = ConnectionStatus::Connected;
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget(); // メモリリーク対策！

                // (2) メッセージ受信時 (onmessage) のコールバック
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Some(message_str) = e.data().as_string() {
                        // 受信した文字列を ServerMessage にデシリアライズ！
                        match serde_json::from_str::<ServerMessage>(&message_str) {
                            Ok(message) => {
                                // パース成功！メッセージキューに追加！
                                log(&format!("NetworkManager: Parsed message: {:?}", message));
                                let mut queue = queue_arc_clone_message.lock().expect("Failed to lock queue on message");
                                queue.push_back(message); // キューの末尾に追加
                            }
                            Err(e) => {
                                // パース失敗！エラーログを出力。
                                log(&format!("NetworkManager: Failed to parse message: {}. Raw: {}", e, message_str));
                            }
                        }
                    } else {
                        log("NetworkManager: Received non-string message data.");
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();

                // (3) エラー発生時 (onerror) のコールバック
                let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                    log(&format!("NetworkManager: WebSocket error occurred: {:?}", e.message()));
                    // 共有状態の status を Error に更新！
                    let mut status = status_arc_clone_error.lock().expect("Failed to lock status on error");
                    *status = ConnectionStatus::Error;
                }) as Box<dyn FnMut(ErrorEvent)>);
                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();

                // (4) 接続切断時 (onclose) のコールバック
                let onclose_callback = Closure::wrap(Box::new(move |_| {
                    log("NetworkManager: WebSocket connection closed.");
                    // 共有状態の status を Disconnected に更新！
                    let mut status = status_arc_clone_close.lock().expect("Failed to lock status on close");
                    *status = ConnectionStatus::Disconnected;
                }) as Box<dyn FnMut(JsValue)>);
                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                onclose_callback.forget();

                // 接続が開始された WebSocket インスタンスを self.ws に保存する。
                // これで send_message などから使えるようになる。
                self.ws = Some(ws);
            }
            Err(e) => {
                // WebSocketオブジェクトの作成自体に失敗した場合
                log(&format!("NetworkManager: Failed to create WebSocket: {:?}", e));
                // 状態を「エラー」に更新！
                *self.status_arc.lock().expect("Failed to lock status on create error") = ConnectionStatus::Error;
            }
        }
    }

    /// WebSocketサーバーにメッセージを送信するよ。
    /// このメソッドは状態を変更しないので `&self` のままでOK。
    pub fn send_message(&self, message: &str) -> Result<(), &'static str> {
        if let Some(ref ws) = self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                match ws.send_with_str(message) {
                    Ok(_) => {
                        log(&format!("NetworkManager: Message sent: {}", message));
                        Ok(())
                    }
                    Err(e) => {
                        log(&format!("NetworkManager: Failed to send message: {:?}", e));
                        Err("Failed to send message")
                    }
                }
            } else {
                log("NetworkManager: Cannot send message, WebSocket is not open.");
                Err("WebSocket connection is not open")
            }
        } else {
            log("NetworkManager: Cannot send message, not connected.");
            Err("Not connected to WebSocket server")
        }
    }

    /// 現在の接続状態を取得するよ。
    /// 共有状態 `status_arc` から読み取るように変更！
    pub fn get_status(&self) -> ConnectionStatus {
        // Mutex をロックして、中の ConnectionStatus をクローンして返す。
        self.status_arc.lock().expect("Failed to lock status for get").clone()
    }

    /// WebSocket 接続を切断するよ。
    ///
    /// `ws` フィールドを変更する必要があるので `&mut self` にする。
    pub fn disconnect(&mut self) {
        // `take()` は Option から値を取り出し、元の Option を None にするメソッド。
        if let Some(ws) = self.ws.take() {
            // WebSocket の close() メソッドを呼び出す。
            // close() は Result を返すけど、ここではエラーを無視してる。（必要ならハンドリングする）
            match ws.close() {
                Ok(_) => log("NetworkManager: WebSocket connection closed by disconnect()."),
                Err(e) => log(&format!("NetworkManager: Error closing WebSocket: {:?}", e)),
            }
            // 状態も Disconnected に更新する (onclose コールバックも呼ばれるはずだけど念のため)
            *self.status_arc.lock().expect("Failed to lock status on disconnect") = ConnectionStatus::Disconnected;
        } else {
            log("NetworkManager: disconnect() called but already disconnected.");
        }
    }
}

// 注意点:
// - Mutex のロックに失敗した場合 (expect が呼ばれるケース) は、プログラムがパニックする。
//   これは他のスレッド/コンテキストがロックを持ったままパニックした場合などに起こりうる。
//   より堅牢なアプリケーションでは、expect の代わりに適切にエラーハンドリングすることが推奨されるよ！
// - Closure::forget() はメモリリークを引き起こす可能性があることに注意！
//   本来は、WebSocket オブジェクトが不要になったタイミングで Closure を解放 (drop) する仕組みが必要。
//   (例えば、NetworkManager が Drop トレイトを実装して、そこで解放処理を行うなど)
//   今回はシンプルにするために forget() を使ってるけど、長期的なプロジェクトでは見直しが必要かも！

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