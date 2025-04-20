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
use crate::app::event_handler; // event_handler モジュールを use する！
use crate::{log, error}; // log と error マクロをインポート (lib.rs から)
// use crate::ecs::entity::Entity; // 未使用
// use crate::app::init_handler; // 未使用 (super:: で直接呼ぶため)
// use crate::app::network_handler; // 未使用 (super:: で直接呼ぶため)
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
        let dragging_state_arc = Arc::new(Mutex::new(None));
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
}

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