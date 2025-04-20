// src/app/network_receiver.rs
//! Handles processing messages received from the WebSocket server.

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::protocol::{ServerMessage, PlayerId};
use crate::ecs::entity::Entity;
use crate::ecs::world::World;
use crate::app::state_handler; 
use crate::log;
use log::error;

/// `process_received_messages` が返す結果の種類を表す enum だよ！
#[derive(Debug, Clone)]
pub enum ProcessedMessageResult {
    Nothing,
    StateChanged,
    MoveRejected { 
        entity_id: Entity,
        reason: String,
    },
}

/// 受信メッセージキューを処理して、発生した重要イベントのリストを返すよ！
pub fn process_received_messages(
    message_queue_arc: &Arc<Mutex<VecDeque<ServerMessage>>>,
    my_player_id_arc: &Arc<Mutex<Option<PlayerId>>>,
    world_arc: &Arc<Mutex<World>>
) -> Vec<ProcessedMessageResult> { 
    let mut results: Vec<ProcessedMessageResult> = Vec::new();

    let messages_to_process: Vec<ServerMessage> = {
        let mut queue = message_queue_arc.lock().expect("Failed to lock message queue");
        queue.drain(..).collect() 
    };

    if messages_to_process.is_empty() {
        return results;
    }

    log(&format!("App::NetworkReceiver: Processing {} received messages...", messages_to_process.len()));

    for message in messages_to_process {
        log(&format!("  Processing: {:?}", message));
        match message {
            ServerMessage::GameJoined { your_player_id, initial_game_state } => {
                *my_player_id_arc.lock().expect("Failed to lock my_player_id") = Some(your_player_id);
                log(&format!("App::NetworkReceiver: Game joined! My Player ID: {}", your_player_id));
                if state_handler::apply_game_state(world_arc, initial_game_state) {
                    results.push(ProcessedMessageResult::StateChanged);
                }
            }
            ServerMessage::GameStateUpdate { current_game_state } => {
                log("App::NetworkReceiver: Received GameStateUpdate.");
                if state_handler::apply_game_state(world_arc, current_game_state) {
                    results.push(ProcessedMessageResult::StateChanged);
                }
            }
            ServerMessage::MoveRejected { entity_id, reason } => {
                log(&format!("App::NetworkReceiver: Move rejected by server for entity {:?}: {}", entity_id, reason));
                results.push(ProcessedMessageResult::MoveRejected { entity_id, reason });
            }
            ServerMessage::PlayerJoined { player_id, player_name } => {
                log(&format!("App::NetworkReceiver: Player {} ({}) joined.", player_name, player_id));
                // StateChanged will likely happen via GameStateUpdate
            }
            ServerMessage::PlayerLeft { player_id } => {
                log(&format!("App::NetworkReceiver: Player {} left.", player_id));
                 // StateChanged will likely happen via GameStateUpdate
            }
            ServerMessage::Pong => {
                log("App::NetworkReceiver: Received Pong from server.");
            }
            ServerMessage::Error { message } => {
                error!("App::NetworkReceiver: Received error from server: {}", message);
            }
        }
    }
    results
} 