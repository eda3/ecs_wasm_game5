// src/lib.rs

// WASM と JavaScript を繋ぐための基本！
use wasm_bindgen::prelude::*;
// ★復活！ JsCast トレイトを使う！★
use wasm_bindgen::JsCast;

// ★修正: web-sys から window と、HtmlElement を使う！ Element は削除！★
use web_sys::{window, HtmlElement, Event, EventTarget, HtmlSpanElement, MouseEvent, DomRect};

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
pub mod rules; // ★追加: 新しい rules モジュールを宣言！

// 各モジュールから必要な型をインポート！
use crate::world::World;
use crate::network::NetworkManager; // NetworkManager をインポート (ConnectionStatusは不要なので削除)
use crate::protocol::{ClientMessage, ServerMessage, GameStateData, CardData, PlayerData, PositionData, PlayerId};
use crate::components::{card::{Card, Rank, Suit}, position::Position, stack::StackInfo, player::Player};
use crate::components::stack::StackType; // components::stack から StackType を直接インポート！
use crate::entity::Entity; // send_make_move で使う Entity も use しておく！
use serde_json; // serde_json を使う
use crate::network::ConnectionStatus; // ↓↓↓ ConnectionStatus を再度 use する！
// systems モジュールと、その中の DealInitialCardsSystem を使う宣言！
use crate::systems::deal_system::DealInitialCardsSystem;
use wasm_bindgen::closure::Closure; // ★追加: イベント関連の型と Closure を use★

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

// --- ★追加: ヘルパー関数 --- ★
fn get_rank_text(rank: &Rank) -> String {
    match rank {
        Rank::Ace => "A".to_string(),
        Rank::King => "K".to_string(),
        Rank::Queen => "Q".to_string(),
        Rank::Jack => "J".to_string(),
        Rank::Ten => "10".to_string(),
        Rank::Nine => "9".to_string(),
        Rank::Eight => "8".to_string(),
        Rank::Seven => "7".to_string(),
        Rank::Six => "6".to_string(),
        Rank::Five => "5".to_string(),
        Rank::Four => "4".to_string(),
        Rank::Three => "3".to_string(),
        Rank::Two => "2".to_string(),
    }
}

fn get_suit_symbol(suit: &Suit) -> String {
    match suit {
        Suit::Heart => "♥".to_string(),
        Suit::Diamond => "♦".to_string(),
        Suit::Club => "♣".to_string(),
        Suit::Spade => "♠".to_string(),
    }
}

// --- ★追加: ドラッグ情報保持用構造体 --- ★
// (Wasm外部には公開しないので #[wasm_bindgen] は不要)
#[derive(Clone, Debug)] // Clone できるようにしておく
struct DraggingInfo {
    entity_id: Entity,
    offset_x: i32,
    offset_y: i32,
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

        // ★ event_closures を初期化 ★
        let event_closures_arc = Arc::new(Mutex::new(Vec::new()));
        // ★追加: 新しいフィールドの初期化★
        let dragging_state_arc = Arc::new(Mutex::new(None));
        let window_mousemove_closure_arc = Arc::new(Mutex::new(None));
        let window_mouseup_closure_arc = Arc::new(Mutex::new(None));

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

    // 受信メッセージ処理 (状態変更フラグを返すように変更！)
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) -> bool { // ★戻り値を bool に変更！
        let mut state_changed = false; // ★状態変更フラグを追加！

        // 1. メッセージキューをロックして、中身を一時的な Vec に移す
        let messages_to_process: Vec<ServerMessage> = { // 新しいスコープを作る
            let mut queue = self.message_queue.lock().expect("Failed to lock message queue");
            queue.drain(..).collect()
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
                    if self.apply_game_state(initial_game_state) { // ★apply_game_state の戻り値を見る
                        state_changed = true; // ★状態が変わったことを記録！
                    }
                }
                ServerMessage::GameStateUpdate { current_game_state } => {
                    log("GameApp: Received GameStateUpdate.");
                    if self.apply_game_state(current_game_state) { // ★apply_game_state の戻り値を見る
                        state_changed = true; // ★状態が変わったことを記録！
                    }
                }
                ServerMessage::MoveRejected { reason } => {
                    log(&format!("GameApp: Move rejected by server: {}", reason));
                    // TODO: MoveRejected をJSに伝える仕組み？
                }
                ServerMessage::PlayerJoined { player_id, player_name } => {
                    log(&format!("GameApp: Player {} ({}) joined.", player_name, player_id));
                    // TODO: プレイヤーリスト更新のために state_changed = true; すべき？
                    //       apply_game_state でプレイヤーも更新するなら不要
                }
                ServerMessage::PlayerLeft { player_id } => {
                    log(&format!("GameApp: Player {} left.", player_id));
                    // TODO: プレイヤーリスト更新のために state_changed = true; すべき？
                }
                ServerMessage::Pong => {
                    log("GameApp: Received Pong from server.");
                }
                ServerMessage::Error { message } => {
                    log(&format!("GameApp: Received error from server: {}", message));
                }
            }
        }
        state_changed // ★最後にフラグの値を返す！
    }

    /// サーバーから受け取った GameStateData を World に反映させる内部関数。
    /// 状態が更新された場合は true を返すように変更！
    fn apply_game_state(&mut self, game_state: GameStateData) -> bool { // ★戻り値を bool に変更！
        log("GameApp: Applying game state update...");
        let mut world = match self.world.lock() { // poison 対応
            Ok(guard) => guard,
            Err(poisoned) => {
                log(&format!("World mutex poisoned in apply_game_state: {:?}. Recovering...", poisoned));
                poisoned.into_inner()
            }
        };

        // ★状態変更があったかどうかのフラグ (今は単純に常に true を返す)
        let mut did_change = false;

        // --- 1. 既存のプレイヤーとカード情報をクリア --- 
        did_change = true; // クリアしたら変更ありとみなす
        log("  Clearing existing player and card entities...");
        let player_entities: Vec<Entity> = world
            .get_all_entities_with_component::<Player>()
            .into_iter()
            .collect();
        for entity in player_entities {
            world.remove_component::<Player>(entity);
            // log(&format!("    Removed Player component from {:?}", entity));
        }
        let card_entities: Vec<Entity> = world
            .get_all_entities_with_component::<Card>()
            .into_iter()
            .collect();
        for entity in card_entities {
            world.remove_component::<Card>(entity);
            world.remove_component::<Position>(entity);
            world.remove_component::<StackInfo>(entity);
            // log(&format!("    Removed Card related components from {:?}", entity));
        }

        // --- 2. 新しいプレイヤー情報を反映 --- 
        if !game_state.players.is_empty() { did_change = true; }
        log(&format!("  Applying {} players...", game_state.players.len()));
        for player_data in game_state.players {
            log(&format!("    Player ID: {}, Name: {}", player_data.id, player_data.name));
            // TODO: 実際にプレイヤーコンポーネントを追加/更新する
        }

        // --- 3. 新しいカード情報を反映 --- 
        if !game_state.cards.is_empty() { did_change = true; }
        log(&format!("  Applying {} cards...", game_state.cards.len()));
        for card_data in game_state.cards {
            let entity = card_data.entity;
            world.create_entity_with_id(entity); // 存在保証
            let card_component = Card {
                suit: card_data.suit,
                rank: card_data.rank,
                is_face_up: card_data.is_face_up,
            };
            world.add_component(entity, card_component);
            let stack_info_component = StackInfo {
                stack_type: card_data.stack_type,
                position_in_stack: card_data.position_in_stack,
            };
            world.add_component(entity, stack_info_component);
            let position_component = Position {
                x: card_data.position.x,
                y: card_data.position.y,
            };
            world.add_component(entity, position_component);
        }

        log("GameApp: Game state update applied.");
        did_change // ★ 変更があったかどうかを返す！
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
        log("GameApp: get_initial_state_data called.");
        let card_entities = world.get_all_entities_with_component::<Card>();
        let mut card_data_list = Vec::with_capacity(card_entities.len());
        log(&format!("  Found {} card entities. Creating CardData list...", card_entities.len()));
        for &entity in &card_entities {
            let card = world.get_component::<Card>(entity).expect(&format!("Card component not found for entity {:?}", entity));
            let stack_info = world.get_component::<StackInfo>(entity).expect(&format!("StackInfo component not found for entity {:?}", entity));
            let position = world.get_component::<Position>(entity).expect(&format!("Position component not found for entity {:?}", entity));
            let position_data = PositionData { x: position.x, y: position.y };
            let card_data = CardData {
                entity, suit: card.suit, rank: card.rank, is_face_up: card.is_face_up,
                stack_type: stack_info.stack_type, position_in_stack: stack_info.position_in_stack,
                position: position_data,
            };
            card_data_list.push(card_data);
        }
        log("  CardData list created successfully.");
        GameStateData { players: Vec::<PlayerData>::new(), cards: card_data_list, }
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
        for entity in card_entities { // ここは &entity ではなく entity でOKだったかも？ world のメソッドによる
            let card = world.get_component::<Card>(entity).expect("Card component not found");
            let stack_info = world.get_component::<StackInfo>(entity).expect("StackInfo component not found");
             // ★ Position も取得！
            let position = world.get_component::<Position>(entity).expect("Position component not found");

            let (stack_type_str, stack_index_json) = match stack_info.stack_type {
                StackType::Stock => ("Stock", serde_json::Value::Null),
                StackType::Waste => ("Waste", serde_json::Value::Null),
                StackType::Foundation(index) => ("Foundation", serde_json::json!(index)),
                StackType::Tableau(index) => ("Tableau", serde_json::json!(index)),
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
        // ★新しいロジック関数を呼び出すように変更！★
        Self::handle_double_click_logic(entity_id, Arc::clone(&self.world), Arc::clone(&self.network_manager));
    }

    /// ★追加: ダブルクリック時の実際のロジック (static メソッドっぽく)★
    fn handle_double_click_logic(entity_id: usize, world_arc: Arc<Mutex<World>>, network_manager_arc: Arc<Mutex<NetworkManager>>) {
        log(&format!("  Executing double-click logic for entity_id: {}", entity_id));
        let entity = Entity(entity_id);

        // World をロックして、必要な情報を取得
        let world = match world_arc.lock() {
            Ok(w) => w,
            Err(e) => {
                error(&format!("Error locking world in handle_double_click_logic: {}", e));
                return;
            }
        };

        // ダブルクリックされたカードを取得
        let card_to_move = match world.get_component::<Card>(entity) {
            Some(card) => card.clone(), // Clone する!
            None => {
                error(&format!("Card component not found for entity {:?} in handle_double_click_logic", entity));
                return;
            }
        };

        // 自動移動先を探す！🔍 (World の参照を渡す)
        let target_stack_opt = rules::find_automatic_foundation_move(&card_to_move, &world);
        // World のロックを早めに解除！
        drop(world);

        match target_stack_opt {
            Some(target_stack) => {
                // 移動先が見つかった！🎉 MakeMove メッセージを送信！🚀
                log(&format!("  Found automatic move target: {:?} for card {:?}", target_stack, card_to_move));
                let message = ClientMessage::MakeMove { moved_entity: entity, target_stack };

                // メッセージ送信 (send_message ヘルパーが使えないので、ここで直接行う)
                match serde_json::to_string(&message) {
                    Ok(json_message) => {
                         match network_manager_arc.lock() {
                             Ok(nm) => {
                                 if let Err(e) = nm.send_message(&json_message) {
                                     error(&format!("  Failed to send MakeMove message from logic: {}", e));
                                 } else {
                                     log("  MakeMove message sent successfully from logic.");
                                 }
                             },
                             Err(e) => error(&format!("Failed to lock NetworkManager in logic: {}", e))
                         }
                    }
                    Err(e) => error(&format!("Failed to serialize MakeMove message in logic: {}", e))
                }
            }
            None => {
                // 移動先は見つからなかった...😢
                log("  No automatic foundation move found for this card (logic).");
            }
        }
    }

    /// Rust側でゲーム画面を描画する関数
    #[wasm_bindgen]
    pub fn render_game_rust(&self) {
        log("GameApp: render_game_rust() called! Setting inline styles...");

        // --- ステップ1 & 2: 要素取得とクリア ---
        let window = window().expect("Failed to get window");
        let document = window.document().expect("Failed to get document");
        let game_area = document.get_element_by_id("game-area").expect("game-area not found");
        game_area.set_inner_html("");
        {
            let mut closures = self.event_closures.lock().expect("Failed to lock event_closures for clearing");
            closures.clear();
        }

        // --- ステップ3: World からカード情報を取得 --- 
        let world = self.world.lock().expect("Failed to lock world for rendering");
        let card_entities = world.get_all_entities_with_component::<Card>();

        // --- ステップ4: カード要素を作成・設定・追加 & イベントリスナー設定★ ---
        for &entity in &card_entities {
            if let (Some(card), Some(position), Some(stack_info)) = (
                world.get_component::<Card>(entity),
                world.get_component::<Position>(entity),
                world.get_component::<StackInfo>(entity)
            ) {
                // --- 要素作成 & キャスト ---
                let card_element_div = document.create_element("div").expect("Failed to create div");
                let card_element = card_element_div.dyn_into::<HtmlElement>().expect("Failed to cast to HtmlElement");
                let style = card_element.style(); // スタイル操作用に取得

                // --- ★ CSS クラスの代わりにインラインスタイルを設定！ --- ★
                // --- 基本スタイル (.card に相当) ---
                style.set_property("width", "72px").expect("set width");
                style.set_property("height", "96px").expect("set height");
                style.set_property("border", "1px solid #aaa").expect("set border");
                style.set_property("border-radius", "5px").expect("set border-radius");
                style.set_property("position", "absolute").expect("set position");
                style.set_property("box-shadow", "1px 1px 3px rgba(0,0,0,0.2)").expect("set box-shadow");
                style.set_property("cursor", "pointer").expect("set cursor");
                style.set_property("user-select", "none").expect("set user-select");
                style.set_property("font-size", "16px").expect("set font-size");
                style.set_property("font-weight", "bold").expect("set font-weight");
                style.set_property("overflow", "hidden").expect("set overflow");
                style.set_property("box-sizing", "border-box").expect("set box-sizing");
                // z-index もここで設定 (stack_info を使う！)
                style.set_property("z-index", &stack_info.position_in_stack.to_string()).expect("set z-index");

                // --- data-entity-id 属性はそのまま設定 ---
                card_element.set_attribute("data-entity-id", &entity.0.to_string()).expect("Failed to set data-entity-id");
                
                // --- 表裏に応じたスタイルと内容 --- 
                if card.is_face_up {
                    // --- 表向きスタイル (.face-up に相当) ---
                    style.set_property("background-color", "#fff").expect("set bg-color white");
                    style.set_property("display", "flex").expect("set display flex");
                    style.set_property("flex-direction", "column").expect("set flex-direction");
                    style.set_property("justify-content", "space-between").expect("set justify-content");
                    style.set_property("padding", "5px").expect("set padding");

                    // スートによる色分け
                    let color = match card.suit {
                        Suit::Heart | Suit::Diamond => "red",
                        Suit::Club | Suit::Spade => "black",
                    };
                    style.set_property("color", color).expect("set color");

                    // --- ランク span 作成 & スタイル設定 ---
                    let rank_span_el = document.create_element("span").expect("Failed to create rank span");
                    let rank_span = rank_span_el.dyn_into::<HtmlSpanElement>().expect("Failed to cast rank span");
                    let rank_style = rank_span.style();
                    rank_style.set_property("display", "block").expect("set rank display");
                    rank_style.set_property("text-align", "center").expect("set rank text-align");
                    rank_style.set_property("line-height", "1").expect("set rank line-height");
                    rank_span.set_text_content(Some(&get_rank_text(&card.rank)));
                    card_element.append_child(&rank_span).expect("Failed to append rank span");

                    // --- スート span 作成 & スタイル設定 ---
                    let suit_span_el = document.create_element("span").expect("Failed to create suit span");
                    let suit_span = suit_span_el.dyn_into::<HtmlSpanElement>().expect("Failed to cast suit span");
                    let suit_style = suit_span.style();
                    suit_style.set_property("display", "block").expect("set suit display block"); // CSS では flex item だったが、一旦 block で
                    suit_style.set_property("text-align", "center").expect("set suit text-align");
                    suit_style.set_property("font-size", "28px").expect("set suit font-size");
                    suit_style.set_property("line-height", "1").expect("set suit line-height");
                    suit_style.set_property("flex-grow", "1").expect("set suit flex-grow"); // flex container なので効くはず
                    suit_style.set_property("display", "flex").expect("set suit display flex"); // 中央揃えのため再設定
                    suit_style.set_property("justify-content", "center").expect("set suit justify");
                    suit_style.set_property("align-items", "center").expect("set suit align");
                    suit_span.set_text_content(Some(&get_suit_symbol(&card.suit)));
                    card_element.append_child(&suit_span).expect("Failed to append suit span");

                    // ★注意: face-up, suit-*, rank-* クラスはもう付けない！★

                } else {
                    // --- 裏向きスタイル (.face-down に相当) ---
                    style.set_property("background-color", "#4a90e2").expect("set bg-color blue");
                    // ★注意: face-down クラスはもう付けない！★
                }
                
                // 位置スタイル設定 (これは変更なし)
                style.set_property("left", &format!("{}px", position.x)).expect("set left");
                style.set_property("top", &format!("{}px", position.y)).expect("set top");

                // イベントリスナー設定 (これも基本変更なし、ただし dragging クラスは削除)
                let target: EventTarget = card_element.clone().into();
                // ... (クリックリスナー - 中で .selected クラス操作をやめる必要あり)
                // ... (ダブルクリックリスナー - 変更なし)
                // ... (mousedown リスナー - 中で .dragging クラス操作をやめる必要あり)
                
                // 要素を追加
                game_area.append_child(&card_element).expect("Failed to append card");
            }
        }
        log("  Finished iterating, appending elements (with inline styles), and adding listeners.");
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