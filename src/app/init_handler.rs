// src/app/init_handler.rs
//! GameApp の初期化や初期状態送信に関するロジック。

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
// use crate::ecs::entity::Entity; // このファイル内では直接使われていない
use crate::ecs::world::World;
use crate::network::{NetworkManager, ConnectionStatus};
use crate::systems::deal_system::DealInitialCardsSystem;
use crate::protocol::{GameStateData, ClientMessage, CardData, PositionData, ServerMessage};
use crate::components::{self, Card, StackInfo, Position, /*StackType*/}; // StackType は直接使われていない
use crate::{log, error}; // log と error マクロを使う
use crate::app::network_handler; // send_serialized_message を使うために必要
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};

// --- 初期化ヘルパー関数 (GameApp::new から移動) ---

/// World の初期化とコンポーネント登録を行う。
pub(crate) fn initialize_world() -> Arc<Mutex<World>> {
    log("App::Init: Initializing world...");
    let mut world = World::new();
    // コンポーネント登録
    world.register_component::<components::card::Card>();
    world.register_component::<components::position::Position>();
    world.register_component::<components::stack::StackInfo>();
    world.register_component::<components::game_state::GameState>();
    world.register_component::<components::player::Player>();
    // ★ DraggingInfo も登録 ★
    world.register_component::<components::dragging_info::DraggingInfo>();
    Arc::new(Mutex::new(world))
}

/// NetworkManager の初期化を行う。
pub(crate) fn initialize_network(message_queue_arc: Arc<Mutex<VecDeque<ServerMessage>>>) -> Arc<Mutex<NetworkManager>> {
    log("App::Init: Initializing network...");
    let server_url = format!("ws://{}:{}", "localhost", 8101);
    let status_arc = Arc::new(Mutex::new(ConnectionStatus::Disconnected)); // Status Arc はここで作る
    let network_manager = NetworkManager::new(
        server_url,
        Arc::clone(&status_arc),
        message_queue_arc, // 引数で受け取る
    );
    Arc::new(Mutex::new(network_manager))
}

/// Canvas と 2D Context の取得を行う。
pub(crate) fn initialize_canvas() -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    log("App::Init: Initializing canvas...");
    let window = window().ok_or_else(|| JsValue::from_str("Failed to get window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("Failed to get document"))?;
    let canvas = document
        .get_element_by_id("game-canvas")
        .ok_or_else(|| JsValue::from_str("#game-canvas element not found"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("Element is not an HtmlCanvasElement"))?;

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Option for 2d context is None"))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| JsValue::from_str("Context is not CanvasRenderingContext2d"))?;

    log("Canvas and 2D context obtained successfully.");
    Ok((canvas, context))
}

// --- ヘルパー関数 (GameApp::get_initial_state_data から移動) ---

/// 現在の World の状態から GameStateData を作成する。
pub(crate) fn get_initial_state_data(world: &World) -> GameStateData {
    log("App::Init: Generating initial game state data...");
    let players = Vec::new(); // TODO: プレイヤー情報も取得・含めるべき？

    let card_entities = world.get_all_entities_with_component::<Card>();
    let mut cards = Vec::with_capacity(card_entities.len());

    for &entity in &card_entities {
        let card = world.get_component::<Card>(entity)
            .expect(&format!("Card component not found for entity {:?}", entity));
        let stack_info = world.get_component::<StackInfo>(entity)
            .expect(&format!("StackInfo component not found for entity {:?}", entity));
        let position = world.get_component::<Position>(entity)
            .expect(&format!("Position component not found for entity {:?}", entity));

        cards.push(CardData {
            entity,
            suit: card.suit.into(),
            rank: card.rank.into(),
            is_face_up: card.is_face_up,
            stack_type: stack_info.stack_type.into(), // From トレイト実装前提
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

// --- ヘルパー関数 (GameApp::send_initial_state から移動) ---

/// 初期ゲーム状態をサーバーに送信する。
pub(crate) fn send_initial_state(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    initial_state_data: GameStateData
) {
    log("App::Init: send_initial_state called.");
    let message = ClientMessage::ProvideInitialState { initial_state: initial_state_data };
    log("  Sending ProvideInitialState message...");
    if let Err(e) = network_handler::send_serialized_message(network_manager_arc, message) {
        error(&format!("App::Init: Failed to send ProvideInitialState message: {}", e));
    } else {
        log("  ProvideInitialState message sent successfully.");
    }
}

// --- 公開関数 (GameApp から呼び出される) ---

/// JSから初期カード配置を実行するためのロジック。
/// GameApp::deal_initial_cards のロジックを移動。
pub fn deal_initial_cards(
    world_arc: &Arc<Mutex<World>>,
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    deal_system: &DealInitialCardsSystem // DealSystem への参照を受け取る
) {
    log("App::Init: deal_initial_cards() called.");

    // ステップ1: 書き込みロックを取得して DealSystem を実行
    {
        log("  Acquiring mutable lock for DealInitialCardsSystem...");
        let mut mutable_world_guard = match world_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log(&format!("App::Init: World mutex was poisoned! Attempting recovery. Error: {:?}", poisoned));
                poisoned.into_inner()
            }
        };
        log("  Executing DealInitialCardsSystem...");
        deal_system.execute(&mut mutable_world_guard);
        log("  DealInitialCardsSystem executed successfully.");
        log("  Released mutable lock.");
    } // <-- 書き込みロック解放

    // ステップ2: 読み取り専用ロックを取得して初期状態データを取得
    let initial_state_data = {
        log("  Acquiring immutable lock for get_initial_state_data...");
        let immutable_world_guard = match world_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log(&format!("App::Init: World mutex was poisoned (read lock)! Attempting recovery. Error: {:?}", poisoned));
                poisoned.into_inner()
            }
        };
        log("  Getting initial state data...");
        let data = get_initial_state_data(&immutable_world_guard); // このモジュール内の関数を呼ぶ
        log("  Initial state data prepared.");
        log("  Released immutable lock.");
        data
    }; // <-- 読み取りロック解放

    // ステップ3: 状態データを送信
    send_initial_state(network_manager_arc, initial_state_data); // このモジュール内の関数を呼ぶ
} 