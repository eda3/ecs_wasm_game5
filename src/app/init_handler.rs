// src/app/init_handler.rs
//! GameApp の初期化や初期状態送信に関するロジック。

use std::sync::{Arc, Mutex};
use crate::world::World;
use crate::network::NetworkManager;
use crate::systems::deal_system::DealInitialCardsSystem;
use crate::protocol::{GameStateData, ClientMessage, CardData, PositionData};
use crate::components::{Card, StackInfo, Position, StackType};
use crate::{log, error}; // log と error マクロを使う
use crate::app::network_handler; // send_serialized_message を使うために必要

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