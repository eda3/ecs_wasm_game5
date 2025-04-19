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
        // ★修正: app::init_handler の関数を呼び出す！★
        app::init_handler::deal_initial_cards(
            &self.world,
            &self.network_manager,
            &self.deal_system
        );
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
    pub fn render_game_rust(&self) -> Result<(), JsValue> {
        // ★修正: app::renderer の関数を呼び出す！★
        app::renderer::render_game_rust(
            &self.world,
            &self.canvas,
            &self.context
        )
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