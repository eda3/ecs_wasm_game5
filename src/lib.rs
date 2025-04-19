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
use crate::components::{card::Card, position::Position, stack::StackInfo, player::Player};
use crate::components::stack::StackType; // components::stack から StackType を直接インポート！
use crate::entity::Entity; // send_make_move で使う Entity も use しておく！
use serde_json; // serde_json を使う
use crate::network::ConnectionStatus; // ↓↓↓ ConnectionStatus を再度 use する！
// systems モジュールと、その中の DealInitialCardsSystem を使う宣言！
use crate::systems::deal_system::DealInitialCardsSystem;

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
    // DealInitialCardsSystem のインスタンスを持っておこう！ (状態を持たないので Clone でも Default でもOK)
    deal_system: DealInitialCardsSystem,
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

        log("GameApp: Initialization complete.");
        Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
            deal_system, // deal_system を GameApp に追加！
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

    // 受信メッセージ処理 (借用エラー E0502 修正！)
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) {
        // 1. メッセージキューをロックして、中身を一時的な Vec に移す
        let messages_to_process: Vec<ServerMessage> = { // 新しいスコープを作る
            let mut queue = self.message_queue.lock().expect("Failed to lock message queue");
            // queue.drain(..) を使って、キューの中身をすべて取り出して Vec にする
            queue.drain(..).collect()
            // このスコープの終わりで `queue` (MutexGuard) が破棄され、ロックが解除される！
        }; // ← ここでロック解除！🔓

        // 2. ロックが解除された状態で、一時的な Vec を処理する
        if !messages_to_process.is_empty() {
            log(&format!("GameApp: Processing {} received messages...", messages_to_process.len()));
        }

        for message in messages_to_process {
            log(&format!("  Processing: {:?}", message));
            match message {
                ServerMessage::GameJoined { your_player_id, initial_game_state } => {
                    *self.my_player_id.lock().expect("Failed to lock my_player_id") = Some(your_player_id);
                    log(&format!("GameApp: Game joined! My Player ID: {}", your_player_id));
                    // 借用エラーが解決したのでコメントアウト解除！🎉
                    self.apply_game_state(initial_game_state);
                    // log("Error E0502: Temporarily commented out apply_game_state call inside loop."); // コメント削除
                }
                ServerMessage::GameStateUpdate { current_game_state } => {
                    log("GameApp: Received GameStateUpdate.");
                    // 借用エラーが解決したのでコメントアウト解除！🎉
                    self.apply_game_state(current_game_state);
                    // log("Error E0502: Temporarily commented out apply_game_state call inside loop."); // コメント削除
                }
                ServerMessage::MoveRejected { reason } => {
                    log(&format!("GameApp: Move rejected by server: {}", reason));
                }
                ServerMessage::PlayerJoined { player_id, player_name } => {
                    log(&format!("GameApp: Player {} ({}) joined.", player_name, player_id));
                    // TODO: World にプレイヤー情報を追加/更新する処理 (apply_game_state でやるかも？)
                }
                ServerMessage::PlayerLeft { player_id } => {
                    log(&format!("GameApp: Player {} left.", player_id));
                    // TODO: World からプレイヤー情報を削除/更新する処理 (apply_game_state でやるかも？)
                }
                ServerMessage::Pong => {
                    log("GameApp: Received Pong from server.");
                }
                ServerMessage::Error { message } => {
                    log(&format!("GameApp: Received error from server: {}", message));
                }
            }
        }
        // ループの外で apply_game_state を呼ぶ必要はなくなった！
    }

    /// サーバーから受け取った GameStateData を World に反映させる内部関数。
    /// 方針: 既存のカードとプレイヤー情報をクリアし、受け取ったデータで再構築する！
    fn apply_game_state(&mut self, game_state: GameStateData) {
        log("GameApp: Applying game state update...");
        let mut world = self.world.lock().expect("Failed to lock world for game state update");

        // --- 1. 既存のプレイヤーとカード情報をクリア --- 
        log("  Clearing existing player and card entities...");
        let player_entities: Vec<Entity> = world
            .get_all_entities_with_component::<Player>()
            .into_iter()
            .collect();
        for entity in player_entities {
            world.remove_component::<Player>(entity);
            log(&format!("    Removed Player component from {:?}", entity));
        }

        let card_entities: Vec<Entity> = world
            .get_all_entities_with_component::<Card>()
            .into_iter()
            .collect();
        for entity in card_entities {
            world.remove_component::<Card>(entity);
            world.remove_component::<Position>(entity);
            world.remove_component::<StackInfo>(entity);
            log(&format!("    Removed Card related components from {:?}", entity));
        }
        // 注意: GameState エンティティ (Entity(0)) は削除しないように！

        // --- 2. 新しいプレイヤー情報を反映 --- 
        log(&format!("  Applying {} players...", game_state.players.len()));
        for player_data in game_state.players {
            // TODO: プレイヤーエンティティをどう管理するか？
            //       - プレイヤーごとにエンティティを作成？ (例: world.create_entity()?)
            //       - PlayerId をキーにしたリソースとして管理？
            //       - とりあえずログ出力のみ。
            log(&format!("    Player ID: {}, Name: {}", player_data.id, player_data.name));
            // 仮: Player コンポーネントを追加してみる (PlayerId を Entity ID として使う？危険かも)
            // let player_entity = Entity(player_data.id as usize); // ID を usize にキャスト
            // world.add_component(player_entity, Player { name: player_data.name });
        }

        // --- 3. 新しいカード情報を反映 --- 
        log(&format!("  Applying {} cards...", game_state.cards.len()));
        for card_data in game_state.cards {
            let entity = card_data.entity; // サーバー指定のエンティティID

            // TODO: World にこのエンティティIDが存在しない場合の処理。
            //       world.create_entity_with_id(entity) のような機能が必要。
            //       一旦、存在するものとして進める (DealSystemで事前に作られている想定？)
            //       あるいは、create_entity() で新しいIDを割り当てて、
            //       サーバーのIDとクライアントのIDのマッピングを持つ？複雑になる…。
            //       ここでは、world.add_component が存在しない Entity に
            //       対しても内部でよしなにしてくれる…という期待で進める (実際は要確認！)

            // Card コンポーネントを追加 (or 更新)
            let card_component = Card {
                suit: card_data.suit,
                rank: card_data.rank,
                is_face_up: card_data.is_face_up,
            };
            world.add_component(entity, card_component);

            // StackInfo コンポーネントを追加 (or 更新)
            let stack_info_component = StackInfo {
                stack_type: card_data.stack_type,
                position_in_stack: card_data.position_in_stack,
            };
            world.add_component(entity, stack_info_component);

            // Position コンポーネントを追加 (or 更新)！
            let position_component = Position {
                x: card_data.position.x,
                y: card_data.position.y,
            };
            world.add_component(entity, position_component);

            // log(&format!("    Added/Updated components for card entity {:?}", entity));
        }

        log("GameApp: Game state update applied.");
        // World のロックはこの関数のスコープを抜ける時に自動的に解放される。
    }

    // --- 新しく追加！ JSから初期カード配置を実行するためのメソッド --- 🎉
    #[wasm_bindgen]
    pub fn deal_initial_cards(&self) {
        log("GameApp: deal_initial_cards() called.");
        // World のロックを取得する。
        // self.world は Arc<Mutex<World>> なので、.lock() で MutexGuard を取得する。
        // MutexGuard は World への可変参照 (&mut World) を提供してくれるよ！
        match self.world.lock() {
            Ok(mut locked_world) => {
                // ロックに成功したら、保持している deal_system の execute メソッドを呼び出す！
                // execute メソッドに World の可変参照を渡すよ。
                log("  Executing DealInitialCardsSystem...");
                self.deal_system.execute(&mut locked_world);
                log("  DealInitialCardsSystem executed successfully.");
            }
            Err(e) => {
                // ロックに失敗した場合 (他のスレッドがロックを保持したままパニックしたなど)
                log(&format!("GameApp: Failed to lock world for dealing cards: {:?}", e));
            }
        }
    }

    // WASM から World の状態を取得して JSON 文字列で返す (デバッグ・描画用)
    #[wasm_bindgen]
    pub fn get_world_state_json(&self) -> String {
        log("GameApp: get_world_state_json called.");
        let world = self.world.lock().expect("Failed to lock world for getting state");

        // JSONを作るためのデータを集めるよ！
        // まずは Card コンポーネントを持つ全てのエンティティを取得。
        let card_entities = world.get_all_entities_with_component::<Card>();

        // 各カードエンティティの詳細情報を格納するための Vec を用意。
        // serde_json::Value 型を使って、柔軟なJSONオブジェクトを作れるようにするよ。
        let mut cards_json_data: Vec<serde_json::Value> = Vec::with_capacity(card_entities.len());

        log(&format!("  Found {} card entities. Preparing JSON data...", card_entities.len()));

        // 各カードエンティティについてループして、必要な情報を取得・整形する。
        for entity in card_entities {
            // Card コンポーネントを取得 (存在しない場合はエラーにすべきだけど、ここでは unwrap する)
            let card = world.get_component::<Card>(entity).expect("Card component not found");
            // StackInfo コンポーネントを取得 (これも unwrap)
            let stack_info = world.get_component::<StackInfo>(entity).expect("StackInfo component not found");

            // StackType から stack_type (文字列) と stack_index (数値 or null) を決定する。
            let (stack_type_str, stack_index_json) = match stack_info.stack_type {
                StackType::Stock => ("Stock", serde_json::Value::Null),
                StackType::Waste => ("Waste", serde_json::Value::Null),
                StackType::Foundation(index) => ("Foundation", serde_json::json!(index)),
                StackType::Tableau(index) => ("Tableau", serde_json::json!(index)),
            };

            // カード情報を serde_json::json! マクロを使ってJSONオブジェクトにする！便利！✨
            let card_json = serde_json::json!({
                "entity_id": entity.0, // Entity はタプル構造体 Entity(usize) なので .0 で中身の usize を取得
                "suit": format!("{:?}", card.suit), // Debug フォーマットで文字列化 (例: "Heart")
                "rank": format!("{:?}", card.rank), // Debug フォーマットで文字列化 (例: "Ace")
                "is_face_up": card.is_face_up,
                "stack_type": stack_type_str,
                "stack_index": stack_index_json,
                "order": stack_info.position_in_stack // フィールド名は position_in_stack だったね！
            });

            // 作成したカードJSONデータを Vec に追加。
            cards_json_data.push(card_json);
        }

        log("  Card data preparation complete.");

        // 最終的なJSONオブジェクトを作成。"cards" というキーにカードデータの配列を入れる。
        let final_json = serde_json::json!({ "cards": cards_json_data });

        // JSONオブジェクトを文字列に変換して返す。
        match serde_json::to_string(&final_json) {
            Ok(json_string) => {
                // log(&format!("  Returning world state JSON: {}", json_string)); // ちょっと長いのでコメントアウト
                log("  Successfully serialized world state to JSON.");
                json_string
            }
            Err(e) => {
                log(&format!("Error serializing world state to JSON: {}", e));
                // エラーの場合は空のJSON配列などを返す？あるいはエラー情報を含むJSON？
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