// src/app/game_app.rs

// --- 必要なものをインポート ---
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent};
use js_sys::Error;
// log クレートのマクロをインポート
// use log::{info, error}; // ★★★ 削除: lib.rs のマクロと衝突するため ★★★

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
use crate::log; // log と error マクロをインポート (lib.rs から)
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
use crate::app::browser_event_manager; // ★ 警告修正: 未使用のため削除 ★ ← 元に戻す！

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
    // ★★★ 削除: 汎用的なリスナー保持 Vec ★★★
    // event_closures: Arc<Mutex<Vec<Closure<dyn FnMut(Event)>>>>,

    // ★★★ 追加: Canvas 用の個別リスナー保持フィールド ★★★
    canvas_click_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    canvas_dblclick_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    canvas_mousedown_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,

    // ★ Window にアタッチする MouseMove/MouseUp リスナー (これは元々あった)
    window_mousemove_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    window_mouseup_closure: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,

    // Canvas 要素と 2D コンテキスト (これも元々あった)
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
        let canvas_click_closure_arc = Arc::new(Mutex::new(None));
        let canvas_dblclick_closure_arc = Arc::new(Mutex::new(None));
        let canvas_mousedown_closure_arc = Arc::new(Mutex::new(None));
        let window_mousemove_closure_arc = Arc::new(Mutex::new(None));
        let window_mouseup_closure_arc = Arc::new(Mutex::new(None));

        // --- GameApp インスタンス生成 ---
        let game_app = Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
            deal_system,
            canvas_click_closure: canvas_click_closure_arc,
            canvas_dblclick_closure: canvas_dblclick_closure_arc,
            canvas_mousedown_closure: canvas_mousedown_closure_arc,
            window_mousemove_closure: window_mousemove_closure_arc,
            window_mouseup_closure: window_mouseup_closure_arc,
            canvas,
            context,
        };

        // ★★★ Rc<RefCell<>> で包んでリスナーを設定 ★★★
        let game_app_rc = Rc::new(RefCell::new(game_app));
        if let Err(e) = Self::setup_canvas_listeners(Rc::clone(&game_app_rc)) {
             // エラーを JS の console.error に表示したいけど、log! は使えない…
             // とりあえず println! で出す
             println!("Error setting up canvas listeners: {:?}", e);
             // パニックさせるか、エラー状態を持つか… ここではとりあえず続行
        }

        println!("GameApp: 初期化完了。");

        // ★★★ Rc<RefCell<>> から GameApp を取り出して返す ★★★
        match Rc::try_unwrap(game_app_rc) {
            Ok(cell) => cell.into_inner(),
            Err(_) => {
                // これは通常起こらないはず (他に参照が残ってないため)
                panic!("Failed to unwrap Rc<RefCell<GameApp>> during initialization");
            }
        }
    }

    /// Canvas にイベントリスナーを設定するヘルパー関数
    fn setup_canvas_listeners(game_app_rc: Rc<RefCell<GameApp>>) -> Result<(), JsValue> {
        let canvas = game_app_rc.borrow().canvas.clone(); // キャンバスへの参照を取得 (clone が必要)

        // --- Click Listener --- (例)
        {
            let game_app_clone = Rc::clone(&game_app_rc);
            let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
                // Event を MouseEvent にキャスト
                if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                    // Canvas ローカル座標を取得 (ヘルパー関数を使う想定)
                    // TODO: get_canvas_coordinates を実装またはインポート
                    let coords = Self::get_canvas_coordinates_from_event(&game_app_clone.borrow().canvas, &mouse_event);
                    if let Some((x, y)) = coords {
                        // GameApp のメソッド呼び出し
                        game_app_clone.borrow_mut().handle_click(x, y);
                    }
                } else {
                     println!("Failed to cast to MouseEvent in click listener");
                }
            });

            canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            // 作成したクロージャを GameApp に保存
            *game_app_rc.borrow_mut().canvas_click_closure.lock().unwrap() = Some(closure);
            // closure.forget(); // drop で解除するので forget しない！
        }

        // --- DblClick Listener --- ★★★ 実装 ★★★
        {
            let game_app_clone = Rc::clone(&game_app_rc);
            let canvas_clone = canvas.clone(); // 座標取得用に canvas も clone
            let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
                if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                    let coords = Self::get_canvas_coordinates_from_event(&canvas_clone, &mouse_event);
                    if let Some((x, y)) = coords {
                        // ダブルクリックされた場所の Entity ID を取得
                        let entity_id_opt = game_app_clone.borrow().get_entity_id_at(x, y);
                        if let Some(entity_id) = entity_id_opt {
                            // Entity があれば handle_double_click を呼び出す
                            game_app_clone.borrow().handle_double_click(entity_id);
                        } else {
                            // カードがない場所でのダブルクリックは無視
                             println!("DblClick on empty area ignored.");
                        }
                    }
                } else {
                     println!("Failed to cast to MouseEvent in dblclick listener");
                }
            });
            canvas.add_event_listener_with_callback("dblclick", closure.as_ref().unchecked_ref())?;
            *game_app_rc.borrow_mut().canvas_dblclick_closure.lock().unwrap() = Some(closure);
        }

        // --- MouseDown Listener --- ★★★ 実装 ★★★
        {
            let game_app_clone = Rc::clone(&game_app_rc);
            let canvas_clone = canvas.clone(); // 座標取得用に canvas も clone
            let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
                if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                    // 左クリック (button 0) 以外は無視
                    if mouse_event.button() != 0 {
                         println!("Ignoring non-left mousedown event.");
                        return;
                    }

                    let coords = Self::get_canvas_coordinates_from_event(&canvas_clone, &mouse_event);
                    if let Some((x, y)) = coords {
                        // マウスダウンされた場所の Entity ID を取得
                        let entity_id_opt = game_app_clone.borrow().get_entity_id_at(x, y);
                        if let Some(entity_id) = entity_id_opt {
                            // Entity があれば handle_drag_start を呼び出す
                            // handle_drag_start は &mut self だけど、Rc<RefCell<>> 経由で呼べる！
                            game_app_clone.borrow_mut().handle_drag_start(entity_id, x, y);
                        } else {
                            // カードがない場所でのマウスダウンは無視 (ドラッグ開始しない)
                            println!("Mousedown on empty area ignored.");
                        }
                    }
                } else {
                     println!("Failed to cast to MouseEvent in mousedown listener");
                }
            });
            canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            *game_app_rc.borrow_mut().canvas_mousedown_closure.lock().unwrap() = Some(closure);
        }

        println!("Canvas listeners set up.");
        Ok(())
    }

    /// MouseEvent から Canvas ローカル座標を取得する (仮実装)
    /// TODO: bootstrap.js の getCanvasCoordinates と同じロジックを実装
    fn get_canvas_coordinates_from_event(canvas: &HtmlCanvasElement, event: &MouseEvent) -> Option<(f32, f32)> {
        let rect = canvas.get_bounding_client_rect();
        let x = event.client_x() as f32 - rect.left() as f32;
        let y = event.client_y() as f32 - rect.top() as f32;
        Some((x, y))
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
                                 if let Err(_e) = nm.send_message(&json_message) {
                                     // error!("Failed to send MakeMove message: {}", _e);
                                 } else {
                                     // info!("MakeMove message sent: {:?}", message);
                                 }
                             },
                             Err(_e) => {} // error!("Failed to lock NetworkManager to send MakeMove: {}", _e)
                         }
                    }
                    Err(_e) => {} // error!("Failed to serialize MakeMove message: {}", _e)
                }
            }
            Err(_e) => {
                // JSON デシリアライズ失敗
                // error!("Failed to deserialize target_stack_json: {}. JSON: {}", _e, target_stack_json);
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
                /* log(&format!(
                    "GameApp: MoveRejected event found for entity {:?}. Returning Some({}) to JS.", 
                    entity_id, entity_id.0
                )); */
                return Some(entity_id.0);
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
        // log(&format!("GameApp::handle_click: Clicked at ({}, {})", x, y));

        let mut world_guard = match self.world.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                // error!("Failed to lock world in handle_click: {}", e);
                return;
            }
        };

        let clicked_element = event_handler::find_clicked_element(&*world_guard, x, y);
        // log(&format!("  >>> Click target identified as: {:?}", clicked_element));

        match clicked_element {
            Some(ClickTarget::Card(_entity)) => {
                // log(&format!("  Handling click on Card: {:?}", _entity));
            }
            Some(ClickTarget::Stack(stack_type)) => {
                // log(&format!("  Handling click on Stack Area: {:?}", stack_type));
                if stack_type == StackType::Stock {
                    // log!("[Input] Stock clicked");
                    if !stock_handler::deal_one_card_from_stock(&mut *world_guard) {
                        let _ = stock_handler::reset_waste_to_stock(&mut *world_guard);
                        // log!("[Input] Tried resetting waste to stock (stock was likely empty).");
                    } else {
                        // log!("[Input] Dealt one card from stock to waste.");
                    }
                } else {
                    // log(&format!("  Clicked on {:?} stack area (no action defined).", stack_type));
                }
            }
            None => {
                // log("  Clicked on empty area.");
            }
        }
        // log("GameApp::handle_click: Finished.");
    }

    /// JSから呼び出され、ドラッグ中のカード位置を更新する。
    /// (内部リスナー削除により、呼び出し元が変わる可能性あり)
    pub fn update_dragged_position(&mut self, entity_id: usize, mouse_x: f32, mouse_y: f32) {
        // ★注意: 内部リスナーを削除したので、この関数が JS から呼ばれるようにする必要があるかも★
        //       現時点では bootstrap.js 側の mousemove リスナーから呼ばれる想定で残しておく
        // log(&format!(
        //     "GameApp: update_dragged_position called (likely redundant) for entity: {}, mouse: ({}, {})",
        //     entity_id, mouse_x, mouse_y
        // ));
        drag_handler::update_dragged_position(
            &self.world,
            entity_id,
            mouse_x,
            mouse_y
        );
    }

    /// 指定された座標にある一番手前のカードの Entity ID を返す (JS呼び出し用)
    pub fn get_entity_id_at(&self, x: f32, y: f32) -> Option<usize> {
        let world = match self.world.try_lock() {
            Ok(guard) => guard,
            Err(_e) => {
                // error!("Failed to lock world in get_entity_id_at: {}", _e);
                return None;
            }
        };

        match event_handler::find_topmost_clicked_card(&world, x, y) {
            Some(ClickTarget::Card(entity)) => {
                // log(&format!("get_entity_id_at: 座標 ({}, {}) でカードエンティティ {:?} を発見。", x, y, entity));
                Some(entity.0)
            }
            _ => {
                // log(&format!("get_entity_id_at: 座標 ({}, {}) にカードは見つかりませんでした。", x, y));
                None
            }
        }
    }

    /// ドラッグ開始時に JS から呼ばれる
    pub fn handle_drag_start(&mut self, entity_usize: usize, start_x: f32, start_y: f32) {
        // log(&format!("GameApp::handle_drag_start: Entity {}, Start: ({}, {})", entity_usize, start_x, start_y));

        // 1. drag_handler を呼び出して DraggingInfo を追加
        drag_handler::handle_drag_start(&self.world, entity_usize, start_x, start_y);

        // ★★★ ステップ6: 内部リスナーのアタッチ処理を復活させる ★★★
        // --- 復活！ ---
        if let Err(e) = browser_event_manager::attach_drag_listeners(
            Arc::clone(&self.world),
            Arc::clone(&self.network_manager), // network_manager も渡す
            // ★ 修正: attach_drag_listeners の引数に合わせる ★
            // Entity(entity_usize), // entity_usize を Entity に変換して渡す
            Arc::clone(&self.window_mousemove_closure),
            Arc::clone(&self.window_mouseup_closure),
            entity_usize, // entity_id として usize を渡す
            &self.canvas, // canvas への参照を渡す (座標変換に必要)
        ) {
            // error!("Failed to attach drag listeners: {:?}", e);
            println!("Failed to attach drag listeners: {:?}", e); // とりあえず println
        }
        // ★★★ ここまで復活 ★★★

        // このログはリスナー削除後には不要かも
        // info!("GameApp::handle_drag_start: Listeners attached (moved to browser_event_manager).");
    }

    /// ドラッグ終了時に JS から呼ばれる
    pub fn handle_drag_end(&mut self, entity_usize: usize, end_x: f32, end_y: f32) {
        // log(&format!("GameApp::handle_drag_end: JS called for entity: {}, end: ({}, {})", entity_usize, end_x, end_y));
        drag_handler::handle_drag_end(
            &self.world,
            &self.network_manager,
            entity_usize,
            end_x,
            end_y
        );
    }

    // // ★内部リスナー用だった handle_drag_end は不要になるのでコメントアウト or 削除★
    // // (browser_event_manager 側で直接 drag_handler を呼ぶように変更した場合)
    // fn handle_drag_end_internal(&mut self, entity_usize: usize, end_x: f32, end_y: f32) {
    //     log(&format!("GameApp::handle_drag_end_internal: Entity {}, End: ({}, {})", entity_usize, end_x, end_y));
    //     drag_handler::handle_drag_end(
    //         &self.world,
    //         &self.network_manager,
    //         entity_usize,
    //         end_x, 
    //         end_y,
    //         // ★削除: Closure Arc は handle_drag_end では直接使わない ★
    //     );
    // }

}

// --- GameApp の Drop 実装 (クリーンアップ用) ---
impl Drop for GameApp {
    fn drop(&mut self) {
        println!("GameApp is being dropped. Cleaning up listeners...");

        // ★★★ Canvas リスナーを解除 ★★★
        if let Err(_e) = browser_event_manager::detach_canvas_listeners(
            &self.canvas,
            &self.canvas_click_closure,
            &self.canvas_dblclick_closure,
            &self.canvas_mousedown_closure,
        ) {
            // ここでも console.error に出したいけど…
            println!("Error detaching canvas listeners: {:?}", _e);
        }

        // ★★★ Window (ドラッグ) リスナーを解除 ★★★
        if let Err(e) = browser_event_manager::detach_drag_listeners(
            &self.window_mousemove_closure,
            &self.window_mouseup_closure,
        ) {
            println!("Error detaching window drag listeners: {:?}", e);
        }

        // ★★★ 削除: 以前の .clear() 呼び出し ★★★
        // self.canvas_click_closure.lock().unwrap().clear();
        // self.canvas_dblclick_closure.lock().unwrap().clear();
        // self.canvas_mousedown_closure.lock().unwrap().clear();
        // self.window_mousemove_closure.lock().unwrap().clear();
        // self.window_mouseup_closure.lock().unwrap().clear();

        println!("Listeners detached.");
    }
} 