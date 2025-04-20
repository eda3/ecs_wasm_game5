// src/app/network_sender.rs
//! Handles sending messages to the WebSocket server.

use std::sync::{Arc, Mutex};
use crate::network::NetworkManager;
use crate::protocol::{ClientMessage, StackType as ProtocolStackType};
use crate::ecs::entity::Entity;
use crate::components::stack::StackType as ComponentStackType;
use crate::log;
use log::error;
use serde_json;

/// 内部ヘルパー: ClientMessage を JSON にシリアライズして送信する。
pub fn send_serialized_message(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    message: ClientMessage
) -> Result<(), String> {
    log(&format!("App::NetworkSender: Preparing to send message: {:?}", message));
    match serde_json::to_string(&message) {
        Ok(json_message) => {
            let nm = network_manager_arc.lock().expect("Failed to lock NetworkManager for sending");
            nm.send_message(&json_message).map_err(|e| e.to_string())
        }
        Err(e) => {
            let error_msg = format!("Failed to serialize ClientMessage: {}", e);
            error!("App::NetworkSender: {}", error_msg);
            Err(error_msg)
        }
    }
}

/// ゲーム参加メッセージを送信する。
pub fn send_join_game(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    player_name: String
) {
    log(&format!("App::NetworkSender: send_join_game called with name: {}", player_name));
    let message = ClientMessage::JoinGame { player_name };
    if let Err(e) = send_serialized_message(network_manager_arc, message) {
        error!("App::NetworkSender: Failed to send JoinGame message: {}", e);
    }
}

/// カード移動メッセージを送信する。
pub fn send_make_move(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    moved_entity_id: usize,
    target_stack_json: String // Keep accepting JSON for now
) {
    log(&format!("App::NetworkSender: send_make_move called with entity: {}, target: {}", moved_entity_id, target_stack_json));
    let moved_entity = Entity(moved_entity_id);

    // Deserialize the JSON string into the component's StackType first
    match serde_json::from_str::<ComponentStackType>(&target_stack_json) {
        Ok(target_stack_component) => {
            // Convert component's StackType to protocol's StackType
            let target_stack_proto: ProtocolStackType = target_stack_component.into();
            let message = ClientMessage::MakeMove { moved_entity, target_stack: target_stack_proto };
            if let Err(e) = send_serialized_message(network_manager_arc, message) {
                error!("App::NetworkSender: Failed to send MakeMove message: {}", e);
            }
        }
        Err(e) => {
            error!("App::NetworkSender: Failed to deserialize target_stack JSON: {}. Input: {}", e, target_stack_json);
        }
    }
} 