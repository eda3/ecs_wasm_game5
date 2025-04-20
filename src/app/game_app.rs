// src/app/game_app.rs

// --- 必要なものをインポート ---
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, HtmlCanvasElement, CanvasRenderingContext2d};
use js_sys::Error;
// log クレートのマクロをインポート
use log::{info, error}; // warn も追加しておく

use crate::ecs::world::World;
use crate::network::NetworkManager;
use crate::protocol::{
    self, // protocol モジュール自体も使う
    ServerMessage, PlayerId,
    ClientMessage // ClientMessage も使う
};
use crate::systems::deal_system::DealInitialCardsSystem;
use crate::components::stack::StackType;
use crate::app::event_handler::{self, ClickTarget}; // event_handler モジュールと ClickTarget を use する！
use crate::{log, error}; // log と error マクロをインポート (lib.rs から)
use crate::ecs::entity::Entity; // Entity を使うためにインポート
use serde_json;
// --- レイアウト情報とレンダラー定数をインポート --- ★追加★

// ★修正: network_handler ではなく、新しいモジュールを use する★
// use super::network_handler::ProcessedMessageResult; 
use super::network_receiver::ProcessedMessageResult; // 受信結果
use crate::app::network_receiver; // ★★★ 追加！ ★★★
 // 接続
 // 送信
 // 受信処理

// ★追加: drag_handler モジュールを use する★
use super::drag_handler;

// ★追加: state_getter モジュールを use する★
use crate::app::state_getter;

// ★追加: browser_event_manager モジュールを use する★
use crate::app::browser_event_manager;

// ★修正: Result を返すように変更 (listener attach のエラーハンドル)
use wasm_bindgen::JsValue;

// ★ 追加 ★
use crate::app::stock_handler;

// --- ゲーム全体のアプリケーション状態を管理する構造体 ---
#[wasm_bindgen]
pub struct GameApp {
    world: Arc<Mutex<World>>,
    network_manager: Arc<Mutex<NetworkManager>>,
    message_queue: Arc<Mutex<VecDeque<ServerMessage>>>,
    my_player_id: Arc<Mutex<Option<PlayerId>>>,
    // DealInitialCardsSystem のインスタンスを持っておこう！ (状態を持たないので Clone でも Default でもOK)
    deal_system: DealInitialCardsSystem,
    // ★追加: イベントリスナーのクロージャを保持する Vec ★
    // Arc<Mutex<>> で囲むことで、&self からでも変更可能にし、
    // スレッドセーフにする (Wasm は基本シングルスレッドだが作法として)
    event_closures: Arc<Mutex<Vec<Closure<dyn FnMut(Event)>>>>,
    // ★追加: ドラッグ状態 (現在ドラッグ中のカード情報)★
    // dragging_state: Arc<Mutex<Option<DraggingInfo>>>, // handle_drag_start/end で直接追加/削除するので不要かも
    // ★追加: Window にアタッチする MouseMove/MouseUp リスナー★
    // (ドラッグ中のみ Some になる)
    window_mousemove_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    window_mouseup_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    // ★追加: Canvas 要素と 2D コンテキストを保持するフィールド★
    // (今回は Arc<Mutex<>> で囲まず、直接保持してみる)
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

// GameApp 構造体のメソッドを実装していくよ！
#[wasm_bindgen]
impl GameApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // log() は lib.rs で定義されているため、ここでは直接使えない
        // 必要なら crate::log() などで参照するか、GameApp 内で log を呼ぶ関数を用意する
        // println! マクロなどは使える
        println!("GameApp: 初期化中..."); // 代わりに println! を使用

        // --- World, Network, Canvas の初期化は init_handler に委譲 ---
        let world_arc = super::init_handler::initialize_world(); // app:: -> super::
        let message_queue_arc = Arc::new(Mutex::new(VecDeque::new()));
        let network_manager_arc = super::init_handler::initialize_network(Arc::clone(&message_queue_arc)); // app:: -> super::

        // Canvas 初期化 (エラー処理は expect で簡略化)
        let (canvas, context) = super::init_handler::initialize_canvas() // app:: -> super::
            .expect("Failed to initialize canvas and context");

        // --- その他のフィールド初期化 ---
        let my_player_id_arc = Arc::new(Mutex::new(None));
        let deal_system = DealInitialCardsSystem::default();
        let event_closures_arc = Arc::new(Mutex::new(Vec::new()));
        let window_mousemove_closure_arc = Arc::new(Mutex::new(None));
        let window_mouseup_closure_arc = Arc::new(Mutex::new(None));

        println!("GameApp: 初期化完了。");
        Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
            deal_system,
            event_closures: event_closures_arc,
            window_mousemove_closure: window_mousemove_closure_arc,
            window_mouseup_closure: window_mouseup_closure_arc,
            canvas,
            context,
        }
    }

    // WebSocket接続
    pub fn connect(&self) {
        // ★修正: network_connector の関数を呼び出す！★
        super::network_connector::connect(&self.network_manager);
    }

    // ゲーム参加メッセージ送信
    #[wasm_bindgen]
    pub fn send_join_game(&self, player_name: String) {
        // ★修正: network_sender の関数を呼び出す！★
        super::network_sender::send_join_game(&self.network_manager, player_name);
    }

    // カード移動メッセージ送信 (引数を JSON 文字列に戻す)
    #[wasm_bindgen]
    pub fn send_make_move(&self, moved_entity_id: usize, target_stack_json: String) { // 引数を JSON 文字列に戻す
        let moved_entity = Entity(moved_entity_id); // usize から Entity へ

        // JSON 文字列をデシリアライズ
        match serde_json::from_str::<protocol::StackType>(&target_stack_json) {
            Ok(target_stack) => {
                // デシリアライズ成功
                let message = ClientMessage::MakeMove { moved_entity, target_stack };

                match serde_json::to_string(&message) {
                    Ok(json_message) => {
                         match self.network_manager.lock() {
                             Ok(nm) => {
                                 if let Err(e) = nm.send_message(&json_message) {
                                     error!("Failed to send MakeMove message: {}", e);
                                 } else {
                                     info!("MakeMove message sent: {:?}", message);
                                 }
                             },
                             Err(e) => error!("Failed to lock NetworkManager to send MakeMove: {}", e)
                         }
                    }
                    Err(e) => error!("Failed to serialize MakeMove message: {}", e)
                }
            }
            Err(e) => {
                // JSON デシリアライズ失敗
                error!("Failed to deserialize target_stack_json: {}. JSON: {}", e, target_stack_json);
            }
        }
    }

    /// JS から呼び出され、受信メッセージキューを処理し、
    /// もしサーバーから移動拒否メッセージがあればそのカードID (usize) を返す。
    /// なければ None (JS側では undefined) を返す。
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) -> Option<usize> { 
        let results = network_receiver::process_received_messages(
            &self.message_queue,
            &self.my_player_id,
            &self.world,
        );

        // 結果の中から MoveRejected を探す
        for result in results {
            if let ProcessedMessageResult::MoveRejected { entity_id, reason: _ } = result {
                log(&format!(
                    "GameApp: MoveRejected event found for entity {:?}. Returning Some({}) to JS.", 
                    entity_id, entity_id.0
                ));
                return Some(entity_id.0); // 見つかったら ID を返す
            }
            // 他のイベントタイプ (StateChanged など) はここでは特に処理しない
            // (StateChanged などで画面更新が必要な場合は、別途JS側で render を呼ぶなどの連携が必要)
        }

        // MoveRejected が見つからなかった場合
        // log("GameApp: No MoveRejected event found in processed messages. Returning None to JS."); // ★ コメントアウト ★
        None // None を返す
    }

    // JSから初期カード配置を実行するためのメソッド
    #[wasm_bindgen]
    pub fn deal_initial_cards(&self) {
        // ★修正: app::init_handler の関数を呼び出す！★
        super::init_handler::deal_initial_cards( // app:: -> super::
            &self.world,
            &self.network_manager,
            &self.deal_system
        );
    }

    /// WASM 側 (`GameApp`) が保持しているゲームの世界 (`World`) の現在の状態を、
    /// JSON 文字列形式で取得するためのメソッドだよ！ JavaScript 側から呼び出して、
    /// デバッグ目的でコンソールに表示したり、画面描画に使ったりすることを想定してるよ！ ✨
    ///
    /// # 戻り値 (Return Value)
    /// - `Ok(String)`: `World` の状態を表す `GameStateData` を JSON 文字列に変換して返すよ！成功！🎉
    /// - `Err(JsValue)`: 何か問題が発生した場合（`World` のロック失敗、JSON への変換失敗など）は、
    ///                  JavaScript 側でエラーとして扱える `JsValue` を返すよ。失敗！😭
    ///
    /// # 処理の流れ (Process Flow)
    /// 内部で `state_getter::get_world_state_json` を呼び出すだけだよ！
    #[wasm_bindgen]
    pub fn get_world_state_json(&self) -> Result<JsValue, JsValue> { // ★戻り値を JsValue に変更★
        println!("GameApp: get_world_state_json が呼ばれました。World の状態を準備中...");

        // ★ state_getter モジュールの関数を呼び出す！★
        // self.world は Arc<Mutex<World>> なので、そのまま参照を渡せるよ！
        state_getter::get_world_state_json(&self.world)
        // 返り値は既に Result<JsValue, JsValue> なので、そのまま返す！
    }

    // 接続状態を文字列で返す (デバッグ用)
    #[wasm_bindgen]
    pub fn get_connection_status_debug(&self) -> String {
        let status = self.network_manager.lock().expect("Failed to lock NetworkManager for status").get_status();
        format!("{:?}", status)
    }

    // 自分の Player ID を返す (デバッグ用)
    #[wasm_bindgen]
    pub fn get_my_player_id_debug(&self) -> Option<u32> {
        self.my_player_id.lock().expect("Failed to lock my_player_id").map(|id| id)
    }

    /// カードがダブルクリックされた時の処理 (JSから呼び出される元のメソッド)
    #[wasm_bindgen]
    pub fn handle_double_click(&self, entity_id: usize) {
        log(&format!("GameApp: handle_double_click called for entity_id: {}", entity_id));
        // event_handler のロジック関数を呼び出す
        event_handler::handle_double_click_logic(
            entity_id,
            Arc::clone(&self.world), // Arc をクローンして渡す
            Arc::clone(&self.network_manager) // Arc をクローンして渡す
        );
    }

    /// Rust側で Canvas にゲーム画面を描画する関数
    #[wasm_bindgen]
    pub fn render_game_rust(&self) -> Result<(), JsValue> {
        super::renderer::render_game_rust( // app:: -> super::
            &self.world,
            &self.canvas,
            &self.context
        // JsValue に変換する必要があるので .map_err を追加
        ).map_err(|e| JsValue::from(Error::new(&format!("Render error: {:?}", e))))
    }

    /// JavaScript から Canvas 上でのクリックイベントを処理するために呼び出される関数だよ！
    ///
    /// # 引数
    /// * `x`: クリックされた Canvas 上の X 座標 (f32)。
    /// * `y`: クリックされた Canvas 上の Y 座標 (f32)。
    ///
    /// # 処理内容
    /// 1. `event_handler::find_clicked_element` を呼び出して、クリックされた要素 (カード or スタック) を特定する。
    /// 2. 特定された要素に応じて、ログを出力する。(デバッグ用)
    /// 3. **TODO:** 今後は、特定された要素に応じて、カードのドラッグ開始処理や、
    ///    スタッククリック時のアクション (例: 山札クリックでカードをめくる) などを実装していくよ！
    #[wasm_bindgen]
    pub fn handle_click(&mut self, x: f32, y: f32) {
        log(&format!("GameApp::handle_click: Clicked at ({}, {})", x, y));

        // --- 1. クリックされた要素を特定 --- 
        let clicked_element = {
            let world = self.world.lock().expect("Failed to lock world for click check");
            event_handler::find_clicked_element(&world, x, y)
        };
        // ★★★ ログ追加: 特定された要素を表示 ★★★
        log(&format!("  >>> Click target identified as: {:?} <<<", clicked_element));

        // --- 2. クリックされた要素に応じて処理を分岐 --- 
        match clicked_element {
            Some(ClickTarget::Card(entity)) => {
                log(&format!("  Handling click on Card: {:?}", entity));
                self.handle_double_click(entity.0); 
            }
            Some(ClickTarget::Stack(stack_type)) => {
                log(&format!("  Handling click on Stack Area: {:?}", stack_type));
                // ★★★ 山札クリック処理を追加 ★★★
                if stack_type == StackType::Stock {
                    // ★★★ ログ追加: Stock クリック処理開始 ★★★
                    log("    >>> Stock pile click detected! Entering stock handling logic... <<<");
                    let mut world_guard = self.world.lock().expect("Failed to lock world for stock click");
                    // ★★★ ログ追加: deal_one_card_from_stock 呼び出し前 ★★★
                    log("      Calling stock_handler::deal_one_card_from_stock...");
                    if !stock_handler::deal_one_card_from_stock(&mut world_guard) {
                        log("    Could not deal from stock, attempting to reset waste...");
                        // ★★★ ログ追加: reset_waste_to_stock 呼び出し前 ★★★
                        log("      Calling stock_handler::reset_waste_to_stock...");
                        stock_handler::reset_waste_to_stock(&mut world_guard);
                    }
                    // ★★★ ログ追加: Stock クリック処理終了 ★★★
                    log("    <<< Finished stock handling logic. >>>");
                }
                // ★★★ ここまで ★★★
                // 他のスタックエリアクリック時の処理 (もしあれば)
            }
            None => {
                log("  Clicked on empty area.");
            }
        }
        log("GameApp::handle_click: Finished.");
    }

    /// JavaScript から呼び出される、ドラッグ中のカードの位置を一時的に更新するためのメソッドだよ！
    /// マウスの動きに合わせてカードの見た目を追従させるために使うんだ。
    /// ⚠️ 注意: この関数は表示上の Position を更新するだけで、
    ///         カードの所属スタック (StackInfo) やゲームの論理的な状態は変更しないよ！
    ///         最終的な移動処理は handle_drag_end で行われる。
    #[wasm_bindgen]
    pub fn update_dragged_position(&mut self, entity_id: usize, mouse_x: f32, mouse_y: f32) {
        // The actual update logic is handled by drag_handler::update_dragged_position,
        // which is called by the mousemove listener.
        log(&format!(
            "GameApp: update_dragged_position called (likely redundant) for entity: {}, mouse: ({}, {})",
            entity_id,
            mouse_x,
            mouse_y
        ));
        // We could potentially call the drag_handler function here too for consistency,
        // but it's primarily driven by the listener now.
        // drag_handler::update_dragged_position(&self.world, entity_id, mouse_x, mouse_y);
    }

    /// JavaScript から呼び出して、指定された Canvas 座標 (x, y) にある
    /// 一番手前の「カード」の Entity ID を取得するための関数だよ！
    /// ダブルクリックされた時に「どのカードがクリックされたか」を JS 側で知るために使うんだ。
    ///
    /// # 引数
    /// * `x`: 判定したい Canvas 上の X 座標 (f32)。
    /// * `y`: 判定したい Canvas 上の Y 座標 (f32)。
    ///
    /// # 戻り値
    /// * `Option<usize>`:
    ///   - `Some(entity_id)`: 指定座標にカードが見つかった場合、そのカードの Entity ID (usize) を返すよ。
    ///   - `None`: 指定座標にカードが見つからなかった場合 (スタックや背景だった場合)。
    ///   JS側では number | undefined として受け取れる！
    #[wasm_bindgen]
    pub fn get_entity_id_at(&self, x: f32, y: f32) -> Option<usize> {
        // まずは World のロックを取得するよ。ロックは大事！🔒
        let world = match self.world.lock() {
            Ok(w) => w,
            Err(e) => {
                // ロックに失敗したらエラーログを出して None (何も見つからなかった) を返す。
                error!("get_entity_id_at 内で World のロックに失敗: {}", e);
                return None;
            }
        };

        // event_handler モジュールの find_clicked_element 関数を呼び出して、
        // 指定された座標 (x, y) に何があるか調べてもらう！🔍
        let clicked_element = event_handler::find_clicked_element(&world, x, y);

        // World のロックはここで解除！🔓 もう World のデータは必要ないからね。
        // drop(world) を明示的に書くことで、ロックが早く解除されることを保証するよ。
        // (ただし、スコープを抜ければ自動で解除されるので必須ではない)
        // drop(world); // 明示的な drop は通常不要

        // find_clicked_element から返ってきた結果 (Option<ClickTarget>) を match で判定！
        match clicked_element {
            // Some(ClickTarget::Card(entity)) が返ってきたら…
            Some(event_handler::ClickTarget::Card(entity)) => {
                // それはカードがクリックされたってこと！🎉
                // カードの Entity ID (entity は Entity(usize) というタプル構造体なので、中の usize を .0 で取り出す) を Some で包んで返す。
                // これで JS 側は、どのカードがクリックされたか ID を知ることができるね！
                log(&format!("get_entity_id_at: 座標 ({}, {}) でカードエンティティ {:?} を発見。", x, y, entity));
                Some(entity.0) // entity.0 は usize 型
            }
            // Some(ClickTarget::Stack(stack_type)) が返ってきたら…
            Some(event_handler::ClickTarget::Stack(stack_type)) => {
                // それはスタックの空きエリアがクリックされたってことだね。
                // 今回はカードの ID だけが欲しいので、スタックの場合は None を返す。
                log(&format!("get_entity_id_at: 座標 ({}, {}) でスタックエリア {:?} を発見。None を返します。", x, y, stack_type));
                None
            }
            // None が返ってきたら…
            None => {
                // それは背景とか、何もない場所がクリックされたってこと。
                // もちろんカードじゃないので None を返す。
                log(&format!("get_entity_id_at: 座標 ({}, {}) では何も見つからず。None を返します。", x, y));
                None
            }
        }
    }

    /// ドラッグ開始時の処理。必要なリスナーをアタッチする。
    #[wasm_bindgen]
    pub fn handle_drag_start(&mut self, entity_usize: usize, start_x: f32, start_y: f32) {
        log(&format!(
            "GameApp::handle_drag_start: Entity {}, Start: ({}, {})",
            entity_usize, start_x, start_y
        ));

        // --- 1. ドラッグ対象の情報を World に追加 --- 
        drag_handler::handle_drag_start(&self.world, entity_usize, start_x, start_y);

        // --- 2. MouseMove と MouseUp リスナーを Window にアタッチ --- 
        // (エラーハンドリングは簡単のために unwrap を使うけど、本当はちゃんと処理すべき)
        if let Err(e) = browser_event_manager::attach_drag_listeners(
            Arc::clone(&self.world),
            Arc::clone(&self.network_manager),
            Arc::clone(&self.window_mousemove_closure),
            Arc::clone(&self.window_mouseup_closure),
            entity_usize,
            &self.canvas, // self.canvas への参照を渡す
        ) {
            error!("GameApp: Failed to attach drag listeners: {:?}", e);
        }
        log("GameApp::handle_drag_start: Listeners attached.");
    }

    /// ドラッグ終了時の処理 (マウスボタンが離された時)
    /// (このメソッド自体は JS から呼ばれるけど、実際のロジックは mouseup リスナーから起動される
    /// drag_handler::handle_drag_end が中心となる)
    #[wasm_bindgen]
    pub fn handle_drag_end(&mut self, entity_usize: usize, end_x: f32, end_y: f32) {
        log(&format!(
            "GameApp::handle_drag_end: JS called for entity: {}, end: ({}, {})",
            entity_usize,
            end_x,
            end_y
        ));
        
        // ★ 重要: 実際のゲームロジック (World 更新、サーバー通知) は、
        //   browser_event_manager の mouseup リスナー内で直接 drag_handler::handle_drag_end が
        //   呼び出されることで実行される。
        //   なので、この GameApp::handle_drag_end メソッドが JS から呼ばれたときに
        //   重複してロジックを実行する必要は *ない*。
        //   もし JS 側で mouseup イベントと同時にこの関数を呼んでいるなら、
        //   この関数の中身はログ出力程度で良いかもしれない。
        //   あるいは、リスナーのデタッチ漏れを防ぐ目的で呼ぶ？ (現状はリスナー内でデタッチしてるはず)
        log("GameApp::handle_drag_end: Logic execution relies on internal mouseup listener.");
        
        // --- もし、ここでリスナーデタッチの再確認が必要なら --- 
        // if let Err(e) = browser_event_manager::detach_drag_listeners(
        //     &self.window_mousemove_closure,
        //     &self.window_mouseup_closure,
        // ) {
        //     error!("GameApp: Error potentially re-detaching listeners in handle_drag_end: {:?}", e);
        // }
    }

} // impl GameApp の終わり

// GameApp が不要になった時に WebSocket 接続を閉じる処理 (Drop トレイト)
impl Drop for GameApp {
    fn drop(&mut self) {
        println!("GameApp: GameApp インスタンスを破棄中。WebSocket を切断します...");
        match self.network_manager.lock() {
            Ok(mut nm) => nm.disconnect(),
            Err(e) => eprintln!("GameApp: 切断のために NetworkManager のロックに失敗: {:?}", e),
        }
    }
} 