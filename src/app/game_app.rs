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
// --- レイアウト情報とレンダラー定数をインポート --- ★追加★
use crate::config::layout;
use crate::app::renderer::{RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT};

// ★追加: network_handler から ProcessedMessageResult をインポート★
use super::network_handler::ProcessedMessageResult;

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
    // ★戻り値を `bool` から `Option<usize>` に変更！★
    //   `usize` は拒否されたカードの Entity ID を表すよ。
    //   拒否がなければ `None`、あれば最初の `Some(entity_id)` を返す。
    #[wasm_bindgen]
    pub fn process_received_messages(&mut self) -> Option<usize> { 
        // ★ network_handler の関数を呼び出す！ 戻り値は Vec<ProcessedMessageResult> ★
        let results = super::network_handler::process_received_messages( // app:: -> super::
            &self.message_queue,
            &self.my_player_id,
            &self.world
        );

        // --- ★処理結果をチェックして、JSに返す値を決定する！★ ---
        // results (Vec<ProcessedMessageResult>) をループで見ていくよ。
        for result in results {
            match result {
                // もし MoveRejected イベントが見つかったら…
                ProcessedMessageResult::MoveRejected { entity_id, reason: _ } => {
                    // ログにも一応出しておく (JS側でも出すけど念のため)
                    log(&format!("GameApp: Move rejected event processed for entity {:?}. Returning ID to JS.", entity_id));
                    // その entity_id (Entity 型なので .0 で中の usize を取り出す) を
                    // Some() で包んで、この関数の戻り値として **すぐに返す！** (return)
                    // これで、最初に見つかった拒否イベントだけが JS に伝わるよ。
                    return Some(entity_id.0);
                }
                // StateChanged や Nothing の場合は、ここでは何もしないでループを続ける。
                // (再描画は requestAnimationFrame のループで毎回行われるので、
                //  StateChanged かどうかを JS に伝える必要は今はなさそう)
                ProcessedMessageResult::StateChanged => {
                    // log("GameApp: State changed event processed."); // 必要ならログ出す
                }
                ProcessedMessageResult::Nothing => {
                    // log("GameApp: Nothing event processed."); // 必要ならログ出す
                }
            }
        }

        // ループが全部終わっても MoveRejected が見つからなかった場合
        // (つまり、拒否イベントが結果リストになかった場合) は、None を返す。
        log("GameApp: No MoveRejected event found in processed messages. Returning None to JS.");
        None
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
    #[wasm_bindgen]
    pub fn handle_drag_start(&mut self, entity_usize: usize, start_x: f32, start_y: f32) { // usize 型を明示
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

    /// ドラッグ終了時の処理 (マウスボタンが離された時)
    /// - カードのエンティティID (`entity_usize`) とドロップ座標 (`end_x`, `end_y`) を受け取るよ。
    /// - ドロップ座標にある要素を特定する。
    /// - もしドロップ先が有効なスタックなら:
    ///   - 移動ルール (`is_move_valid`) をチェックする。
    ///   - ルール上OKなら:
    ///     - `DraggingInfo` を削除する。
    ///     - `update_world_and_notify_server` を呼び出して、World の状態を更新し、サーバーに移動を通知する。
    ///   - ルール上NGなら:
    ///     - `DraggingInfo` を削除する。
    ///     - カードの位置を元の位置 (`original_position` in `DraggingInfo`) に戻す。
    ///     - サーバーには通知しない。
    /// - もしドロップ先が有効なスタックでないなら:
    ///   - `DraggingInfo` を削除する。
    ///   - カードの位置を元の位置に戻す。
    ///   - サーバーには通知しない。
    #[wasm_bindgen]
    pub fn handle_drag_end(&mut self, entity_usize: usize, end_x: f32, end_y: f32) {
        let entity = Entity(entity_usize);
        log(&format!("handle_drag_end called for entity: {:?}, drop coordinates: ({}, {})", entity, end_x, end_y));

        let mut world = self.world.lock().expect("Failed to lock world for drag end");

        // --- 1. DraggingInfo と元のスタック情報を取得 ---
        // まず、対象エンティティから DraggingInfo を削除しつつ取得する。
        // これがないとドラッグ中のカードではないので、何もせずに終了。
        let dragging_info_opt = world.remove_component::<DraggingInfo>(entity);

        let dragging_info = match dragging_info_opt {
            Some(info) => {
                log(&format!("  - Successfully removed DraggingInfo: {:?}", info));
                info // `info` を返す
            }
            None => {
                // DraggingInfo がない = このカードはドラッグされていなかった or 既にドラッグが終わっている
                log(&format!("  - Warning: DraggingInfo not found for entity {:?}. Ignoring drag end.", entity));
                return; // 何もせずに関数を抜ける
            }
        };

        // 移動元のスタック情報を DraggingInfo から取得しておく
        // (ルールチェックや、移動失敗時に戻すスタックの識別に使う)
        // この時点ではまだコンポーネントは削除されていないはず
        let original_stack_info = world.get_component::<StackInfo>(dragging_info.original_stack_entity)
                                       .cloned(); // Option<StackInfo> -> cloned() で Option<StackInfo>

        // --- 2. ドロップ先の要素を特定 ---
        // Canvas の座標 (end_x, end_y) を World 座標に変換する必要があるか確認
        // (今は renderer と同じ座標系と仮定)
        // TODO: 必要なら座標変換処理を追加
        log(&format!("  - Finding element at drop coordinates: ({}, {})", end_x, end_y));
        // ★修正: find_element_at_position は存在しないため、find_clicked_element を使う★
        let target_element = event_handler::find_clicked_element(&world, end_x, end_y);
        log(&format!("  - Found target element: {:?}", target_element));

        // --- 3. ドロップ先に基づいて処理を分岐 ---
        match target_element {
            // --- 3a. ドロップ先が有効なスタックだった場合 ---
            Some(ClickTarget::Stack(target_stack_type)) => { // ★変数名を target_stack_type に変更★
                log(&format!("  - Target is a stack area: {:?}", target_stack_type));

                // ★修正: StackType から対応する Entity を検索する★
                let target_stack_entity_opt = world.find_entity_by_stack_type(target_stack_type);

                // ★修正: 見つかった Entity を使って StackInfo を取得する★
                if let Some(target_stack_entity) = target_stack_entity_opt {
                    log(&format!("    Found stack entity: {:?}", target_stack_entity));
                    // ターゲットスタックの情報を取得 (Entity を使う！)
                    let target_stack_info = world.get_component::<StackInfo>(target_stack_entity);

                    if let Some(target_stack_info) = target_stack_info {
                        // let target_stack_type = target_stack_info.stack_type; // ここは元の target_stack_type と同じはず
                        log(&format!("    Target stack type from component: {:?}", target_stack_info.stack_type));

                        // --- 3a-i. 移動ルールのチェック --- (ここから下は target_stack_entity と target_stack_type を使う)
                        log("  - Checking move validity...");
                        let original_stack_type_for_rules = original_stack_info.as_ref().map(|info| info.stack_type);
                        let moved_card = world.get_component::<Card>(entity).expect("Moved entity must have Card component");

                        let is_valid = match target_stack_type { // ルールチェックは StackType で行う
                            StackType::Foundation(index) => {
                                rules::can_move_to_foundation(&world, entity, index)
                            }
                            StackType::Tableau(index) => {
                                rules::can_move_to_tableau(&world, entity, index)
                            }
                            _ => {
                                log(&format!("  - Dropping onto {:?} is not allowed.", target_stack_type));
                                false
                            }
                        };

                        if is_valid { // 計算した is_valid の結果を使う
                            // --- 3a-ii. 移動ルール OK の場合 ---
                            log("  - Move is valid! Updating world and notifying server...");
                            let target_stack_type_for_proto: protocol::StackType = target_stack_type.into();
                            self.update_world_and_notify_server(
                                world, 
                                entity,
                                target_stack_type, // World 更新には StackType を渡す
                                target_stack_type_for_proto,
                                &dragging_info,
                                original_stack_info
                            );
                        } else {
                            // --- 3a-iii. 移動ルール NG の場合 ---
                            log("  - Move is invalid. Resetting card position.");
                            self.reset_card_position(world, entity, &dragging_info);
                        }
                    } else {
                        // target_stack_entity は見つかったが、StackInfo が取得できなかった場合 (通常はありえない)
                        error(&format!("  - Error: StackInfo not found for target stack entity {:?}. Resetting card position.", target_stack_entity));
                        self.reset_card_position(world, entity, &dragging_info);
                    }
                } else {
                    // target_stack_type に対応する Entity が見つからなかった場合 (通常はありえない)
                    error!("  - Error: Stack entity not found for type {:?}. Resetting card position.", target_stack_type);
                    self.reset_card_position(world, entity, &dragging_info);
                }
            }
            // --- 3b. ドロップ先がカードだった場合 (今はスタックへのドロップのみ想定) ---
            Some(ClickTarget::Card(target_card_entity)) => {
                log(&format!("  - Target is a card ({:?}). Invalid drop target. Resetting card position.", target_card_entity));
                // カードの上は無効なドロップ先として扱う
                self.reset_card_position(world, entity, &dragging_info);
            }
            // --- 3c. ドロップ先が空の領域だった場合 ---
            None => {
                log("  - Target is empty space. Resetting card position.");
                // 何もない場所へのドロップも無効
                self.reset_card_position(world, entity, &dragging_info);
            }
        }

        // ドラッグ終了処理が終わったら、Window のリスナーを解除する
        // (成功時も失敗時も解除する)
        *self.window_mousemove_closure.lock().unwrap() = None;
        *self.window_mouseup_closure.lock().unwrap() = None;
        log("  - Removed window mousemove and mouseup listeners.");

    } // handle_drag_end の終わり


    /// World の状態を更新し、サーバーに移動を通知する内部ヘルパー関数。
    /// `handle_drag_end` から、移動が有効だと判断された場合に呼び出される。
    ///
    /// # 引数
    /// * `world`: World へのミュータブルな参照 (MutexGuard)
    /// * `moved_entity`: 移動されたカードのエンティティ
    /// * `target_stack_type_for_update`: 移動先のスタックタイプ (World の更新用)
    /// * `target_stack_type_for_proto`: 移動先のスタックタイプ (サーバー通知用のプロトコル型) <- ★ network_handler に渡す ★
    /// * `dragging_info`: ドラッグ開始時の情報 (元の位置など)
    /// * `original_stack_info`: 移動元のスタック情報 (Option)
    fn update_world_and_notify_server(
        &self,
        mut world: std::sync::MutexGuard<'_, World>, // MutexGuard を受け取る
        moved_entity: Entity,
        target_stack_type_for_update: StackType, // World 更新用
        target_stack_type_for_proto: protocol::StackType, // ★ サーバー通知用の型 ★
        dragging_info: &DraggingInfo, // ★ DraggingInfo を参照で受け取る ★
        original_stack_info: Option<StackInfo> // ★ 元のスタック情報を受け取る ★
    ) {
        log(&format!("update_world_and_notify_server called for entity: {:?}, target_stack_type: {:?}", moved_entity, target_stack_type_for_update));

        // --- 1. 移動元スタックのカードを表にする処理 (もし必要なら) ---
        //    StackInfo に cards リストがなくなったので、World を検索する必要がある。
        if let Some(original_info) = original_stack_info {
            // 移動元が Tableau だった場合のみ、下のカードを表にする可能性がある
            if let StackType::Tableau(_) = original_info.stack_type {
                let position_below = dragging_info.original_position_in_stack.saturating_sub(1);
                // 移動したカードの1つ下 (position_below) にカードが存在するか確認
                let mut entity_to_reveal: Option<Entity> = None;
                for entity in world.get_all_entities_with_component::<StackInfo>() {
                    if let Some(stack_info) = world.get_component::<StackInfo>(entity) {
                        // 同じスタックタイプで、位置が1つ下のエンティティを探す
                        if stack_info.stack_type == original_info.stack_type && stack_info.position_in_stack as usize == position_below {
                            entity_to_reveal = Some(entity);
                            break;
                        }
                    }
                }

                // 表にするカードが見つかったら、Card コンポーネントを更新
                if let Some(reveal_entity) = entity_to_reveal {
                    if let Some(mut card) = world.get_component_mut::<Card>(reveal_entity) {
                        if !card.is_face_up {
                            log(&format!("  - Revealing card {:?} in original stack {:?}.", reveal_entity, original_info.stack_type));
                            card.is_face_up = true;
                        }
                    }
                }
            }
            // ★修正: StackType::Deck を StackType::Stock に変更★
            // 移動元が山札(Stock) だった場合、一番上のカードを表にする (これは通常 Waste に移動するので不要かも？ルール次第)
            else if original_info.stack_type == StackType::Stock { 
                 // TODO: Stock から移動した場合の処理（通常 Waste に移動するので、
                 //       Waste 側の処理で吸収されるか、特殊な移動ルールの場合に実装）
                 log("  - Moved from Stock. Handling reveal logic if necessary...");
                 // Stock の一番上のカードを探す処理が必要
            }
        }

        // --- 2. 移動先スタックのエンティティを特定 --- ★find_entity_by_stack_type がないと仮定して一旦コメントアウト★
        // TODO: World に find_entity_by_stack_type メソッドを実装後、以下のコメントアウトを解除する
        // ★修正: コメントアウトを解除！ World にメソッドを追加したからね！★
        let target_stack_entity_opt = world.find_entity_by_stack_type(target_stack_type_for_update);
        // 仮の実装: find_entity_by_stack_type がないので、移動先スタックエンティティの特定はスキップ。StackInfo の更新で対応。
        // ★修正: expect を追加して、見つからなかったらパニックさせる (移動ルールチェック後なので見つかるはず)★
        let target_stack_entity = target_stack_entity_opt.expect("Target stack entity not found despite valid move"); 
        log(&format!("  - Finding target stack entity for type: {:?} -> Found: {:?}", target_stack_type_for_update, target_stack_entity));

        // --- 3. 移動先スタックでの新しい順序 (position_in_stack) を計算 --- ★書き換え★
        //     移動先のスタックタイプを持つカードをすべて検索し、最大の position_in_stack を見つける。
        let mut max_pos_in_target_stack: i16 = -1; // u8 だと 0 の場合があるので i16 で初期化
        for entity in world.get_all_entities_with_component::<StackInfo>() {
            // 自分自身 (moved_entity) は除外して検索
            if entity == moved_entity { continue; }

            if let Some(stack_info) = world.get_component::<StackInfo>(entity) {
                if stack_info.stack_type == target_stack_type_for_update {
                    max_pos_in_target_stack = max_pos_in_target_stack.max(stack_info.position_in_stack as i16);
                }
            }
        }
        // 新しい順序は、見つかった最大値 + 1 (カードがなければ 0)
        let new_position_in_stack = (max_pos_in_target_stack + 1) as u8;
        log(&format!("  - Calculated new position_in_stack for {:?}: {}", target_stack_type_for_update, new_position_in_stack));


        // --- 4. moved_entity の StackInfo コンポーネントを更新 --- ★書き換え★
        //     カードリストを直接いじるのではなく、移動したカード自身の StackInfo を更新！
        if let Some(mut card_stack_info) = world.get_component_mut::<StackInfo>(moved_entity) {
            card_stack_info.stack_type = target_stack_type_for_update; // 新しいスタックタイプ
            card_stack_info.position_in_stack = new_position_in_stack; // 計算した新しい順序
            log(&format!("  - Updated StackInfo for moved entity {:?}: type={:?}, position={}", moved_entity, card_stack_info.stack_type, card_stack_info.position_in_stack));
        } else {
             // 通常、カードには StackInfo があるはずだが、なければ警告
            warn!("  - Warning: StackInfo component not found for moved entity {:?}. Cannot update its stack info.", moved_entity);
            // 移動処理を中断すべきかもしれないので、元の位置に戻す
            self.reset_card_position(world, moved_entity, dragging_info);
            return; // エラーなのでここで処理終了
        }

        // --- 5. 移動したカードの Position コンポーネントを計算・更新 --- (内容はほぼ同じ)
        // 新しいスタックでのカードの位置を計算
        // calculate_card_position が World への参照 (&World) を取るように修正されているか確認！
        let new_position = self.calculate_card_position(target_stack_type_for_update, new_position_in_stack, &world);
        log(&format!("  - Calculated new position for {:?}: {:?}", moved_entity, new_position));
        // 計算した位置をカードの Position コンポーネントに設定
        if let Some(mut pos_comp) = world.get_component_mut::<Position>(moved_entity) {
            *pos_comp = new_position;
            log(&format!("  - Updated Position for moved entity {:?}: {:?}", moved_entity, pos_comp));
        } else {
            error!("  - Error: Position component not found for moved entity {:?}. Cannot update position.", moved_entity);
            // 位置が更新できないのは致命的。StackInfo も元に戻した方が良いかも？
            // とりあえずエラーログのみ。
            // 元の位置に戻す処理はここではなく、reset_card_position を呼ぶべきか検討。
        }

        // --- 6. サーバーに移動完了を通知 --- (内容はほぼ同じ)
        log(&format!("  - Notifying server about the move: entity {:?}, target stack type {:?}", moved_entity, target_stack_type_for_proto));
        // network_handler の send_make_move を呼び出す
        match serde_json::to_string(&target_stack_type_for_proto) {
            Ok(target_stack_json) => {
                // 実際の送信処理は network_handler に任せる
                super::network_handler::send_make_move(
                    &self.network_manager,
                    moved_entity.0, // Entity から usize へ
                    target_stack_json
                );
                log("  - MakeMove message sent to server.");
            }
            Err(e) => {
                // JSON シリアライズ失敗
                error!("  - Error: Failed to serialize target_stack_type_for_proto to JSON: {}", e);
            }
        }

        // MutexGuard はスコープを抜けるときに自動的にドロップ（アンロック）される
    }


    /// カードの位置をドラッグ開始時の元の位置に戻す内部ヘルパー関数。
    /// 移動が無効だった場合や、エラー発生時に呼び出される。
    fn reset_card_position(
        &self,
        mut world: std::sync::MutexGuard<'_, World>, // MutexGuard を受け取る
        entity: Entity,
        dragging_info: &DraggingInfo // 元の位置情報を持つ DraggingInfo
    ) {
        log(&format!("reset_card_position called for entity: {:?}", entity));
        // ★修正: original_position フィールドではなく、original_x, original_y を使う★
        // DraggingInfo に保存されている元の座標 (f64) を Position (f32) に変換
        let original_position = Position {
            x: dragging_info.original_x as f32, // f64 -> f32
            y: dragging_info.original_y as f32, // f64 -> f32
        };
        // ★修正: E0382 エラー回避のため、log を先に実行★
        log(&format!("  - Reset position for entity {:?} to {:?}", entity, original_position));
        // カードの Position コンポーネントを元の値で更新
        if let Some(mut pos_comp) = world.get_component_mut::<Position>(entity) {
            *pos_comp = original_position; // ムーブは log の後
        } else {
            // ★修正: log マクロに引数を追加★
            error!("  - Error: Position component not found for entity {:?}. Cannot reset position.", entity);
        }
        // DraggingInfo コンポーネントは、この関数を呼び出す前に既に削除されているはずなので、ここでは何もしない。
    }

    // --- スタックの種類とスタック内での位置に基づいて、カードの描画位置 (Position) を計算するヘルパー関数 ---
    // (ソリティアのレイアウトに合わせて調整が必要)
    fn calculate_card_position(&self, stack_type: StackType, position_in_stack: u8, world: &World) -> Position {
        // position_in_stack は u8 だけど、計算には f32 を使うからキャストするよ！
        let pos_in_stack_f32 = position_in_stack as f32;

        // スタックタイプに応じて基準となる X, Y 座標とオフセットを計算！
        let (base_x, base_y) = match stack_type {
            StackType::Stock => {
                // 山札 (Stock) は常に同じ位置。重ならない。
                (layout::STOCK_POS_X, layout::STOCK_POS_Y)
            }
            StackType::Waste => {
                // 捨て札 (Waste) も基本同じ位置だけど、クロンダイクのルールによっては
                // 3枚ずつめくって重ねて表示する場合がある。
                // 今は単純に1箇所に重ねる想定で、Stock の隣の位置にするよ。
                // TODO: Waste の重なり表示ルールをちゃんと実装するなら、ここを修正！
                (layout::WASTE_POS_X, layout::WASTE_POS_Y)
            }
            StackType::Foundation(index) => {
                // 組札 (Foundation) は、インデックス (0-3) に基づいて横に並ぶ。
                // X座標 = 開始位置 + インデックス * 横オフセット
                let x = layout::FOUNDATION_START_X + (index as f32) * layout::FOUNDATION_X_OFFSET;
                // Y座標は開始位置と同じ。
                let y = layout::FOUNDATION_START_Y;
                (x, y)
            }
            StackType::Tableau(index) => {
                // 場札 (Tableau) は、インデックス (0-6) で横の列が決まる。
                let base_x = layout::TABLEAU_START_X + (index as f32) * layout::TABLEAU_X_OFFSET;
                // Y座標は、その列に既に積まれているカードによって決まる。
                // 基本のY座標 + 表向き/裏向きに応じたオフセット * スタック内の位置
                // ここで、そのスタックの他のカードを見て、表向きか裏向きか判断する必要がある。
                // ちょっと複雑なので、簡略化して「常に表向きオフセットを使う」としてみる。
                // TODO: position_in_stack より前のカードが裏向きかどうかをチェックしてオフセットを計算するロジックを追加する。
                let mut current_y = layout::TABLEAU_START_Y;
                // このスタックのカードを取得してソートする (仮)
                let mut cards_in_this_tableau: Vec<(Entity, StackInfo)> = Vec::new();
                for entity in world.get_all_entities_with_component::<StackInfo>() {
                    if let Some(info) = world.get_component::<StackInfo>(entity) {
                        if info.stack_type == stack_type {
                            cards_in_this_tableau.push((entity, info.clone()));
                        }
                    }
                }
                // position_in_stack でソート (昇順)
                cards_in_this_tableau.sort_by_key(|(_, info)| info.position_in_stack);

                // 0 から position_in_stack - 1 までのカードを見て Y オフセットを累積
                for i in 0..position_in_stack {
                    // Entity ID を取得 (インデックス i がリストの範囲内かチェックが必要だが省略)
                    let card_entity = cards_in_this_tableau[i as usize].0;
                    // そのカードが表向きか取得
                    let is_face_up = world.get_component::<Card>(card_entity)
                                        .map_or(false, |c| c.is_face_up);
                    if is_face_up {
                        current_y += layout::TABLEAU_Y_OFFSET_FACE_UP;
                    } else {
                        current_y += layout::TABLEAU_Y_OFFSET_FACE_DOWN;
                    }
                }
                (base_x, current_y)
            }
            StackType::Hand => {
                // 手札 (Hand) の座標計算。今は仮に左下に置くことにする。
                // TODO: プレイヤーごとに手札の表示位置を決めるロジックが必要。
                (50.0, 600.0 + pos_in_stack_f32 * layout::TABLEAU_Y_OFFSET_FACE_UP) // 適当な座標 + 重なり
            }
        };

        // 計算結果を Position 型にして返す！
        Position { x: base_x, y: base_y }
    }

    /// JavaScript から呼び出される、ドラッグ中のカードの位置を一時的に更新するためのメソッドだよ！
    /// マウスの動きに合わせてカードの見た目を追従させるために使うんだ。
    /// ⚠️ 注意: この関数は表示上の Position を更新するだけで、
    ///         カードの所属スタック (StackInfo) やゲームの論理的な状態は変更しないよ！
    ///         最終的な移動処理は handle_drag_end で行われる。
    #[wasm_bindgen]
    pub fn update_dragged_position(&mut self, entity_id: usize, mouse_x: f32, mouse_y: f32) {
        // デバッグ用に、どのエンティティがどの座標に更新されようとしているかログ出力！
        // console::log_3(&JsValue::from_str("update_dragged_position: entity="), &JsValue::from(entity_id), &JsValue::from(format!("mouse=({}, {})", mouse_x, mouse_y)));

        let entity = Entity(entity_id);

        // World のロックを取得 (Position と DraggingInfo を読み書きするから可変で)
        let mut world_guard = match self.world.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                // ロック失敗！エラーログを出して何もしない。
                log::error!("Failed to lock world in update_dragged_position: {}", e);
                return;
            }
        };

        // --- ドラッグ情報 (オフセット) を取得 --- //
        // ドラッグされているカードの DraggingInfo コンポーネントを取得する。
        // これには、ドラッグ開始時のマウスカーソルとカード左上のズレ (オフセット) が記録されてるはず！
        let dragging_info_opt = world_guard.get_component::<DraggingInfo>(entity);

        if let Some(dragging_info) = dragging_info_opt {
            // DraggingInfo が見つかった！ オフセットを使ってカードの新しい左上座標を計算するよ。
            // カードの左上 X = マウスの X - オフセット X
            // カードの左上 Y = マウスの Y - オフセット Y
            // DraggingInfo のオフセットは f64 だけど、Position は f32 なのでキャストが必要だよ！
            let new_card_x = mouse_x - dragging_info.offset_x as f32;
            let new_card_y = mouse_y - dragging_info.offset_y as f32;

            // --- Position コンポーネントを更新 --- //
            // 移動させるカードの Position コンポーネントを可変 (mut) で取得する。
            if let Some(position_component) = world_guard.get_component_mut::<Position>(entity) {
                // Position コンポーネントの x と y を、計算した新しい座標で上書き！
                position_component.x = new_card_x;
                position_component.y = new_card_y;
                // ログで更新後の座標を確認！ (コメントアウトしてもOK)
                // log::info!("  Updated dragged Position for {:?} to ({}, {})", entity, new_card_x, new_card_y);
            } else {
                // Position コンポーネントが見つからないのはおかしい… エラーログ！
                // ★修正: log マクロに引数を追加★
                log::error!("  Failed to get Position component for dragged entity {:?} during update", entity);
            }
        } else {
            // DraggingInfo が見つからないってことは、もうドラッグが終わってるか、何かおかしい。
            // エラーログを出しておく。
            // ★修正: log マクロに引数を追加★
            log::error!("  DraggingInfo component not found for entity {:?} in update_dragged_position", entity);
            // この場合、位置の更新は行わない。
        }

        // World のロックはこの関数のスコープを抜けるときに自動的に解除されるよ。
        // drop(world_guard); // 明示的に書いてもOK！
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