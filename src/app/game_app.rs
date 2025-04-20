// src/app/game_app.rs

// --- 必要なものをインポート ---
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, HtmlCanvasElement, CanvasRenderingContext2d};
use js_sys::Error;
// log クレートのマクロをインポート
use log::{info, error, warn}; // warn も追加しておく

use crate::ecs::world::World;
use crate::network::NetworkManager;
use crate::protocol::{
    self, // protocol モジュール自体も使う
    ServerMessage, PlayerId, GameStateData, PlayerData, CardData, PositionData,
    ClientMessage // ClientMessage も使う
};
use crate::systems::deal_system::DealInitialCardsSystem;
use crate::components::dragging_info::DraggingInfo;
use crate::components::card::Card;
use crate::components::stack::{StackInfo, StackType};
use crate::components::position::Position;
use crate::components::player::Player;
use crate::app::event_handler::{self, ClickTarget}; // event_handler モジュールと ClickTarget を use する！
use crate::{log, error}; // log と error マクロをインポート (lib.rs から)
use crate::ecs::entity::Entity; // Entity を使うためにインポート
use crate::logic::rules;
use serde_json;

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
    // dragging_state: Arc<Mutex<Option<DraggingInfo>>>, // handle_drag_start/end で直接追加/削除するので不要かも
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
        println!("GameApp: 初期化中..."); // 代わりに println! を使用

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
        let window_mousemove_closure_arc = Arc::new(Mutex::new(None));
        let window_mouseup_closure_arc = Arc::new(Mutex::new(None));

        println!("GameApp: 初期化完了。");
        Self {
            world: world_arc,
            network_manager: network_manager_arc,
            message_queue: message_queue_arc,
            my_player_id: my_player_id_arc,
            deal_system,
            event_closures: event_closures_arc,
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

    // カード移動メッセージ送信 (引数を JSON 文字列に戻す)
    #[wasm_bindgen]
    pub fn send_make_move(&self, moved_entity_id: usize, target_stack_json: String) { // 引数を JSON 文字列に戻す
        let moved_entity = Entity(moved_entity_id); // usize から Entity へ

        // JSON 文字列をデシリアライズ
        match serde_json::from_str::<protocol::StackType>(&target_stack_json) {
            Ok(target_stack) => {
                // デシリアライズ成功
                let message = ClientMessage::MakeMove { moved_entity, target_stack };

                match serde_json::to_string(&message) {
                    Ok(json_message) => {
                         match self.network_manager.lock() {
                             Ok(nm) => {
                                 if let Err(e) = nm.send_message(&json_message) {
                                     error!("Failed to send MakeMove message: {}", e);
                                 } else {
                                     info!("MakeMove message sent: {:?}", message);
                                 }
                             },
                             Err(e) => error!("Failed to lock NetworkManager to send MakeMove: {}", e)
                         }
                    }
                    Err(e) => error!("Failed to serialize MakeMove message: {}", e)
                }
            }
            Err(e) => {
                // JSON デシリアライズ失敗
                error!("Failed to deserialize target_stack_json: {}. JSON: {}", e, target_stack_json);
            }
        }
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
        println!("GameApp: get_world_state_json が呼ばれました。World の状態を準備中...");

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
        println!("  プレイヤーデータを収集中...");
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
        println!("    プレイヤー {} 人発見。", players.len());

        // --- 3. カード (`Card`) データの収集 ---
        println!("  カードデータを収集中...");
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
                    eprintln!("警告: エンティティ {:?} に必要なコンポーネント (Card, StackInfo, Position) が全て取得できませんでした。スキップします。", entity);
                    None
                }
            })
            .collect(); // イテレータの結果を Vec<CardData> に集める。
        println!("    完全なデータを持つカード {} 枚発見。", cards.len());


        // --- 4. GameStateData の構築 ---
        println!("  GameStateData を構築中...");
        // 集めたプレイヤーデータとカードデータを使って、`GameStateData` を作るよ！
        let game_state_data = GameStateData {
            players, // さっき集めた players リスト
            cards,   // さっき集めた cards リスト
            // TODO: 必要なら他のフィールド (例: current_turn, game_status) も World から取得して追加する
        };

        // --- 5. JSON 文字列へのシリアライズ ---
        println!("  GameStateData を JSON 文字列にシリアライズ中...");
        // `serde_json::to_string` を使って `GameStateData` を JSON 文字列に変換！ ✨
        // これも失敗する可能性があるので `Result` が返ってくる。
        serde_json::to_string(&game_state_data)
            // `map_err` で、もし `serde_json` がエラー (Err) を返したら...
            .map_err(|e| {
                // エラー内容をコンソールに出力 (eprintln! はエラー出力用)
                eprintln!("GameStateData の JSON シリアライズエラー: {}", e);
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
        println!("GameApp: handle_double_click がエンティティ ID: {} に対して呼ばれました。", entity_id);
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

    /// JavaScript から Canvas 上でのクリックイベントを処理するために呼び出される関数だよ！
    ///
    /// # 引数
    /// * `x`: クリックされた Canvas 上の X 座標 (f32)。
    /// * `y`: クリックされた Canvas 上の Y 座標 (f32)。
    ///
    /// # 処理内容
    /// 1. `event_handler::find_clicked_element` を呼び出して、クリックされた要素 (カード or スタック) を特定する。
    /// 2. 特定された要素に応じて、ログを出力する。(デバッグ用)
    /// 3. **TODO:** 今後は、特定された要素に応じて、カードのドラッグ開始処理や、
    ///    スタッククリック時のアクション (例: 山札クリックでカードをめくる) などを実装していくよ！
    #[wasm_bindgen]
    pub fn handle_click(&self, x: f32, y: f32) {
        // まずは World のロックを取得するよ
        let world = match self.world.lock() {
            Ok(w) => w,
            Err(e) => {
                error(&format!("handle_click 内で World のロックに失敗: {}", e));
                return; // ロック失敗したら何もできないので終了
            }
        };

        // クリックされた要素を探す！ event_handler モジュールの関数を呼び出すよ！
        let clicked_element = event_handler::find_clicked_element(&world, x, y);

        // World のロックはもう不要なので早めに解除！
        drop(world);

        // クリックされた要素に応じてログを出力！ (今はまだログだけ)
        match clicked_element {
            Some(event_handler::ClickTarget::Card(entity)) => {
                // カードがクリックされた！
                log(&format!("カードをクリック: {:?}", entity));
                // TODO: カードクリック時の処理 (ドラッグ開始など) をここに追加！
            }
            Some(event_handler::ClickTarget::Stack(stack_type)) => {
                // スタックエリアがクリックされた！
                log(&format!("スタックエリアをクリック: {:?}", stack_type));
                // TODO: スタッククリック時の処理 (山札をめくるなど) をここに追加！
            }
            None => {
                // 何もないところがクリックされた！
                log("空きスペースをクリック。");
                // TODO: 背景クリック時の処理 (もし必要なら)
            }
        }
    }

    /// JavaScript から呼び出して、指定された Canvas 座標 (x, y) にある
    /// 一番手前の「カード」の Entity ID を取得するための関数だよ！
    /// ダブルクリックされた時に「どのカードがクリックされたか」を JS 側で知るために使うんだ。
    ///
    /// # 引数
    /// * `x`: 判定したい Canvas 上の X 座標 (f32)。
    /// * `y`: 判定したい Canvas 上の Y 座標 (f32)。
    ///
    /// # 戻り値
    /// * `Option<usize>`:
    ///   - `Some(entity_id)`: 指定座標にカードが見つかった場合、そのカードの Entity ID (usize) を返すよ。
    ///   - `None`: 指定座標にカードが見つからなかった場合 (スタックや背景だった場合)。
    ///   JS側では number | undefined として受け取れる！
    ///
    /// # 処理の流れ
    /// 1. `World` のロックを取得する。失敗したらエラーログを出して `None` を返すよ。
    /// 2. `event_handler::find_clicked_element` 関数を呼び出して、指定座標の要素を特定する。
    /// 3. `find_clicked_element` の結果を `match` で判定する。
    ///    - `Some(ClickTarget::Card(entity))` だったら、そのカードの ID (`entity.0`) を `Some()` で包んで返す。
    ///    - それ以外 (`Some(ClickTarget::Stack(_))` や `None`) だったら、`None` を返す。
    /// 4. World のロックを早めに解除する (`drop`)。
    #[wasm_bindgen]
    pub fn get_entity_id_at(&self, x: f32, y: f32) -> Option<usize> {
        // まずは World のロックを取得するよ。ロックは大事！🔒
        let world = match self.world.lock() {
            Ok(w) => w,
            Err(e) => {
                // ロックに失敗したらエラーログを出して None (何も見つからなかった) を返す。
                error(&format!("get_entity_id_at 内で World のロックに失敗: {}", e));
                return None;
            }
        };

        // event_handler モジュールの find_clicked_element 関数を呼び出して、
        // 指定された座標 (x, y) に何があるか調べてもらう！🔍
        let clicked_element = event_handler::find_clicked_element(&world, x, y);

        // World のロックはここで解除！🔓 もう World のデータは必要ないからね。
        // drop(world) を明示的に書くことで、ロックが早く解除されることを保証するよ。
        drop(world);

        // find_clicked_element から返ってきた結果 (Option<ClickTarget>) を match で判定！
        match clicked_element {
            // Some(ClickTarget::Card(entity)) が返ってきたら…
            Some(event_handler::ClickTarget::Card(entity)) => {
                // それはカードがクリックされたってこと！🎉
                // カードの Entity ID (entity は Entity(usize) というタプル構造体なので、中の usize を .0 で取り出す) を Some で包んで返す。
                // これで JS 側は、どのカードがクリックされたか ID を知ることができるね！
                log(&format!("get_entity_id_at: 座標 ({}, {}) でカードエンティティ {:?} を発見。", x, y, entity));
                Some(entity.0) // entity.0 は usize 型
            }
            // Some(ClickTarget::Stack(stack_type)) が返ってきたら…
            Some(event_handler::ClickTarget::Stack(stack_type)) => {
                // それはスタックの空きエリアがクリックされたってことだね。
                // 今回はカードの ID だけが欲しいので、スタックの場合は None を返す。
                log(&format!("get_entity_id_at: 座標 ({}, {}) でスタックエリア {:?} を発見。None を返します。", x, y, stack_type));
                None
            }
            // None が返ってきたら…
            None => {
                // それは背景とか、何もない場所がクリックされたってこと。
                // もちろんカードじゃないので None を返す。
                log(&format!("get_entity_id_at: 座標 ({}, {}) では何も見つからず。None を返します。", x, y));
                None
            }
        }
    }

    // ドラッグ開始時の処理
    // entity: ドラッグが開始されたカードの Entity ID (usize)
    // start_x, start_y: ドラッグが開始された Canvas 上の座標 (f32)
    fn handle_drag_start(&mut self, entity_usize: usize, start_x: f32, start_y: f32) { // usize 型を明示
        // try_lock は Result を返すため、if let Ok で受ける
        if let Ok(mut world) = self.world.try_lock() {
            // Entity 型に変換
            let entity = Entity(entity_usize);

            // ドラッグ対象エンティティから必要なコンポーネントを取得
            // Entity 型で取得
            let position_opt = world.get_component::<Position>(entity);
            let stack_info_opt = world.get_component::<StackInfo>(entity);

            // Position と StackInfo の両方が取得できた場合のみ処理を進める
            if let (Some(position), Some(stack_info)) = (position_opt, stack_info_opt) {
                // ドラッグ開始座標とカードの左上座標の差分 (オフセット) を計算
                let offset_x = start_x - position.x; // f32 のまま計算
                let offset_y = start_y - position.y; // f32 のまま計算

                // DraggingInfo コンポーネントを作成
                // 正しいフィールド名を使用し、型キャストを追加
                let dragging_info = DraggingInfo {
                    original_x: position.x.into(), // f32 -> f64
                    original_y: position.y.into(), // f32 -> f64
                    offset_x: offset_x.into(),   // f32 -> f64
                    offset_y: offset_y.into(),   // f32 -> f64
                    original_position_in_stack: stack_info.position_in_stack as usize, // u8 -> usize
                    // original_stack_entity: stack_info.stack_entity, // StackInfo に存在しないためコメントアウト
                    // ★一時的な修正: ダミーの Entity ID を設定 (usize::MAX は最大値)
                    original_stack_entity: Entity(usize::MAX), // TODO: 後で正しいスタック Entity を取得する
                };

                // エンティティに DraggingInfo コンポーネントを追加
                // add_component は () を返すので match は不要 (エラーハンドリングが必要なら別途)
                // Entity 型で渡す
                world.add_component(entity, dragging_info);
                log::info!("Added DraggingInfo component to entity {:?}", entity);

            } else {
                // 必要なコンポーネントが取得できなかった場合
                log::error!("Failed to get Position or StackInfo for entity {:?} in handle_drag_start", entity);
            }
        } else {
            log::error!("Failed to lock world in handle_drag_start");
        }
    }

    // ドラッグ終了時の処理
    fn handle_drag_end(&mut self, entity_usize: usize, end_x: f32, end_y: f32) {
        log::info!("handle_drag_end: entity={}, end_x={}, end_y={}", entity_usize, end_x, end_y);
        let entity_to_move = Entity(entity_usize); // usize から Entity へ

        // 1. World のロックを取得 (try_lock は read/write の両方で使う)
        let mut world_guard = match self.world.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock world in handle_drag_end: {}", e);
                // ロック失敗時は何もできないので終了
                return;
            }
        };

        // 2. DraggingInfo を削除し、元の情報を取得
        let dragging_info_opt = world_guard.remove_component::<DraggingInfo>(entity_to_move);

        // DraggingInfo が取得できない場合は、不正な状態なので処理を中断
        let dragging_info = match dragging_info_opt {
            Some(info) => info,
            None => {
                log::warn!("DraggingInfo component not found for entity {:?} during drag end. Aborting move.", entity_to_move);
                // World のロックを解除
                drop(world_guard);
                return;
            }
        };
        log::info!("Removed DraggingInfo: {:?}", dragging_info);

        // 3. ドロップ先の要素を特定
        // event_handler::find_clicked_element を呼び出して、ドロップ座標に何があるか調べる
        // World のロックはまだ保持しているので、& *world_guard で参照を渡す
        let drop_target = event_handler::find_clicked_element(&*world_guard, end_x, end_y);
        log::info!("Drop target found: {:?}", drop_target);

        // 4. 移動が妥当か判定し、結果に応じて処理
        let mut move_is_valid = false; // 移動が成功したかどうかのフラグ
        // ★ サーバー通知用と World 更新用の StackType を別々に保持 ★
        let mut target_stack_for_update: Option<StackType> = None;
        let mut target_stack_for_proto: Option<protocol::StackType> = None;

        match drop_target {
            // --- 4a. スタックエリアにドロップされた場合 ---
            Some(ClickTarget::Stack(target_stack_type)) => {
                log::info!("Dropped onto stack area: {:?}", target_stack_type);
                let is_valid = match target_stack_type {
                    StackType::Foundation(index) => {
                        rules::can_move_to_foundation(&*world_guard, entity_to_move, index)
                    }
                    StackType::Tableau(index) => {
                        rules::can_move_to_tableau(&*world_guard, entity_to_move, index)
                    }
                    // Stock や Waste に直接ドロップするルールは通常ないので false とする
                    StackType::Stock | StackType::Waste => {
                        log::warn!("Cannot drop directly onto Stock or Waste.");
                        false
                    }
                    // Hand にドロップするルールもないので false とする
                    StackType::Hand => {
                        log::warn!("Cannot drop onto Hand stack area.");
                        false
                    }
                };

                if is_valid {
                    log::info!("Move to stack {:?} is valid.", target_stack_type);
                    move_is_valid = true;
                    target_stack_for_update = Some(target_stack_type);
                    // ★ protocol::StackType への変換 match に Hand (unreachable) を追加 ★
                    target_stack_for_proto = Some(match target_stack_type {
                        StackType::Stock => protocol::StackType::Stock,
                        StackType::Waste => protocol::StackType::Waste,
                        StackType::Foundation(i) => protocol::StackType::Foundation(i),
                        StackType::Tableau(i) => protocol::StackType::Tableau(i),
                        StackType::Hand => unreachable!("Validated move target cannot be Hand stack"), // is_valid が false なのでここには来ないはず
                    });
                } else {
                    log::info!("Move to stack {:?} is invalid.", target_stack_type);
                }
            }
            // --- 4b. 別のカードの上にドロップされた場合 ---
            Some(ClickTarget::Card(target_card_entity)) => {
                log::info!("Dropped onto card: {:?}", target_card_entity);
                if let Some(target_card_stack_info) = world_guard.get_component::<StackInfo>(target_card_entity) {
                    let target_stack_type = target_card_stack_info.stack_type;
                    log::info!("Target card belongs to stack: {:?}", target_stack_type);
                    // ★ is_valid の match に Hand を追加 ★
                    let is_valid = match target_stack_type {
                        StackType::Foundation(index) => {
                            rules::can_move_to_foundation(&*world_guard, entity_to_move, index)
                        }
                        StackType::Tableau(index) => {
                            rules::can_move_to_tableau(&*world_guard, entity_to_move, index)
                        }
                        StackType::Stock | StackType::Waste => {
                            log::warn!("Cannot drop onto a card in Stock or Waste.");
                            false
                        }
                        // Hand のカード上へのドロップも無効
                        StackType::Hand => {
                            log::warn!("Cannot drop onto a card in Hand stack.");
                            false
                        }
                    };

                    if is_valid {
                        log::info!("Move to stack {:?} (via card drop) is valid.", target_stack_type);
                        move_is_valid = true;
                        target_stack_for_update = Some(target_stack_type);
                        // ★ protocol::StackType への変換 match に Hand (unreachable) を追加 ★
                        target_stack_for_proto = Some(match target_stack_type {
                            StackType::Stock => protocol::StackType::Stock,
                            StackType::Waste => protocol::StackType::Waste,
                            StackType::Foundation(i) => protocol::StackType::Foundation(i),
                            StackType::Tableau(i) => protocol::StackType::Tableau(i),
                            StackType::Hand => unreachable!("Validated move target cannot be Hand stack"),
                        });
                    } else {
                        log::info!("Move to stack {:?} (via card drop) is invalid.", target_stack_type);
                    }
                } else {
                    log::error!("Failed to get StackInfo for target card {:?}.", target_card_entity);
                    move_is_valid = false;
                }
            }
            // --- 4c. 何もない場所にドロップされた場合 ---
            None => {
                log::info!("Dropped onto empty space. Move is invalid.");
                move_is_valid = false;
            }
        }

        // 5. World 更新、サーバー通知、または位置リセットの実行
        if move_is_valid {
            // ★ target_stack_for_update と target_stack_for_proto の両方が Some であることを確認 ★
            if let (Some(stack_for_update), Some(stack_for_proto)) = (target_stack_for_update, target_stack_for_proto) {
                // ★ 修正: 正しい引数を渡す ★
                self.update_world_and_notify_server(world_guard, entity_to_move, stack_for_update, stack_for_proto);
            } else {
                 log::error!("Move was valid but target stack types were None. This should not happen!");
                 self.reset_card_position(world_guard, entity_to_move, &dragging_info);
            }
        } else {
            self.reset_card_position(world_guard, entity_to_move, &dragging_info);
        }
    }

    // --- ヘルパー関数: World 更新とサーバー通知 --- (シグネチャは変更なし、内部の型変換を削除)
    fn update_world_and_notify_server(
        &self,
        mut world: std::sync::MutexGuard<'_, World>,
        moved_entity: Entity,
        target_stack_type_for_update: StackType, // World 更新用
        target_stack_type_for_proto: protocol::StackType // サーバー通知用
    ) {
        log::info!("Updating world and notifying server for entity {:?} moving to {:?}", moved_entity, target_stack_type_for_update);

        // --- World 更新 ---
        // 1. 新しい Position の計算 (TODO: 正確な計算ロジックが必要！)
        let new_pos = Position { x: 100.0, y: 100.0 }; // 仮の位置
        if let Some(pos_component) = world.get_component_mut::<Position>(moved_entity) {
            *pos_component = new_pos;
            log::info!("  Updated Position for entity {:?}", moved_entity);
        } else {
            log::error!("  Failed to get Position component for entity {:?}", moved_entity);
        }

        // 2. 新しい StackInfo の計算 (TODO: position_in_stack の計算が必要！)
        let new_pos_in_stack: u8 = 0; // 仮の値
        if let Some(stack_info_component) = world.get_component_mut::<StackInfo>(moved_entity) {
            stack_info_component.stack_type = target_stack_type_for_update; // ★ 渡された更新用型を使用
            stack_info_component.position_in_stack = new_pos_in_stack;
            log::info!("  Updated StackInfo for entity {:?}", moved_entity);
        } else {
            log::error!("  Failed to get StackInfo component for entity {:?}", moved_entity);
        }

        // 3. 移動元のスタックで公開されるカードを表にする処理 (TODO: 実装)
        log::warn!("  TODO: Implement logic to turn face up the revealed card in the original stack.");

        drop(world);
        log::info!("  World lock released.");

        // --- サーバー通知 (NetworkManager を直接使う) ---
        // ★ 渡されたプロトコル用型を使用 ★
        let message = ClientMessage::MakeMove { moved_entity, target_stack: target_stack_type_for_proto };
        match serde_json::to_string(&message) {
            Ok(json_message) => {
                match self.network_manager.lock() {
                    Ok(nm) => {
                        if let Err(e) = nm.send_message(&json_message) {
                            error!("Failed to send MakeMove message directly: {}", e);
                        } else {
                            info!("MakeMove message sent directly: {:?}", message);
                        }
                    }
                    Err(e) => error!("Failed to lock NetworkManager to send MakeMove directly: {}", e)
                }
            }
            Err(e) => error!("Failed to serialize MakeMove message directly: {}", e)
        }
    }

    // --- ヘルパー関数: カード位置のリセット --- (シグネチャは変更なし、内部の型変換を削除)
    fn reset_card_position(
        &self,
        mut world: std::sync::MutexGuard<'_, World>, // MutexGuard を受け取る
        entity: Entity,
        dragging_info: &DraggingInfo // 戻す位置の情報を持つ
    ) {
        log::info!("Resetting position for entity {:?}", entity);
        if let Some(pos_component) = world.get_component_mut::<Position>(entity) {
            // DraggingInfo に保存されていた元の座標に戻す
            pos_component.x = dragging_info.original_x as f32; // f64 -> f32
            pos_component.y = dragging_info.original_y as f32; // f64 -> f32
            log::info!("  Position reset to ({}, {})", pos_component.x, pos_component.y);
        } else {
            log::error!("  Failed to get Position component for entity {:?} during reset", entity);
        }
        // World のロックはスコープを抜けるときに解除される
        // drop(world); // 明示的に書いても良い
    }

} // impl GameApp の終わり

// GameApp が不要になった時に WebSocket 接続を閉じる処理 (Drop トレイト)
impl Drop for GameApp {
    fn drop(&mut self) {
        println!("GameApp: GameApp インスタンスを破棄中。WebSocket を切断します...");
        match self.network_manager.lock() {
            Ok(mut nm) => nm.disconnect(),
            Err(e) => eprintln!("GameApp: 切断のために NetworkManager のロックに失敗: {:?}", e),
        }
    }
} 