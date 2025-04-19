// src/lib.rs

// WASM と JavaScript を繋ぐための基本！
use wasm_bindgen::prelude::*;
// ★復活！ JsCast トレイトを使う！★
use wasm_bindgen::JsCast;

// ★修正: 未使用の型をごっそり削除！ Event, window, HtmlCanvasElement, CanvasRenderingContext2d は残す★
use web_sys::{window, Event, HtmlCanvasElement, CanvasRenderingContext2d};

// 標準ライブラリから、スレッドセーフな共有ポインタとミューテックスを使うよ。
// 非同期のコールバック関数からでも安全にデータを共有・変更するために必要！
use std::sync::{Arc, Mutex};
// メッセージキュー（受信メッセージを一時的に溜めておく場所）のために VecDeque を使うよ。
use std::collections::VecDeque;

// 自分で作ったモジュールたち！ これでコードを整理してるんだ。
pub mod entity;
pub mod component;
pub mod world; // この world モジュールは自作ECSのコアになるかも？
pub mod system;
pub mod components; // components モジュールを宣言
pub mod systems;
pub mod network; // network モジュールを宣言
pub mod protocol; // protocol モジュールを宣言
pub mod rules; // ★追加: 新しい rules モジュールを宣言！
pub mod logic; // ← これを追加！
pub mod app; // ★追加: 新しい app モジュールを宣言

// 各モジュールから必要な型をインポート！
// use crate::world::World; // <-- これも不要 (自作Worldを使う想定)
// use hecs::World; // <-- これを削除！
use crate::network::NetworkManager; // NetworkManager をインポート (ConnectionStatusは不要なので削除)
use crate::protocol::{ClientMessage, ServerMessage, GameStateData, CardData, PositionData, PlayerId};
use crate::components::stack::StackType; // components::stack から StackType を直接インポート！
use crate::entity::Entity; // send_make_move で使う Entity も use しておく！ (自作Entityを使う)
use serde_json; // serde_json を使う
use crate::network::ConnectionStatus; // ↓↓↓ ConnectionStatus を再度 use する！
// systems モジュールと、その中の DealInitialCardsSystem を使う宣言！
use wasm_bindgen::closure::Closure; // ★追加: イベント関連の型と Closure を use★
use crate::components::dragging_info::DraggingInfo; // ★変更: 新しいパスからインポート！
use crate::world::World; // <<< これを追加！
use crate::systems::deal_system::DealInitialCardsSystem;

// components/ 以下の主要なコンポーネントを use 宣言！
// (ここで use したものは、このファイル内では直接型名で参照できる！)
use crate::components::{ 
    card::Card, // Import specifics from card module
    position::Position,
    player::Player, // Import Player from components
    stack::{StackInfo}, // Import StackInfo/StackType from components
};

use crate::logic::auto_move::find_automatic_foundation_move;

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
    dragging_state: Arc<Mutex<Option<DraggingInfo>>>,
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
        log("GameApp: Initializing for Canvas rendering...");
        let mut world = World::new();
        // コンポーネント登録 (Player も忘れずに)
        world.register_component::<components::card::Card>();
        world.register_component::<components::position::Position>();
        world.register_component::<components::stack::StackInfo>();
        world.register_component::<components::game_state::GameState>();
        world.register_component::<components::player::Player>();

        let world_arc = Arc::new(Mutex::new(world));
        let message_queue_arc = Arc::new(Mutex::new(VecDeque::new()));
        let my_player_id_arc = Arc::new(Mutex::new(None));

        let server_url = format!("ws://{}:{}", "localhost", 8101);
        let status_arc = Arc::new(Mutex::new(ConnectionStatus::Disconnected));

        let network_manager = NetworkManager::new(
            server_url,
            Arc::clone(&status_arc),
            Arc::clone(&message_queue_arc),
        );
        let network_manager_arc = Arc::new(Mutex::new(network_manager));

        // DealInitialCardsSystem のインスタンスも作る！ default() で作れるようにしておいてよかった！ ✨
        let deal_system = DealInitialCardsSystem::default();

        // ★ event_closures を初期化 ★
        let event_closures_arc = Arc::new(Mutex::new(Vec::new()));
        // ★追加: 新しいフィールドの初期化★
        let dragging_state_arc = Arc::new(Mutex::new(None));
        let window_mousemove_closure_arc = Arc::new(Mutex::new(None));
        let window_mouseup_closure_arc = Arc::new(Mutex::new(None));

        // ★ Canvas 要素と 2D コンテキストを取得・設定 ★
        let window = window().expect("Failed to get window");
        let document = window.document().expect("Failed to get document");
        let canvas = document
            .get_element_by_id("game-canvas") // ★ ID を "game-canvas" に変更！★
            .expect("#game-canvas element not found")
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .expect("Element is not an HtmlCanvasElement");

        let context = canvas
            .get_context("2d")
            .expect("Failed to get 2d context")
            .expect("Option for 2d context is None") // get_context は Option<Result<Object>> を返す
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| ())
            .expect("Context is not CanvasRenderingContext2d");
        
        log("Canvas and 2D context obtained successfully.");

        log("GameApp: Initialization complete.");
        Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
            deal_system, // deal_system を GameApp に追加！
            event_closures: event_closures_arc, // ★初期化したものをセット★
            dragging_state: dragging_state_arc,
            window_mousemove_closure: window_mousemove_closure_arc,
            window_mouseup_closure: window_mouseup_closure_arc,
            // ★取得した canvas と context をセット★
            canvas,
            context,
        }
    }

    // WebSocket接続
    pub fn connect(&self) {
        // ★修正: app::network_handler の関数を呼び出す！★
        app::network_handler::connect(&self.network_manager);
    }

    // ゲーム参加メッセージ送信
    #[wasm_bindgen]
    pub fn send_join_game(&self, player_name: String) {
        // ★修正: app::network_handler の関数を呼び出す！★
        app::network_handler::send_join_game(&self.network_manager, player_name);
    }

    // カード移動メッセージ送信
    #[wasm_bindgen]
    pub fn send_make_move(&self, moved_entity_id: usize, target_stack_json: String) {
        // ★修正: app::network_handler の関数を呼び出す！★
        app::network_handler::send_make_move(&self.network_manager, moved_entity_id, target_stack_json);
    }

    // 受信メッセージ処理
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) -> bool {
        // ★修正: app::network_handler の関数を呼び出す！ 必要な Arc を渡す★
        app::network_handler::process_received_messages(
            &self.message_queue,
            &self.my_player_id,
            &self.world
        )
    }

    // JSから初期カード配置を実行するためのメソッド
    #[wasm_bindgen]
    pub fn deal_initial_cards(&self) {
        log("GameApp: deal_initial_cards() called.");

        // ステップ1: 書き込み可能ロックを取得して DealSystem を実行
        { // スコープを区切ってロックの生存期間を明確にする
            log("  Acquiring mutable lock for DealInitialCardsSystem...");
            let mut mutable_world_guard = match self.world.lock() {
                 Ok(guard) => guard,
                 Err(poisoned) => {
                     log(&format!("GameApp: World mutex was poisoned! Attempting recovery. Error: {:?}", poisoned));
                     // poison エラーからデータを復旧（あるいはデフォルト値を使うなど）
                     // ここでは単純に復旧を試みる
                     poisoned.into_inner()
                 }
            };
            // let mut mutable_world_guard = self.world.lock().expect("Failed mutable lock 1");
            log("  Executing DealInitialCardsSystem...");
            self.deal_system.execute(&mut mutable_world_guard);
            log("  DealInitialCardsSystem executed successfully.");
            // スコープの終わりで mutable_world_guard が drop され、ロックが解放される！
            log("  Released mutable lock.");
        } // <-- ここで書き込みロック解放！🔓

        // ステップ2: 読み取り専用ロックを取得して初期状態データを取得
        let initial_state_data = { // スコープを区切る
            log("  Acquiring immutable lock for get_initial_state_data...");
            let immutable_world_guard = match self.world.lock() {
                 Ok(guard) => guard,
                 Err(poisoned) => {
                     log(&format!("GameApp: World mutex was poisoned (read lock)! Attempting recovery. Error: {:?}", poisoned));
                     poisoned.into_inner()
                 }
            };
            // let immutable_world_guard = self.world.lock().expect("Failed immutable lock");
            log("  Getting initial state data...");
            let data = self.get_initial_state_data(&immutable_world_guard);
            log("  Initial state data prepared.");
            // スコープの終わりで immutable_world_guard が drop され、ロックが解放される！
            log("  Released immutable lock.");
            data // スコープの結果としてデータを返す
        }; // <-- ここで読み取りロック解放！🔓

        // ステップ3: 状態データを送信 (ロックは不要)
        self.send_initial_state(initial_state_data);
    }

    /// 現在の World の状態から GameStateData を作成する
    fn get_initial_state_data(&self, world: &World) -> GameStateData {
        log("GameApp: Generating initial game state data...");
        let players = Vec::new(); // 初期状態ではプレイヤー情報は空？

        // World から全ての Card エンティティと関連コンポーネントを取得
        let card_entities = world.get_all_entities_with_component::<Card>();
        let mut cards = Vec::with_capacity(card_entities.len());

        for &entity in &card_entities {
            // 各エンティティから必要なコンポーネントを取得 (存在しない場合はエラー)
            let card = world.get_component::<Card>(entity).expect(&format!("Card component not found for entity {:?}", entity));
            let stack_info = world.get_component::<StackInfo>(entity).expect(&format!("StackInfo component not found for entity {:?}", entity));
            let position = world.get_component::<Position>(entity).expect(&format!("Position component not found for entity {:?}", entity));

            // CardData を作成して Vec に追加
            cards.push(CardData {
                entity,
                suit: card.suit.into(), // components::card::Suit -> protocol::Suit
                rank: card.rank.into(), // components::card::Rank -> protocol::Rank
                is_face_up: card.is_face_up,
                // TODO: components::stack::StackType から protocol::StackType への変換が必要
                stack_type: match stack_info.stack_type {
                    StackType::Tableau(index) => protocol::StackType::Tableau(index),
                    StackType::Foundation(index) => protocol::StackType::Foundation(index),
                    StackType::Stock => protocol::StackType::Stock,
                    StackType::Waste => protocol::StackType::Waste,
                    StackType::Hand => protocol::StackType::Hand,
                },
                // TODO: StackInfo の position_in_stack は u8 なので String に変換？
                //       protocol.rs の CardData.position_in_stack が String なら必要。
                //       u8 のまま送るなら .to_string() は不要。
                position_in_stack: stack_info.position_in_stack,
                position: PositionData {
                    x: position.x,
                    y: position.y,
                },
            });
        }

        GameStateData {
            players,
            cards,
        }
    }

    // 初期ゲーム状態をサーバーに送信するメソッド
    fn send_initial_state(&self, initial_state_data: GameStateData) {
        log("GameApp: send_initial_state called.");
        let message = ClientMessage::ProvideInitialState { initial_state: initial_state_data, };
        log(&format!("  Sending ProvideInitialState message..."));
        if let Err(e) = self.send_message(message) {
            log(&format!("GameApp: Failed to send ProvideInitialState message: {}", e));
        } else {
            log("  ProvideInitialState message sent successfully.");
        }
    }

    // WASM から World の状態を取得して JSON 文字列で返す (デバッグ・描画用)
    #[wasm_bindgen]
    pub fn get_world_state_json(&self) -> String {
        log("GameApp: get_world_state_json called.");
        let world = self.world.lock().expect("Failed to lock world for getting state");
        let card_entities = world.get_all_entities_with_component::<Card>();
        let mut cards_json_data: Vec<serde_json::Value> = Vec::with_capacity(card_entities.len());
        log(&format!("  Found {} card entities. Preparing JSON data...", card_entities.len()));
        for &entity in &card_entities {
            let card = world.get_component::<Card>(entity).expect("Card component not found");
            let stack_info = world.get_component::<StackInfo>(entity).expect("StackInfo component not found");
             // ★ Position も取得！
            let position = world.get_component::<Position>(entity).expect("Position component not found");

            // JSONに変換する際、StackTypeの各バリアントを文字列と対応するインデックス（またはNull）のタプルに変換する
            let (stack_type_str, stack_index_json) = match stack_info.stack_type {
                // Stock, Waste, Foundationはインデックスを持つタプルバリアントなので、(index)で値を取り出す
                StackType::Stock => ("Stock", serde_json::Value::Null), // Stockにはインデックス不要
                StackType::Waste => ("Waste", serde_json::Value::Null), // Wasteにもインデックス不要
                StackType::Foundation(index) => ("Foundation", serde_json::json!(index)), // indexを使用
                // Tableauもインデックスを持つタプルバリアント
                StackType::Tableau(index) => ("Tableau", serde_json::json!(index)), // 誤: crate::component::StackType::Tableau, stack_info.stack_index -> 正: StackType::Tableau(index), index
                // Handは単純なバリアント
                StackType::Hand => ("Hand", serde_json::Value::Null), // 誤: crate::component::StackType::Hand
            };
            let card_json = serde_json::json!({
                "entity_id": entity.0,
                "suit": format!("{:?}", card.suit),
                "rank": format!("{:?}", card.rank),
                "is_face_up": card.is_face_up,
                "stack_type": stack_type_str,
                "stack_index": stack_index_json,
                "order": stack_info.position_in_stack,
                // ★ Position も JSON に追加！
                "x": position.x,
                "y": position.y,
            });
            cards_json_data.push(card_json);
        }
        log("  Card data preparation complete.");
        let final_json = serde_json::json!({ "cards": cards_json_data });
        match serde_json::to_string(&final_json) {
            Ok(json_string) => { log("  Successfully serialized world state to JSON."); json_string }
            Err(e) => {
                log(&format!("Error serializing world state to JSON: {}", e));
                serde_json::json!({ "error": "Failed to serialize world state", "details": e.to_string() }).to_string()
            }
        }
    }

    // 接続状態を文字列で返す (デバッグ用)
    #[wasm_bindgen]
    pub fn get_connection_status_debug(&self) -> String {
        // 内部でロックを取るので match を使う方が丁寧かもだけど、デバッグ用なので expect で！
        let status = self.network_manager.lock().expect("Failed to lock NetworkManager for status").get_status();
        format!("{:?}", status) // Debug トレイトを使って文字列に変換
    }

    // 自分の Player ID を返す (デバッグ用)
    #[wasm_bindgen]
    pub fn get_my_player_id_debug(&self) -> Option<u32> {
        // Option<PlayerId> を Option<u32> に変換する
        self.my_player_id.lock().expect("Failed to lock my_player_id").map(|id| id)
    }

    /// カードがダブルクリックされた時の処理 (JSから呼び出される元のメソッド)
    #[wasm_bindgen]
    pub fn handle_double_click(&self, entity_id: usize) {
        log(&format!("GameApp: handle_double_click called for entity_id: {}", entity_id));
        // ★修正: app::event_handler の関数を呼び出す！★
        app::event_handler::handle_double_click_logic(
            entity_id,
            Arc::clone(&self.world),
            Arc::clone(&self.network_manager)
        );
    }

    /// Rust側で Canvas にゲーム画面を描画する関数
    #[wasm_bindgen]
    pub fn render_game_rust(&self) -> Result<(), JsValue> { // Result を返すように変更
        log("GameApp: render_game_rust() called!");

        // --- ステップ1: コンテキストと Canvas 寸法を取得 --- ★変更！★
        let context = &self.context;
        let canvas = &self.canvas;
        let canvas_width = canvas.width() as f64; // u32 から f64 へキャスト
        let canvas_height = canvas.height() as f64;

        // --- ステップ2: Canvas をクリア --- ★変更！★
        context.clear_rect(0.0, 0.0, canvas_width, canvas_height);
        // log(&format!("  Canvas cleared ({}x{})."), canvas_width, canvas_height);

        // --- ステップ3: World からカード情報を取得 & ★ソート！★ ---
        let world = self.world.lock().map_err(|e| JsValue::from_str(&format!("Failed to lock world mutex: {}", e)))?;

        // --- カード要素の取得とソート ---
        // ↓↓↓ E0599 修正: world.iter() ではなく get_all_entities_with_component を使う！
        let card_entities = world.get_all_entities_with_component::<Card>();
        let mut cards_to_render: Vec<(Entity, &Position, &Card, Option<DraggingInfo>, Option<&StackInfo>)> = Vec::with_capacity(card_entities.len());

        for &entity in &card_entities {
            // ループ内で各コンポーネントを取得
            if let (Some(pos), Some(card)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Card>(entity)
            ) {
                // DraggingInfo と StackInfo は Option で取得
                let dragging_info = world.get_component::<DraggingInfo>(entity).cloned(); // cloned() で Option<DraggingInfo> に
                let stack_info = world.get_component::<StackInfo>(entity); // &StackInfo の Option

                cards_to_render.push((entity, pos, card, dragging_info, stack_info));
            } else {
                // Card または Position が見つからない場合はスキップ (またはエラーログ)
                log(&format!("Warning: Skipping entity {:?} in render_game_rust because Card or Position component is missing.", entity));
            }
        }
        // ↑↑↑ E0599 修正ここまで

        // Sort cards by stack and position within the stack, or original position if dragging
        cards_to_render.sort_by(|a, b| {
            // ★ 修正: `crate::component::` を削除 (DraggingInfoはもともとOK) ★
            let (_, _, _, dragging_info_a, stack_info_a_opt): &(Entity, &Position, &Card, Option<DraggingInfo>, Option<&StackInfo>) = a;
            // ★ 修正: `crate::component::` を削除 (DraggingInfoはもともとOK) ★
            let (_, _, _, dragging_info_b, stack_info_b_opt): &(Entity, &Position, &Card, Option<DraggingInfo>, Option<&StackInfo>) = b;

            // Use original stack order if dragging, otherwise current stack order
            let order_a = dragging_info_a
                .as_ref()
                // ★ 修正: `crate::component::` を削除 (DraggingInfoはもともとOK) ★
                .map(|di: &DraggingInfo| di.original_position_in_stack)
                // ★ 修正: `crate::component::` を削除 ★
                .or_else(|| stack_info_a_opt.map(|si: &StackInfo| si.position_in_stack as usize)) // u8 を usize にキャスト
                .unwrap_or(0); // Default order if no stack info

            let order_b = dragging_info_b
                .as_ref()
                // ★ 修正: `crate::component::` を削除 (DraggingInfoはもともとOK) ★
                .map(|di: &DraggingInfo| di.original_position_in_stack)
                // ★ 修正: `crate::component::` を削除 ★
                .or_else(|| stack_info_b_opt.map(|si: &StackInfo| si.position_in_stack as usize)) // u8 を usize にキャスト
                .unwrap_or(0); // Default order if no stack info

            order_a.cmp(&order_b)
        });

        // --- DOM操作 (未実装) ---
        // ... DOM操作のコード ...

        log(&format!("Sorted card render data ({} entities): {:?}", cards_to_render.len(), cards_to_render));

        Ok(())
    }
}

// GameApp が不要になった時に WebSocket 接続を閉じる処理 (Drop トレイト)
// JS側でインスタンスがGCされた時などに呼ばれる…はず！
impl Drop for GameApp {
    fn drop(&mut self) {
        log("GameApp: Dropping GameApp instance. Disconnecting WebSocket...");
        // ロックを取得して disconnect を呼ぶ
        match self.network_manager.lock() {
            Ok(mut nm) => nm.disconnect(),
            Err(e) => log(&format!("GameApp: Failed to lock NetworkManager for disconnect: {:?}", e)),
        }
    }
}

// ... 関数型・ベストプラクティスコメント、次のステップコメント ...