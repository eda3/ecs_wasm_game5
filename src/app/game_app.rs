// src/app/game_app.rs

// --- 必要なものをインポート ---
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, HtmlCanvasElement, CanvasRenderingContext2d};
use js_sys::Error;

use crate::ecs::world::World;
use crate::network::{NetworkManager, /*ConnectionStatus*/};
use crate::protocol::{
    ServerMessage, PlayerId, GameStateData, PlayerData, CardData, PositionData, /*StackType*/
};
use crate::systems::deal_system::DealInitialCardsSystem;
use crate::components::dragging_info::DraggingInfo;
use crate::components::card::Card;
use crate::components::stack::StackInfo;
use crate::components::position::Position;
use crate::components::player::Player;
// use crate::ecs::entity::Entity; // 未使用
// use crate::app::init_handler; // 未使用 (super:: で直接呼ぶため)
// use crate::app::network_handler; // 未使用 (super:: で直接呼ぶため)
// use crate::app::event_handler; // 未使用 (super:: で直接呼ぶため)
// use crate::app::state_handler; // 未使用 (super:: で直接呼ぶため)
// use crate::app::renderer; // 未使用 (super:: で直接呼ぶため)
// use crate::app::app_state::AppState; // ★ app_state が見つからないため一旦コメントアウト

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
        // log() は lib.rs で定義されているため、ここでは直接使えない
        // 必要なら crate::log() などで参照するか、GameApp 内で log を呼ぶ関数を用意する
        // println! マクロなどは使える
        println!("GameApp: Initializing..."); // 代わりに println! を使用

        // --- World, Network, Canvas の初期化は init_handler に委譲 ---
        let world_arc = super::init_handler::initialize_world(); // app:: -> super::
        let message_queue_arc = Arc::new(Mutex::new(VecDeque::new()));
        let network_manager_arc = super::init_handler::initialize_network(Arc::clone(&message_queue_arc)); // app:: -> super::

        // Canvas 初期化 (エラー処理は expect で簡略化)
        let (canvas, context) = super::init_handler::initialize_canvas() // app:: -> super::
            .expect("Failed to initialize canvas and context");

        // --- その他のフィールド初期化 ---
        let my_player_id_arc = Arc::new(Mutex::new(None));
        let deal_system = DealInitialCardsSystem::default();
        let event_closures_arc = Arc::new(Mutex::new(Vec::new()));
        let dragging_state_arc = Arc::new(Mutex::new(None));
        let window_mousemove_closure_arc = Arc::new(Mutex::new(None));
        let window_mouseup_closure_arc = Arc::new(Mutex::new(None));

        println!("GameApp: Initialization complete.");
        Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
            deal_system,
            event_closures: event_closures_arc,
            dragging_state: dragging_state_arc,
            window_mousemove_closure: window_mousemove_closure_arc,
            window_mouseup_closure: window_mouseup_closure_arc,
            canvas,
            context,
        }
    }

    // WebSocket接続
    pub fn connect(&self) {
        // ★修正: app::network_handler の関数を呼び出す！★
        super::network_handler::connect(&self.network_manager); // app:: -> super::
    }

    // ゲーム参加メッセージ送信
    #[wasm_bindgen]
    pub fn send_join_game(&self, player_name: String) {
        // ★修正: app::network_handler の関数を呼び出す！★
        super::network_handler::send_join_game(&self.network_manager, player_name); // app:: -> super::
    }

    // カード移動メッセージ送信
    #[wasm_bindgen]
    pub fn send_make_move(&self, moved_entity_id: usize, target_stack_json: String) {
        // ★修正: app::network_handler の関数を呼び出す！★
        super::network_handler::send_make_move(&self.network_manager, moved_entity_id, target_stack_json); // app:: -> super::
    }

    // 受信メッセージ処理
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) -> bool {
        // ★修正: app::network_handler の関数を呼び出す！ 必要な Arc を渡す★
        super::network_handler::process_received_messages( // app:: -> super::
            &self.message_queue,
            &self.my_player_id,
            &self.world
        )
    }

    // JSから初期カード配置を実行するためのメソッド
    #[wasm_bindgen]
    pub fn deal_initial_cards(&self) {
        // ★修正: app::init_handler の関数を呼び出す！★
        super::init_handler::deal_initial_cards( // app:: -> super::
            &self.world,
            &self.network_manager,
            &self.deal_system
        );
    }

    /// WASM 側 (`GameApp`) が保持しているゲームの世界 (`World`) の現在の状態を、
    /// JSON 文字列形式で取得するためのメソッドだよ！ JavaScript 側から呼び出して、
    /// デバッグ目的でコンソールに表示したり、画面描画に使ったりすることを想定してるよ！ ✨
    ///
    /// # 戻り値 (Return Value)
    /// - `Ok(String)`: `World` の状態を表す `GameStateData` を JSON 文字列に変換して返すよ！成功！🎉
    /// - `Err(JsValue)`: 何か問題が発生した場合（`World` のロック失敗、JSON への変換失敗など）は、
    ///                  JavaScript 側でエラーとして扱える `JsValue` を返すよ。失敗！😭
    ///
    /// # 処理の流れ (Process Flow)
    /// 1. `World` のデータを安全に読み書きするために、`Mutex` をロックするよ。(`lock().expect()` は仮。本当は `?` でエラー伝播したいけど、`wasm-bindgen` の制約で少し工夫が必要かも)
    /// 2. `World` から「プレイヤー (`Player`)」コンポーネントを持つ全てのエンティティを取得するよ。
    /// 3. 各プレイヤーエンティティから `PlayerData` を作るよ。`Player` コンポーネントから名前などを取得する。
    /// 4. `World` から「カード (`Card`)」コンポーネントを持つ全てのエンティティを取得するよ。
    /// 5. 各カードエンティティから `CardData` を作るよ。`Card`, `StackInfo`, `Position` コンポーネントから必要な情報を取得する。
    /// 6. 作成した `PlayerData` のリストと `CardData` のリストを使って、`GameStateData` 構造体のインスタンスを作るよ。
    /// 7. `GameStateData` インスタンスを `serde_json::to_string` を使って JSON 文字列に変換（シリアライズ）するよ。
    /// 8. 成功したら JSON 文字列を `Ok` で包んで、失敗したらエラー情報を `Err(JsValue)` で包んで返すよ。
    ///
    /// # 関数型スタイルについて (Functional Style Notes)
    /// - `World` からエンティティリストを取得した後、`iter()`, `map()`, `filter_map()`, `collect()` などの
    ///   イテレータメソッドを積極的に使って、データを変換・収集していくよ！ これは Rust でよく使われるイディオム（慣用句）だよ！ ✨
    /// - `for` ループを完全に排除するわけじゃないけど、データの変換処理は `map` とかで書くとスッキリすることが多いよ！ 👍
    #[wasm_bindgen]
    pub fn get_world_state_json(&self) -> Result<String, JsValue> {
        // デバッグ用にコンソールに出力！ (JavaScript の console.log みたいなもの)
        println!("GameApp: get_world_state_json called. Preparing world state...");

        // 1. World の Mutex をロックする！ 🔑
        //   - `self.world` は `Arc<Mutex<World>>` 型だよ。複数の場所から安全に World を使うための仕組み。
        //   - `.lock()` で Mutex のロックを取得しようとする。他の誰かがロックしてたら、解除されるまで待つよ。
        //   - `.map_err(|e| ...)`: もしロック取得に失敗 (前の所有者がパニックしたとか) したら...
        //     - `e.to_string()` でエラー内容を文字列にして、
        //     - `Error::new()` で JavaScript の Error オブジェクトを作って、
        //     - `JsValue::from()` でそれを `JsValue` に変換して `Err` として返すよ。JS にエラーを伝える！
        //   - `?` 演算子: `Result` が `Ok(値)` なら中の値を取り出し、`Err(エラー)` なら即座に関数からそのエラーを返す、超便利なやつ！ ✨
        let world = self.world.lock()
            .map_err(|e| JsValue::from(Error::new(&format!("Failed to lock world: {}", e))))?;

        // --- 2. プレイヤー (`Player`) データの収集 ---
        println!("  Collecting player data...");
        // `world.get_all_entities_with_component::<Player>()` で Player コンポーネントを持つ全エンティティIDを取得。
        let player_entities = world.get_all_entities_with_component::<Player>();
        // `iter()`: エンティティIDのリストをイテレータ（順番に処理できるやつ）に変換。
        // `filter_map(|&entity| ...)`: 各エンティティID (`entity`) に対して処理を行う。
        //   - `world.get_component::<Player>(entity)` で Player コンポーネントを取得 (Option<Player> が返る)。
        //   - `map(|player| ...)`: もし Player コンポーネントが取得できたら (`Some(player)`)、PlayerData を作る。
        //     - `PlayerData { id: entity.0 as PlayerId, name: player.name.clone() }`
        //       - `entity.0` は Entity 型の中の usize 値。それを PlayerId (u32) にキャスト。
        //       - `player.name.clone()`: Player コンポーネントから名前をコピーしてくる。
        //   - `filter_map` は `Some(PlayerData)` だけを集めて、`None` は無視する。万が一 Player が取れなくても大丈夫！
        // `collect::<Vec<_>>()`: イテレータの結果 (PlayerData) を Vec (リスト) に集める。
        let players: Vec<PlayerData> = player_entities.iter()
            .filter_map(|&entity| {
                world.get_component::<Player>(entity).map(|player| {
                    PlayerData {
                        id: entity.0 as PlayerId, // Entity (usize) から PlayerId (u32) へキャスト
                        name: player.name.clone(), // Player コンポーネントから名前をコピー
                    }
                })
            })
            .collect();
        println!("    Found {} players.", players.len());

        // --- 3. カード (`Card`) データの収集 ---
        println!("  Collecting card data...");
        // Player と同様に、Card コンポーネントを持つ全エンティティIDを取得。
        let card_entities = world.get_all_entities_with_component::<Card>();
        // `filter_map` を使って、必要なコンポーネント (Card, StackInfo, Position) が
        // 全て揃っているエンティティだけを `CardData` に変換して集めるよ！
        let cards: Vec<CardData> = card_entities.iter()
            .filter_map(|&entity| {
                // カードに必要なコンポーネントをまとめて取得しようとする
                let card_opt = world.get_component::<Card>(entity);
                let stack_info_opt = world.get_component::<StackInfo>(entity);
                let position_opt = world.get_component::<Position>(entity);

                // `if let` を使って、全てのコンポーネントが `Some` (取得成功) だったら中身を取り出す。
                // 一つでも `None` (取得失敗) だったら、この `filter_map` のクロージャは `None` を返すので、
                // そのエンティティのデータは無視されるよ。安全！ 👍
                if let (Some(card), Some(stack_info), Some(position)) = (card_opt, stack_info_opt, position_opt) {
                    // 全て取得成功！ `CardData` を構築する。
                    Some(CardData {
                        entity, // エンティティID そのもの
                        suit: card.suit,
                        rank: card.rank,
                        is_face_up: card.is_face_up,
                        stack_type: stack_info.stack_type, // StackInfo から取得
                        position_in_stack: stack_info.position_in_stack, // StackInfo から取得
                        position: PositionData { // PositionData を作る
                            x: position.x, // Position から取得
                            y: position.y, // Position から取得
                        },
                    })
                } else {
                    // 必要なコンポーネントが揃っていなかった場合 (普通はありえないはずだけど念のため)
                    // エラーログを出力して、このエンティティはスキップ (`None` を返す)
                    eprintln!("Warning: Could not retrieve all required components (Card, StackInfo, Position) for entity {:?}. Skipping.", entity);
                    None
                }
            })
            .collect(); // イテレータの結果を Vec<CardData> に集める。
        println!("    Found {} cards with complete data.", cards.len());


        // --- 4. GameStateData の構築 ---
        println!("  Constructing GameStateData...");
        // 集めたプレイヤーデータとカードデータを使って、`GameStateData` を作るよ！
        let game_state_data = GameStateData {
            players, // さっき集めた players リスト
            cards,   // さっき集めた cards リスト
            // TODO: 必要なら他のフィールド (例: current_turn, game_status) も World から取得して追加する
        };

        // --- 5. JSON 文字列へのシリアライズ ---
        println!("  Serializing GameStateData to JSON string...");
        // `serde_json::to_string` を使って `GameStateData` を JSON 文字列に変換！ ✨
        // これも失敗する可能性があるので `Result` が返ってくる。
        serde_json::to_string(&game_state_data)
            // `map_err` で、もし `serde_json` がエラー (Err) を返したら...
            .map_err(|e| {
                // エラー内容をコンソールに出力 (eprintln! はエラー出力用)
                eprintln!("Error serializing GameStateData to JSON: {}", e);
                // JavaScript の Error オブジェクトを作って JsValue に変換して返す！
                JsValue::from(Error::new(&format!("Failed to serialize game state: {}", e)))
            })
        // `map_err` が成功した場合は `Ok(json_string)` がそのまま返る。
        // `map_err` が失敗した場合は `Err(js_value)` が返る。
        // これで関数の戻り値の型 `Result<String, JsValue>` にピッタリ合うね！ 🎉
    }

    // 接続状態を文字列で返す (デバッグ用)
    #[wasm_bindgen]
    pub fn get_connection_status_debug(&self) -> String {
        let status = self.network_manager.lock().expect("Failed to lock NetworkManager for status").get_status();
        format!("{:?}", status)
    }

    // 自分の Player ID を返す (デバッグ用)
    #[wasm_bindgen]
    pub fn get_my_player_id_debug(&self) -> Option<u32> {
        self.my_player_id.lock().expect("Failed to lock my_player_id").map(|id| id)
    }

    /// カードがダブルクリックされた時の処理 (JSから呼び出される元のメソッド)
    #[wasm_bindgen]
    pub fn handle_double_click(&self, entity_id: usize) {
        println!("GameApp: handle_double_click called for entity_id: {}", entity_id);
        super::event_handler::handle_double_click_logic( // app:: -> super::
            entity_id,
            Arc::clone(&self.world),
            Arc::clone(&self.network_manager)
        );
    }

    /// Rust側で Canvas にゲーム画面を描画する関数
    #[wasm_bindgen]
    pub fn render_game_rust(&self) -> Result<(), JsValue> {
        super::renderer::render_game_rust( // app:: -> super::
            &self.world,
            &self.canvas,
            &self.context
        // JsValue に変換する必要があるので .map_err を追加
        ).map_err(|e| JsValue::from(Error::new(&format!("Render error: {:?}", e))))
    }
}

// GameApp が不要になった時に WebSocket 接続を閉じる処理 (Drop トレイト)
impl Drop for GameApp {
    fn drop(&mut self) {
        println!("GameApp: Dropping GameApp instance. Disconnecting WebSocket...");
        match self.network_manager.lock() {
            Ok(mut nm) => nm.disconnect(),
            Err(e) => eprintln!("GameApp: Failed to lock NetworkManager for disconnect: {:?}", e),
        }
    }
} 