// src/app/network_handler.rs
//! GameApp のネットワーク関連（接続、送受信、メッセージ処理）のロジック。

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::network::NetworkManager;
use crate::protocol::{ClientMessage, ServerMessage, PlayerId, GameStateData};
use crate::ecs::entity::Entity;
use crate::components::stack::StackType;
use crate::ecs::world::World; // process_received_messages が state_handler を呼ぶために必要
use crate::app::state_handler; // apply_game_state を呼び出すために必要
use crate::{log, error}; // log と error マクロを使う
use serde_json;

// --- ★★★ 新しい enum: メッセージ処理の結果を詳細に伝えるための型 ★★★ ---
/// `process_received_messages` が返す結果の種類を表す enum だよ！
/// これで、JS側がどんな重要なイベントが起きたかを知ることができるんだ ✨
#[derive(Debug, Clone)] // JS には渡さないけど、デバッグ用に Debug/Clone はつけとく
pub enum ProcessedMessageResult {
    /// 特に何も重要なイベントは発生しなかったよ。
    Nothing,
    /// `GameStateUpdate` などで World の状態が更新されたよ。
    /// (JS側で再描画が必要になるかも？)
    StateChanged,
    /// サーバーからカード移動が拒否されたよ！🙅‍♀️
    MoveRejected { 
        /// 拒否されたカードのエンティティID。
        entity_id: Entity,
        /// 拒否された理由。
        reason: String,
    },
    // TODO: 必要なら他のイベント (例: PlayerJoined, PlayerLeft, GameWon など) も追加できるよ！
}

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

/// 受信メッセージキューを処理して、発生した重要イベントのリストを返すよ！
/// GameApp::process_received_messages のロジックを移動。
/// ★戻り値を `bool` から `Vec<ProcessedMessageResult>` に変更！★
pub fn process_received_messages(
    message_queue_arc: &Arc<Mutex<VecDeque<ServerMessage>>>,
    my_player_id_arc: &Arc<Mutex<Option<PlayerId>>>,
    world_arc: &Arc<Mutex<World>> // apply_game_state を呼ぶために必要
) -> Vec<ProcessedMessageResult> { // ★戻り値の型を変更！★
    // 処理結果を格納するための空の Vec を用意するよ！
    let mut results: Vec<ProcessedMessageResult> = Vec::new();

    // メッセージキューから処理すべきメッセージを全部取り出すよ。
    // キューのロックはここだけで済むように、スコープを区切る。
    let messages_to_process: Vec<ServerMessage> = {
        let mut queue = message_queue_arc.lock().expect("Failed to lock message queue");
        queue.drain(..).collect() // drain でキューを空にして、要素を Vec に集める
    };

    // 処理するメッセージがなければ、すぐに空の results を返す。
    if messages_to_process.is_empty() {
        return results;
    }

    log(&format!("App::Network: Processing {} received messages...", messages_to_process.len()));

    // 取り出したメッセージを1つずつループで処理していくよ！
    for message in messages_to_process {
        log(&format!("  Processing: {:?}", message));
        // message の種類に応じて match で処理を分岐！
        match message {
            ServerMessage::GameJoined { your_player_id, initial_game_state } => {
                // 自分のプレイヤーIDを保存！
                *my_player_id_arc.lock().expect("Failed to lock my_player_id") = Some(your_player_id);
                log(&format!("App::Network: Game joined! My Player ID: {}", your_player_id));
                // 受け取った初期ゲーム状態で World を更新！
                // state_handler の apply_game_state を呼び出す。
                if state_handler::apply_game_state(world_arc, initial_game_state) {
                    // 状態が変わったら、結果リストに StateChanged を追加！
                    results.push(ProcessedMessageResult::StateChanged);
                }
            }
            ServerMessage::GameStateUpdate { current_game_state } => {
                log("App::Network: Received GameStateUpdate.");
                // 受け取ったゲーム状態で World を更新！
                if state_handler::apply_game_state(world_arc, current_game_state) {
                    // 状態が変わったら、結果リストに StateChanged を追加！
                    results.push(ProcessedMessageResult::StateChanged);
                }
            }
            // ★★★ MoveRejected の処理を変更！ ★★★
            ServerMessage::MoveRejected { entity_id, reason } => {
                // 移動が拒否されたことをログに出力。
                log(&format!("App::Network: Move rejected by server for entity {:?}: {}", entity_id, reason));
                // ★結果リストに MoveRejected イベントを追加！★
                //   これで呼び出し元 (最終的には JS) が拒否されたことを知れる！
                results.push(ProcessedMessageResult::MoveRejected { entity_id, reason });
            }
            ServerMessage::PlayerJoined { player_id, player_name } => {
                log(&format!("App::Network: Player {} ({}) joined.", player_name, player_id));
                // PlayerJoined イベントを results に追加しても良いけど、
                // GameStateUpdate でプレイヤーリストが更新されるはずなので、
                // ここでは特に何もしなくてもいいかも？ (必要なら追加！)
                // results.push(ProcessedMessageResult::PlayerJoined { player_id, player_name });
            }
            ServerMessage::PlayerLeft { player_id } => {
                log(&format!("App::Network: Player {} left.", player_id));
                // PlayerLeft も同様。
                // results.push(ProcessedMessageResult::PlayerLeft { player_id });
            }
            ServerMessage::Pong => {
                log("App::Network: Received Pong from server.");
                // Pong は特に重要なイベントじゃないので results には追加しない。
            }
            ServerMessage::Error { message } => {
                // サーバーからのエラーメッセージはコンソールに出力。
                error(&format!("App::Network: Received error from server: {}", message));
                // results に Error イベントを追加してもいいかも？
                // results.push(ProcessedMessageResult::ServerError { message });
            }
        }
    }
    // 処理が終わったら、収集した結果のリストを返す！
    results
} 