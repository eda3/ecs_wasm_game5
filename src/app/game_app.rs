// src/app/game_app.rs

// --- 必要なものをインポート ---
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
// use std::io::Error; // ★ 削除 ★

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent};
use log::error;
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
// use crate::app::stock_handler; // ★ 削除 ★

// ★ 追加: layout_calculator と components を使うための use 文 ★
use crate::app::layout_calculator;
use crate::components::{Card, Position, StackInfo}; // ★ self を削除 ★

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
        // ★ Weak ポインタを作成 ★
        let game_app_weak = Rc::downgrade(&game_app_rc);

        // ★ setup_canvas_listeners に Weak ポインタを渡す ★
        if let Err(e) = Self::setup_canvas_listeners(game_app_weak /* Rc ではなく Weak */) {
             // エラーを JS の console.error に表示したいけど、log! は使えない…
             // とりあえず println! で出す
             println!("Error setting up canvas listeners: {:?}", e);
             // パニックさせるか、エラー状態を持つか… ここではとりあえず続行
        }

        println!("GameApp: 初期化完了。");

        // ★★★ Rc<RefCell<>> から GameApp を取り出して返す (Weak を使ったので成功するはず) ★★★
        match Rc::try_unwrap(game_app_rc) {
            Ok(cell) => cell.into_inner(),
            Err(_) => {
                // Weak を使ったので、通常ここには来ないはず
                panic!("Failed to unwrap Rc<RefCell<GameApp>> during initialization despite using Weak pointers");
            }
        }
    }

    /// Canvas にイベントリスナーを設定するヘルパー関数
    // ★ 引数を Weak ポインタに変更 ★
    fn setup_canvas_listeners(game_app_weak: Weak<RefCell<GameApp>>) -> Result<(), JsValue> {
        // Canvas を取得するために Weak ポインタをアップグレード (borrow する必要はない)
        // アップグレードできない (= GameApp が既に Drop されている) 場合はリスナー設定できない
        let canvas = match game_app_weak.upgrade() {
            Some(rc) => rc.borrow().canvas.clone(),
            None => return Err(JsValue::from_str("Cannot setup listeners: GameApp already dropped?")),
        };
        // let canvas = game_app_rc.borrow().canvas.clone(); // 古いコード

        // --- Click Listener ---
        {
            // ★ Weak ポインタをクローンしてクロージャにキャプチャ ★
            let game_app_weak_clone = game_app_weak.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
                // ★ Weak ポインタをアップグレードして Rc を取得 ★
                if let Some(game_app_rc) = game_app_weak_clone.upgrade() {
                    if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                        let coords = Self::get_canvas_coordinates_from_event(&game_app_rc.borrow().canvas, &mouse_event);
                        if let Some((x, y)) = coords {
                            // ★ Rc を使ってメソッド呼び出し (borrow_mut) ★
                            game_app_rc.borrow_mut().handle_click(x, y);
                        }
                    } else {
                         println!("Failed to cast to MouseEvent in click listener");
                    }
                } else {
                     println!("GameApp weak ref upgrade failed in click listener");
                }
            });

            canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            // ★ GameApp 取得時にアップグレード必要 ★
            if let Some(rc) = game_app_weak.upgrade() {
                 *rc.borrow_mut().canvas_click_closure.lock().unwrap() = Some(closure);
            } else {
                // GameApp が存在しない場合はクロージャを保存できない (が、通常は発生しないはず)
                println!("Warning: Could not store click closure as GameApp was dropped?");
            }
        }

        // --- DblClick Listener --- ★★★ 修正 ★★★
        {
            // ★ Weak ポインタをクローン ★
            let game_app_weak_clone = game_app_weak.clone();
            let canvas_clone = canvas.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
                // ★ Weak ポインタをアップグレード ★
                if let Some(game_app_rc) = game_app_weak_clone.upgrade() {
                    if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                        let coords = Self::get_canvas_coordinates_from_event(&canvas_clone, &mouse_event);
                        if let Some((x, y)) = coords {
                            // ★ Rc を使ってメソッド呼び出し (borrow) ★
                            let entity_id_opt = game_app_rc.borrow().get_entity_id_at(x, y);
                            if let Some(entity_id) = entity_id_opt {
                                // ★ Rc を使ってメソッド呼び出し (borrow) ★
                                game_app_rc.borrow().handle_double_click(entity_id);
                            } else {
                                 println!("DblClick on empty area ignored.");
                            }
                        }
                    } else {
                         println!("Failed to cast to MouseEvent in dblclick listener");
                    }
                } else {
                    println!("GameApp weak ref upgrade failed in dblclick listener");
                }
            });
            canvas.add_event_listener_with_callback("dblclick", closure.as_ref().unchecked_ref())?;
            // ★ GameApp 取得時にアップグレード必要 ★
            if let Some(rc) = game_app_weak.upgrade() {
                *rc.borrow_mut().canvas_dblclick_closure.lock().unwrap() = Some(closure);
            } else {
                println!("Warning: Could not store dblclick closure as GameApp was dropped?");
            }
        }

        // --- MouseDown Listener --- ★★★ 修正 ★★★
        {
            // ★ Weak ポインタをクローン ★
            let game_app_weak_clone = game_app_weak.clone();
            let canvas_clone = canvas.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
                // ★ Weak ポインタをアップグレード ★
                if let Some(game_app_rc) = game_app_weak_clone.upgrade() {
                    if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                        if mouse_event.button() != 0 {
                             println!("Ignoring non-left mousedown event.");
                            return;
                        }

                        let coords = Self::get_canvas_coordinates_from_event(&canvas_clone, &mouse_event);
                        if let Some((x, y)) = coords {
                            // ★ Rc を使ってメソッド呼び出し (borrow) ★
                            let entity_id_opt = game_app_rc.borrow().get_entity_id_at(x, y);
                            if let Some(entity_id) = entity_id_opt {
                                // ★ Rc を使ってメソッド呼び出し (borrow_mut) ★
                                game_app_rc.borrow_mut().handle_drag_start(entity_id, x, y);
                            } else {
                                println!("Mousedown on empty area ignored.");
                            }
                        }
                    } else {
                         println!("Failed to cast to MouseEvent in mousedown listener");
                    }
                } else {
                    println!("GameApp weak ref upgrade failed in mousedown listener");
                }
            });
            canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            // ★ GameApp 取得時にアップグレード必要 ★
            if let Some(rc) = game_app_weak.upgrade() {
                *rc.borrow_mut().canvas_mousedown_closure.lock().unwrap() = Some(closure);
            } else {
                 println!("Warning: Could not store mousedown closure as GameApp was dropped?");
            }
        }

        println!("Canvas listeners set up using Weak pointers.");
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
        ).map_err(|e| JsValue::from_str(&format!("Render error: {:?}", e))) // ★ 修正: エラーを文字列化して JsValue に ★
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
        // ★ 早期リターンを追加 (デバッグ用、または不要なら削除) ★
        // log(&format!("GameApp::handle_click received: ({}, {})", x, y));
        // return; // ここで一旦止めてみる

        // World のロックを取得 (エラーハンドリングを改善)
        let world = match self.world.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // パニックの連鎖を防ぐため、エラーログを出してリターンする
                error!("World mutex poisoned in handle_click: {:?}. Aborting click.", poisoned);
                return;
                // poisoned.into_inner() // ← パニックリカバリーする場合はこちら
            }
        };

        let target_element = event_handler::find_clicked_element(&world, x, y, None);
        log(&format!("Canvas clicked at ({}, {}). Target: {:?}", x, y, target_element));

        // World のロックを一時的に解放 (match 内で再度ロックが必要な場合があるため)
        drop(world);

        match target_element {
            Some(ClickTarget::Stack(stack_type)) => {
                log(&format!("Clicked on stack area: {:?}", stack_type));
                // ★★★ ここに Stock クリック処理の呼び出しを追加 ★★★
                if stack_type == StackType::Stock {
                    log("Stock area clicked! Calling handle_stock_click...");
                    // ★ self の可変参照が必要なので、match の外で呼び出すか、
                    //   handle_stock_click が &self を取るようにする。
                    //   今回は match の後に呼び出す形にする。
                } else {
                    // 他のスタックエリアがクリックされた場合の処理 (もしあれば)
                    log("Clicked on other stack area.");
                }
            }
            Some(ClickTarget::Card(entity)) => {
                // カードクリック時の処理は mousedown でドラッグ開始、dblclick で移動試行なので、
                // 通常の click では何もしないことが多い。
                // 必要ならここに処理を追加。
                log(&format!("Clicked on card entity: {:?}. (No action for single click)", entity));
            }
            None => {
                // 何もない場所がクリックされた場合の処理
                log("Clicked on empty area.");
            }
        }

        // ★ Stock がクリックされた場合、ここで handle_stock_click を呼び出す ★
        if let Some(ClickTarget::Stack(StackType::Stock)) = target_element {
            self.handle_stock_click();
        }
    }

    /// 山札 (Stock) がクリックされたときの処理
    fn handle_stock_click(&mut self) {
        log("handle_stock_click called.");
        let mut world = match self.world.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!("World mutex poisoned in handle_stock_click: {:?}. Aborting.", poisoned);
                return;
            }
        };
        // NetworkManager は現時点では使わないが、将来のために残しておく
        // let network_manager = self.network_manager.clone();

        // --- 1. Stock の一番上のカードを探す ---
        // position_in_stack が最も大きいものを探す
        let top_stock_card_entity = world
            .get_all_entities_with_component::<StackInfo>()
            .into_iter()
            .filter(|e| world.get_component::<StackInfo>(*e).map_or(false, |si| si.stack_type == StackType::Stock))
            .max_by_key(|e| world.get_component::<StackInfo>(*e).unwrap().position_in_stack); // unwrap はフィルタリング後なので安全なはず

        if let Some(top_card_entity) = top_stock_card_entity {
            // --- 2. Stock にカードがある場合: Waste に移動 ---
            log(&format!("Found top card in Stock: {:?}", top_card_entity));

            // Waste の現在のカード数を取得 (次の position_in_stack のため)
            let waste_card_count = world
                .get_all_entities_with_component::<StackInfo>()
                .into_iter()
                .filter(|e| world.get_component::<StackInfo>(*e).map_or(false, |si| si.stack_type == StackType::Waste))
                .count();
            let next_waste_pos = waste_card_count as u8;

            // ★ 修正: 位置計算を borrow_mut の前に移動 ★
            let new_pos = layout_calculator::calculate_card_position(StackType::Waste, next_waste_pos, &*world);

            // カードのコンポーネントを更新
            let mut card_moved = false;
            if let Some(stack_info) = world.get_component_mut::<StackInfo>(top_card_entity) {
                stack_info.stack_type = StackType::Waste;
                stack_info.position_in_stack = next_waste_pos;
                card_moved = true; // StackInfo 変更成功
            }
            if let Some(card) = world.get_component_mut::<Card>(top_card_entity) {
                card.is_face_up = true;
            }
            if let Some(position) = world.get_component_mut::<Position>(top_card_entity) {
                // ★ 修正: 事前に計算した位置を使用 ★
                // ★ 修正: calculate_card_position の結果を一旦変数に入れる ★ // ← このコメントは古くなったので削除
                // let new_pos = layout_calculator::calculate_card_position(StackType::Waste, next_waste_pos, &*world); // ← 移動済み
                *position = new_pos;
            }

            if card_moved {
                log(&format!("Moved card {:?} from Stock to Waste (pos: {})", top_card_entity, next_waste_pos));
                // ★ サーバーに通知 (必要なら) ★
                // let mut nm = network_manager.lock().unwrap();
                // nm.send_message(ClientMessage::DrawFromStock); // 例
            }

        } else {
            // --- 3. Stock が空の場合: Waste から Stock に戻す ---
            log("Stock is empty. Checking Waste...");

            // Waste にあるカードの Entity を収集
            let waste_cards: Vec<Entity> = world
                .get_all_entities_with_component::<StackInfo>()
                .into_iter()
                .filter(|e| world.get_component::<StackInfo>(*e).map_or(false, |si| si.stack_type == StackType::Waste))
                .collect();

            if !waste_cards.is_empty() {
                log(&format!("Found {} cards in Waste. Moving them back to Stock.", waste_cards.len()));
                let mut cards_reset = 0;
                // Waste のカードを position_in_stack の昇順でソートして Vec<(Entity, u8)> を作成
                let mut sorted_waste_cards_with_pos: Vec<(Entity, u8)> = waste_cards
                    .iter()
                    .map(|e| (*e, world.get_component::<StackInfo>(*e).unwrap().position_in_stack))
                    .collect();
                sorted_waste_cards_with_pos.sort_by_key(|&(_, pos)| pos);

                // ★ 修正: Vec を作ってイテレートする (borrow checker のため) ★
                let entities_to_update: Vec<(Entity, u8)> = sorted_waste_cards_with_pos
                    .iter()
                    .enumerate()
                    .map(|(index, (entity, _))| (*entity, index as u8))
                    .collect();

                // ★ 修正: 位置計算と更新を分離 ★
                //    事前に新しい位置を計算して Vec に格納
                let mut new_positions = Vec::new();
                for (entity, new_stock_pos) in &entities_to_update {
                     let new_pos = layout_calculator::calculate_card_position(StackType::Stock, *new_stock_pos, &*world);
                     new_positions.push((*entity, new_pos)); // タプル (Entity, Position) を格納
                }

                // ★ 修正: コンポーネント更新ループ ★
                for (index, (entity, _original_waste_pos)) in sorted_waste_cards_with_pos.iter().enumerate() { // sorted_waste_cards_with_pos を再度使うか、entities_to_update を使うか注意
                    let new_stock_pos = index as u8; // entities_to_update を使わない場合は index を使う
                    if let Some(stack_info) = world.get_component_mut::<StackInfo>(*entity) {
                        stack_info.stack_type = StackType::Stock;
                        stack_info.position_in_stack = new_stock_pos;
                    }
                    if let Some(card) = world.get_component_mut::<Card>(*entity) {
                        card.is_face_up = false; // Stock に戻すときは裏向き
                    }
                    // 事前に計算した位置を探して適用
                    if let Some((_, new_pos)) = new_positions.iter().find(|(e, _)| *e == *entity) {
                         if let Some(position) = world.get_component_mut::<Position>(*entity) {
                            // ★ 修正: calculate_card_position の結果を一旦変数に入れる ★ // ← このコメントは古くなったので削除
                            // let new_pos = layout_calculator::calculate_card_position(StackType::Stock, new_stock_pos, &*world); // ← 移動済み
                            *position = new_pos.clone(); // ★ 修正: clone() を呼ぶ ★
                        }
                    }
                    cards_reset += 1;
                }
                log(&format!("Reset {} cards from Waste to Stock.", cards_reset));
                 // ★ サーバーに通知 (必要なら) ★
                // let mut nm = network_manager.lock().unwrap();
                // nm.send_message(ClientMessage::ResetStockFromWaste); // 例
            } else {
                log("Waste is also empty. Nothing to do.");
            }
        }
        // World のロックはこのスコープを抜けるときに解放される
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

        match event_handler::find_topmost_clicked_card(&world, x, y, None) {
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