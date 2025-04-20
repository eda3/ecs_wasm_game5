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

// ★追加: state_getter モジュールを use する★
use super::state_getter;

// ★追加: drag_handler モジュールを use する★
use super::drag_handler;

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
    /// 内部で `state_getter::get_world_state_json` を呼び出すだけだよ！
    #[wasm_bindgen]
    pub fn get_world_state_json(&self) -> Result<JsValue, JsValue> { // ★戻り値を JsValue に変更★
        println!("GameApp: get_world_state_json が呼ばれました。World の状態を準備中...");

        // ★ state_getter モジュールの関数を呼び出す！★
        // self.world は Arc<Mutex<World>> なので、そのまま参照を渡せるよ！
        state_getter::get_world_state_json(&self.world)
        // 返り値は既に Result<JsValue, JsValue> なので、そのまま返す！
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
    pub fn handle_click(&mut self, x: f32, y: f32) {
        // --- クリック要素の特定 (event_handler に委譲) --- 
        // World は handle_click_logic 内でロックされるのでここでは不要

        // ★ 修正: クリック要素を特定するだけに変更 ★
        let clicked_target = {
            let world = match self.world.lock() {
                Ok(w) => w,
                Err(e) => {
                    error!("handle_click 内で World のロックに失敗: {}", e);
                    return;
                }
            };
            event_handler::find_clicked_element(&world, x, y)
            // world のロックはここでドロップされる
        };

        // --- クリック要素に応じた処理 --- 
        match clicked_target {
            Some(ClickTarget::Card(entity)) => {
                log(&format!("カード {:?} をクリック -> ドラッグ開始処理へ", entity));
                // ★ カードがクリックされたらドラッグ開始処理を呼ぶ！★
                self.handle_drag_start(entity.0, x, y);
            }
            Some(ClickTarget::Stack(stack_type)) => {
                log(&format!("スタックエリア {:?} をクリック -> スタックアクション処理へ", stack_type));
                // ★ スタッククリック時のロジックは event_handler に移譲する ★
                //   (ただし、サーバー通信などは network_handler 経由で行うべき)
                // TODO: event_handler にスタッククリック処理を実装し、それを呼ぶ
                // event_handler::handle_stack_click_logic(&self.world, &self.network_manager, stack_type);
                match stack_type {
                    StackType::Stock => log("  山札クリック！ (処理は TODO)"),
                    StackType::Waste => log("  捨て札クリック！ (処理は TODO)"),
                    _ => log("  他のスタッククリック (処理は TODO)"),
                }
            }
            None => {
                log("空きスペースをクリック。");
                // 何もしない
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
    pub fn handle_drag_start(&mut self, entity_usize: usize, start_x: f32, start_y: f32) {
        // ★ drag_handler モジュールの関数を呼び出す！★
        // self.world は Arc<Mutex<World>> なので、参照を渡す
        drag_handler::handle_drag_start(&self.world, entity_usize, start_x, start_y);
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
                            // ★ 修正: drag_handler の関数を呼び出す ★
                            drag_handler::update_world_and_notify_server(
                                world, // MutexGuard を渡す (self.world ではない)
                                &self.network_manager, // NetworkManager の参照を渡す
                                entity,
                                target_stack_type, // World 更新には StackType を渡す
                                target_stack_type_for_proto,
                                &dragging_info,
                                original_stack_info
                            );
                        } else {
                            // --- 3a-iii. 移動ルール NG の場合 ---
                            log("  - Move is invalid. Resetting card position.");
                            // ★ 修正: drag_handler の関数を呼び出す ★
                            drag_handler::reset_card_position(world, entity, &dragging_info);
                        }
                    } else {
                        // target_stack_entity は見つかったが、StackInfo が取得できなかった場合 (通常はありえない)
                        error(&format!("  - Error: StackInfo not found for target stack entity {:?}. Resetting card position.", target_stack_entity));
                        // ★ 修正: drag_handler の関数を呼び出す ★
                        drag_handler::reset_card_position(world, entity, &dragging_info);
                    }
                } else {
                    // target_stack_type に対応する Entity が見つからなかった場合 (通常はありえない)
                    error!("  - Error: Stack entity not found for type {:?}. Resetting card position.", target_stack_type);
                    // ★ 修正: drag_handler の関数を呼び出す ★
                    drag_handler::reset_card_position(world, entity, &dragging_info);
                }
            }
            // --- 3b. ドロップ先がカードだった場合 (今はスタックへのドロップのみ想定) ---
            Some(ClickTarget::Card(target_card_entity)) => {
                log(&format!("  - Target is a card ({:?}). Invalid drop target. Resetting card position.", target_card_entity));
                // カードの上は無効なドロップ先として扱う
                // ★ 修正: drag_handler の関数を呼び出す ★
                drag_handler::reset_card_position(world, entity, &dragging_info);
            }
            // --- 3c. ドロップ先が空の領域だった場合 ---
            None => {
                log("  - Target is empty space. Resetting card position.");
                // 何もない場所へのドロップも無効
                // ★ 修正: drag_handler の関数を呼び出す ★
                drag_handler::reset_card_position(world, entity, &dragging_info);
            }
        }

        // ドラッグ終了処理が終わったら、Window のリスナーを解除する
        // (成功時も失敗時も解除する)
        *self.window_mousemove_closure.lock().unwrap() = None;
        *self.window_mouseup_closure.lock().unwrap() = None;
        log("  - Removed window mousemove and mouseup listeners.");

    } // handle_drag_end の終わり

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