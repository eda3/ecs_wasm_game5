// src/app/network_handler.rs
//! GameApp のネットワーク関連（接続、送受信、メッセージ処理）のロジック。

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::network::NetworkManager;
use crate::protocol::{ClientMessage, ServerMessage};
use crate::entity::Entity;
use crate::components::stack::StackType;
use crate::world::World; // process_received_messages が state_handler を呼ぶために必要
use crate::protocol::PlayerId;
use crate::app::state_handler; // apply_game_state を呼び出すために必要
use crate::{log, error}; // log と error マクロを使う
use serde_json;

// --- WebSocket 接続 --- 

pub fn connect(network_manager_arc: &Arc<Mutex<NetworkManager>>) {
    log("App::Network: connect() called.");
    match network_manager_arc.lock() {
        Ok(mut nm) => nm.connect(),
        Err(e) => log(&format!("App::Network: Failed to lock NetworkManager for connect: {:?}", e)),
    }
}

// --- メッセージ送信関連 --- 

/// 内部ヘルパー: ClientMessage を JSON にシリアライズして送信する。
/// GameApp::send_message のロジックを移動。
pub(crate) fn send_serialized_message(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    message: ClientMessage
) -> Result<(), String> {
    log(&format!("App::Network: Preparing to send message: {:?}", message));
    match serde_json::to_string(&message) {
        Ok(json_message) => {
            // network_manager のロックは send_message の中で行われる想定
            let nm = network_manager_arc.lock().expect("Failed to lock NetworkManager for sending");
            nm.send_message(&json_message).map_err(|e| e.to_string())
        }
        Err(e) => {
            let error_msg = format!("Failed to serialize ClientMessage: {}", e);
            error(&error_msg);
            Err(error_msg)
        }
    }
}

/// ゲーム参加メッセージを送信する。
/// GameApp::send_join_game のロジックを移動。
pub fn send_join_game(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    player_name: String
) {
    log(&format!("App::Network: send_join_game called with name: {}", player_name));
    let message = ClientMessage::JoinGame { player_name };
    if let Err(e) = send_serialized_message(network_manager_arc, message) {
        error(&format!("App::Network: Failed to send JoinGame message: {}", e));
    }
}

/// カード移動メッセージを送信する。
/// GameApp::send_make_move のロジックを移動。
pub fn send_make_move(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    moved_entity_id: usize,
    target_stack_json: String
) {
    log(&format!("App::Network: send_make_move called with entity: {}, target: {}", moved_entity_id, target_stack_json));
    let moved_entity = Entity(moved_entity_id);

    match serde_json::from_str::<StackType>(&target_stack_json) {
        Ok(target_stack) => {
            // components::StackType から protocol::StackType へ変換が必要
            let protocol_target_stack: crate::protocol::StackType = target_stack.into(); // From 実装前提
            let message = ClientMessage::MakeMove { moved_entity, target_stack: protocol_target_stack };
            if let Err(e) = send_serialized_message(network_manager_arc, message) {
                error(&format!("App::Network: Failed to send MakeMove message: {}", e));
            }
        }
        Err(e) => {
            error(&format!("App::Network: Failed to deserialize target_stack JSON: {}. Input: {}", e, target_stack_json));
        }
    }
}

// --- 受信メッセージ処理 --- 

/// 受信メッセージを処理する。
/// GameApp::process_received_messages のロジックを移動。
/// 状態変更があったかどうかを bool で返す。
pub fn process_received_messages(
    message_queue_arc: &Arc<Mutex<VecDeque<ServerMessage>>>,
    my_player_id_arc: &Arc<Mutex<Option<PlayerId>>>,
    world_arc: &Arc<Mutex<World>> // apply_game_state を呼ぶために必要
) -> bool {
    let mut state_changed = false;

    let messages_to_process: Vec<ServerMessage> = {
        let mut queue = message_queue_arc.lock().expect("Failed to lock message queue");
        queue.drain(..).collect()
    };

    if !messages_to_process.is_empty() {
        log(&format!("App::Network: Processing {} received messages...", messages_to_process.len()));
    }

    for message in messages_to_process {
        log(&format!("  Processing: {:?}", message));
        match message {
            ServerMessage::GameJoined { your_player_id, initial_game_state } => {
                *my_player_id_arc.lock().expect("Failed to lock my_player_id") = Some(your_player_id);
                log(&format!("App::Network: Game joined! My Player ID: {}", your_player_id));
                // state_handler の関数を呼び出す
                if state_handler::apply_game_state(world_arc, initial_game_state) {
                    state_changed = true;
                }
            }
            ServerMessage::GameStateUpdate { current_game_state } => {
                log("App::Network: Received GameStateUpdate.");
                // state_handler の関数を呼び出す
                if state_handler::apply_game_state(world_arc, current_game_state) {
                    state_changed = true;
                }
            }
            ServerMessage::MoveRejected { reason } => {
                log(&format!("App::Network: Move rejected by server: {}", reason));
            }
            ServerMessage::PlayerJoined { player_id, player_name } => {
                log(&format!("App::Network: Player {} ({}) joined.", player_name, player_id));
                // apply_game_state でプレイヤーは更新されるので、ここでは state_changed を true にしない
            }
            ServerMessage::PlayerLeft { player_id } => {
                log(&format!("App::Network: Player {} left.", player_id));
                // apply_game_state でプレイヤーはクリアされる？ TODO: PlayerLeft 専用の処理が必要かも
            }
            ServerMessage::Pong => {
                log("App::Network: Received Pong from server.");
            }
            ServerMessage::Error { message } => {
                error(&format!("App::Network: Received error from server: {}", message));
            }
        }
    }
    state_changed
} 