// src/lib.rs

// WASM と JavaScript を繋ぐための基本！
use wasm_bindgen::prelude::*;

// 標準ライブラリから、スレッドセーフな共有ポインタとミューテックスを使うよ。
// 非同期のコールバック関数からでも安全にデータを共有・変更するために必要！
use std::sync::{Arc, Mutex};
// メッセージキュー（受信メッセージを一時的に溜めておく場所）のために VecDeque を使うよ。
use std::collections::VecDeque;

// 自分で作ったモジュールたち！ これでコードを整理してるんだ。
pub mod entity;
pub mod component;
pub mod world;
pub mod system;
pub mod components; // components モジュールを宣言
pub mod systems;
pub mod network; // network モジュールを宣言
pub mod protocol; // protocol モジュールを宣言

// 各モジュールから必要な型をインポート！
use crate::world::World;
use crate::network::NetworkManager; // NetworkManager をインポート (ConnectionStatusは不要なので削除)
use crate::protocol::{ClientMessage, ServerMessage, GameStateData, PlayerId}; // protocol から主要な型をインポート
use crate::components::stack::StackType; // components::stack から StackType を直接インポート！
use crate::entity::Entity; // send_make_move で使う Entity も use しておく！
use serde_json; // serde_json を使う
use crate::network::ConnectionStatus; // ↓↓↓ ConnectionStatus を再度 use する！

// JavaScript の console.log を Rust から呼び出すための準備 (extern ブロック)。
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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
}

// GameApp 構造体のメソッドを実装していくよ！
#[wasm_bindgen]
impl GameApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        log("GameApp: Initializing...");
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

        let server_url = format!("ws://{}:{}", "162.43.8.148", 8101);
        let status_arc = Arc::new(Mutex::new(ConnectionStatus::Disconnected));

        let network_manager = NetworkManager::new(
            server_url,
            Arc::clone(&status_arc),
            Arc::clone(&message_queue_arc),
        );
        let network_manager_arc = Arc::new(Mutex::new(network_manager));

        log("GameApp: Initialization complete.");
        Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
        }
    }

    // WebSocket接続 (network.rs 修正待ち → 修正済み！ connect 呼び出しを有効化！)
    pub fn connect(&self) {
        log("GameApp: connect() called.");
        // network.rs が修正されたので、connect の呼び出しを有効にする！
        // network_manager は Arc<Mutex<>> なので、ロックしてからメソッドを呼ぶ。
        // connect は &mut self を取るので、MutexGuard を取得する必要がある。
        match self.network_manager.lock() {
            Ok(mut nm) => nm.connect(), // ロック成功！connect を呼ぶ
            Err(e) => log(&format!("GameApp: Failed to lock NetworkManager for connect: {:?}", e)), // ロック失敗
        }
    }

    // メッセージ送信ヘルパー
    fn send_message(&self, message: ClientMessage) -> Result<(), String> {
        log(&format!("GameApp: Preparing to send message: {:?}", message));
        match serde_json::to_string(&message) {
            Ok(json_message) => {
                let nm = self.network_manager.lock().expect("Failed to lock NetworkManager for sending");
                nm.send_message(&json_message).map_err(|e| e.to_string())
            }
            Err(e) => {
                let error_msg = format!("Failed to serialize ClientMessage: {}", e);
                log(&error_msg);
                Err(error_msg)
            }
        }
    }

    // ゲーム参加メッセージ送信
    #[wasm_bindgen]
    pub fn send_join_game(&self, player_name: String) {
        log(&format!("GameApp: send_join_game called with name: {}", player_name));
        let message = ClientMessage::JoinGame { player_name };
        if let Err(e) = self.send_message(message) {
            log(&format!("GameApp: Failed to send JoinGame message: {}", e));
        }
    }

    // カード移動メッセージ送信
    #[wasm_bindgen]
    pub fn send_make_move(&self, moved_entity_id: usize, target_stack_json: String) {
        log(&format!("GameApp: send_make_move called with entity: {}, target: {}", moved_entity_id, target_stack_json));
        let moved_entity = Entity(moved_entity_id); // Entity を use したので crate::entity:: は不要

        // JSON を StackType に変換 (StackType を use したので直接使える)
        match serde_json::from_str::<StackType>(&target_stack_json) {
            Ok(target_stack) => {
                let message = ClientMessage::MakeMove { moved_entity, target_stack };
                if let Err(e) = self.send_message(message) {
                    log(&format!("GameApp: Failed to send MakeMove message: {}", e));
                }
            }
            Err(e) => {
                log(&format!("GameApp: Failed to deserialize target_stack JSON: {}. Input: {}", e, target_stack_json));
            }
        }
    }

    // 受信メッセージ処理 (借用エラーあり)
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) {
        let mut queue = self.message_queue.lock().expect("Failed to lock message queue");
        while let Some(message) = queue.pop_front() {
            log(&format!("GameApp: Processing received message: {:?}", message));
            match message {
                ServerMessage::GameJoined { your_player_id, initial_game_state } => {
                    *self.my_player_id.lock().expect("Failed to lock my_player_id") = Some(your_player_id);
                    log(&format!("GameApp: Game joined! My Player ID: {}", your_player_id));
                    // ここで借用エラーが発生する！
                    // self.apply_game_state(initial_game_state);
                    log("Error E0502: Temporarily commented out apply_game_state call inside loop.");
                }
                ServerMessage::GameStateUpdate { current_game_state } => {
                    log("GameApp: Received GameStateUpdate.");
                    // ここも借用エラーが発生する！
                    // self.apply_game_state(current_game_state);
                    log("Error E0502: Temporarily commented out apply_game_state call inside loop.");
                }
                ServerMessage::MoveRejected { reason } => {
                    log(&format!("GameApp: Move rejected by server: {}", reason));
                }
                ServerMessage::PlayerJoined { player_id, player_name } => {
                    log(&format!("GameApp: Player {} ({}) joined.", player_name, player_id));
                    // TODO: World にプレイヤー情報を追加/更新する処理
                }
                ServerMessage::PlayerLeft { player_id } => {
                    log(&format!("GameApp: Player {} left.", player_id));
                    // TODO: World からプレイヤー情報を削除/更新する処理
                }
                ServerMessage::Pong => {
                    log("GameApp: Received Pong from server.");
                }
                ServerMessage::Error { message } => {
                    log(&format!("GameApp: Received error from server: {}", message));
                }
            }
        }
        // TODO: ループの外で apply_game_state を呼ぶなど、E0502 エラーの根本対応が必要。
    }

    // ゲーム状態適用 (未実装)
    fn apply_game_state(&mut self, game_state: GameStateData) {
        log("GameApp: Applying game state update...");
        let mut world = self.world.lock().expect("Failed to lock world for game state update");

        log(&format!("  Players: {:?}", game_state.players));

        for card_data in game_state.cards {
            let entity = card_data.entity;
            let card_component = components::card::Card {
                suit: card_data.suit,
                rank: card_data.rank,
                is_face_up: card_data.is_face_up,
            };
            world.add_component(entity, card_component);

            let stack_info_component = components::stack::StackInfo {
                stack_type: card_data.stack_type,
                position_in_stack: card_data.position_in_stack,
            };
            world.add_component(entity, stack_info_component);
        }
        log("GameApp: Game state update applied (implementation is preliminary!).");
    }

    // デバッグ用: 接続状態取得
    #[wasm_bindgen]
    pub fn get_connection_status_debug(&self) -> String {
        match self.network_manager.lock() {
            Ok(nm) => format!("{:?}", nm.get_status()),
            Err(_) => "Error: Failed to lock NetworkManager".to_string(),
        }
    }

    // デバッグ用: プレイヤーID取得
    #[wasm_bindgen]
    pub fn get_my_player_id_debug(&self) -> Option<u32> {
        self.my_player_id.lock().unwrap().clone()
    }
}

// ... 関数型・ベストプラクティスコメント、次のステップコメント ...