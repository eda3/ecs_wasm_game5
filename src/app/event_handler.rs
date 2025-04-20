// src/app/event_handler.rs
//! ユーザー入力やUIイベントに関連する GameApp のロジック。

use std::sync::{Arc, Mutex};
use crate::ecs::world::World;
use crate::network::NetworkManager;
use crate::ecs::entity::Entity;
use crate::components::card::Card;
use crate::components::stack::StackType; // StackType も使うから use するよ！
use crate::components::position::Position; // Position も使うから use するよ！
use crate::config::layout; // レイアウト情報も使う！
use crate::app::renderer::{RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT}; // カードのサイズを Renderer から持ってくる！
use crate::logic::auto_move::find_automatic_foundation_move;
use crate::protocol::{self, ClientMessage}; // protocol モジュールと ClientMessage をインポート
use crate::{log, error}; // log と error マクロをインポート (lib.rs から)
use serde_json;
// use itertools::Itertools; // ★ max_by を使うので不要になった ★
// use crate::app::AppEvent; // ★ AppEvent が見つからないため一旦コメントアウト
// use crate::components::dragging_info::DraggingInfo; // 現状未使用
// use web_sys::MouseEvent; // 現状未使用
// use wasm_bindgen::JsValue; // 現状未使用
// use web_sys::console; // 現状未使用

/// ダブルクリック時の実際のロジック (lib.rs の GameApp::handle_double_click_logic から移動)
pub fn handle_double_click_logic(
    entity_id: usize,
    world_arc: Arc<Mutex<World>>,
    network_manager_arc: Arc<Mutex<NetworkManager>>
) {
    log(&format!("  Executing double-click logic for entity_id: {}", entity_id));
    let entity = Entity(entity_id);

    // World をロックして、必要な情報を取得
    let world_guard = match world_arc.lock() {
        Ok(w) => w,
        Err(e) => {
            error(&format!("Error locking world in handle_double_click_logic: {}", e));
            return;
        }
    };

    // ダブルクリックされたカードを取得
    let card_to_move = match world_guard.get_component::<Card>(entity) {
        Some(card) => card.clone(), // Clone する!
        None => {
            error(&format!("Card component not found for entity {:?} in handle_double_click_logic", entity));
            return;
        }
    };

    // 自動移動先を探す！🔍
    // find_automatic_foundation_move 関数を呼び出して、指定されたカードエンティティ (entity) が
    // 自動的に移動できる Foundation があるか探す。
    // 引数には World の参照 (`&*world_guard`) とカードの Entity ID (`entity`) を渡すよ！
    let target_stack_opt = find_automatic_foundation_move(&*world_guard, entity);
    // World のロックを早めに解除！ これ以降 World の状態は読み書きできないけど、
    // ロック時間が短くなって、他の処理をブロックする可能性が減るんだ。👍
    drop(world_guard);

    match target_stack_opt {
        Some(target_stack) => {
            // 移動先が見つかった！🎉 MakeMove メッセージを送信！🚀
            log(&format!("  Found automatic move target: {:?} for card {:?}", target_stack, card_to_move));
            // components::stack::StackType を protocol::StackType に変換
            let protocol_target_stack: protocol::StackType = target_stack.into(); // From トレイトを実装済みと仮定 (protocol.rsで実装必要かも)
            let message = ClientMessage::MakeMove { moved_entity: entity, target_stack: protocol_target_stack };

            // メッセージ送信
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

// TODO: ドラッグ開始、ドラッグ中、ドラッグ終了のイベントハンドラロジックもここに移動する 

// --- クリック判定ロジック ---

/// クリックされた要素の種類を表す Enum だよ！
/// カードがクリックされたのか、それともスタックの空きスペースがクリックされたのかを示すんだ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // デバッグ表示、コピー、比較ができるようにするおまじない✨
pub enum ClickTarget {
    /// カードがクリックされた場合。どのカードか (Entity) を保持するよ。
    Card(Entity),
    /// スタックの空きエリアがクリックされた場合。どの種類のスタックか (StackType) を保持するよ。
    Stack(StackType),
}

/// 指定された座標 (x, y) にあるクリック可能な要素 (カード or スタックエリア)
/// を探して返す。
/// 一番手前にある要素が見つかる。
pub fn find_clicked_element(world: &World, x: f32, y: f32) -> Option<ClickTarget> {
    // ★ 修正: カードの判定を先に行うように戻す！ ★
    let card_target = find_topmost_clicked_card(world, x, y);
    if card_target.is_some() {
        // カードが見つかればそれを返す
        return card_target;
    }
    // カードが見つからなければ、背景のスタックエリアを探す
    find_clicked_stack_area(world, x, y)

    // --- スタック優先だったコード (削除) ---
    // if let Some(stack_target) = find_clicked_stack_area(world, x, y) {
    //     // スタックエリアが見つかったら、それを優先して返す！
    //     return Some(stack_target);
    // }
    // find_topmost_clicked_card(world, x, y)
}

/// 指定された座標 (x, y) にある、最も手前 (y 座標が最大) のクリック可能な要素 (カード) を探す。
/// 重なりを考慮し、一番上の要素のみを返す。
pub fn find_topmost_clicked_card(world: &World, x: f32, y: f32) -> Option<ClickTarget> {
    // ★★★ 関数の中身を y 座標で比較する元のロジックに戻す ★★★
    log("  Checking for clicked cards...");

    let position_entities = world.get_all_entities_with_component::<Position>();
    if position_entities.is_empty() {
        return None;
    }

    // 2. Position持ちエンティティをフィルタリング & マッピング
    //    - Card も持っているか？
    //    - クリック範囲内か？
    //    => 条件を満たす (Entity, y_pos: f32) のイテレータを作る
    let clicked_cards_iter = position_entities
        .into_iter()
        .filter_map(|entity| { // 条件を満たさないものは None を返して除外
            if world.get_component::<Card>(entity).is_some() {
                 let pos = world.get_component::<Position>(entity).unwrap();

                 let card_left = pos.x;
                 let card_top = pos.y;
                 let card_right = card_left + RENDER_CARD_WIDTH as f32;
                 let card_bottom = card_top + RENDER_CARD_HEIGHT as f32;

                 let is_inside = x >= card_left && x < card_right && y >= card_top && y < card_bottom;

                 if is_inside {
                     // ヒット！ このカードの Entity と Y 座標 (手前判定用) を返す
                     Some((entity, pos.y))
                 } else {
                     None // クリック範囲外
                 }
            } else {
                None // Card コンポーネントがない
            }
        });

    // 3. クリックされたカードの中から、Y座標が最大のものを探す！
    //    max_by はイテレータを消費して Option<(Entity, f32)> を返す
    //    ★ 型アノテーションを追加して E0282 を解消 ★
    let topmost_card = clicked_cards_iter
        .max_by(|(_entity1, y1): &(Entity, f32), (_entity2, y2): &(Entity, f32)| {
            // f32 の比較は total_cmp を使うのが Rust では推奨！
            y1.total_cmp(y2)
        });

    // 4. 結果を Option<ClickTarget> に変換して返す
    match topmost_card {
        Some((entity, _y_pos)) => {
            log(&format!("  Topmost clicked card found: {:?}", entity));
            Some(ClickTarget::Card(entity)) // 正しく ClickTarget でラップして返す
        }
        None => {
            log("  No card found at the clicked position (matching criteria).");
            None
        }
    }
    // ★★★ ここまで修正 ★★★
}

/// クリックされた座標 (x, y) がスタックの表示エリア内にあるか判定するヘルパー関数だよ。
/// カードがクリックされなかった場合に呼び出されることを想定しているよ。
///
/// # 引数
/// * `_world`: ゲーム世界の現在の状態 (`World`)。(現状未使用だけど、将来的に使うかも？例えば空のスタックのみ判定対象にするとか)
/// * `x`: クリックされた画面上の X 座標。
/// * `y`: クリックされた画面上の Y 座標。
///
/// # 戻り値
/// * `Option<ClickTarget>`:
///   - `Some(ClickTarget::Stack(stack_type))` : クリック座標がいずれかのスタックエリア内にあった場合。
///   - `None`: どのスタックエリアにもヒットしなかった場合。
///
/// # 実装詳細
/// 1. 各スタックタイプ (`Stock`, `Waste`, `Foundation` 0-3, `Tableau` 0-6) の基本的な表示領域（通常はカード1枚分のサイズ）を計算する。
///    - `src/config/layout.rs` の定数とカードサイズ (`RENDER_CARD_WIDTH`, `RENDER_CARD_HEIGHT`) を使うよ。
/// 2. 順番に各スタックの領域をチェックし、クリック座標 `(x, y)` が領域内に含まれていれば、
///    対応する `StackType` を `ClickTarget::Stack` でラップして `Some` で返す。最初に見つかった時点で終了！
/// 3. 全てのスタックエリアをチェックしてもヒットしなかった場合は `None` を返す。
///
/// # 注意点
/// - この関数は `find_topmost_clicked_card` の後に呼ばれる前提だよ。
/// - そのため、ここでの判定は「カード以外のスタックのพื้นฐาน的な場所」をクリックしたかどうかのチェックが主になるよ。
/// - (将来的に) Tableau などで、カードがある場合でも一番下の空きスペースをクリックしたい、みたいな細かい制御が必要なら、`world` の情報を使ってさらに判定を絞り込む必要があるかもね！
fn find_clicked_stack_area(_world: &World, x: f32, y: f32) -> Option<ClickTarget> {
    log("  Checking for clicked stack areas..."); // デバッグログ！

    // --- 各スタックエリアの判定 ---

    // Helper: 座標が矩形内にあるかチェックする関数
    // 矩形は (左上のX, 左上のY, 幅, 高さ) のタプルで表現するよ。
    fn is_point_in_rect(px: f32, py: f32, rect: (f32, f32, f32, f32)) -> bool {
        let (rx, ry, rw, rh) = rect;
        px >= rx && px < rx + rw && py >= ry && py < ry + rh
    }

    // 1. Stock エリアのチェック
    let stock_rect = (layout::STOCK_POS_X, layout::STOCK_POS_Y, RENDER_CARD_WIDTH as f32, RENDER_CARD_HEIGHT as f32);
    if is_point_in_rect(x, y, stock_rect) {
        log("    Hit Stock area.");
        return Some(ClickTarget::Stack(StackType::Stock));
    }

    // 2. Waste エリアのチェック
    let waste_rect = (layout::WASTE_POS_X, layout::WASTE_POS_Y, RENDER_CARD_WIDTH as f32, RENDER_CARD_HEIGHT as f32);
     if is_point_in_rect(x, y, waste_rect) {
        log("    Hit Waste area.");
        return Some(ClickTarget::Stack(StackType::Waste));
    }

    // 3. Foundation エリアのチェック (4箇所)
    for i in 0..4 {
        let foundation_x = layout::FOUNDATION_START_X + i as f32 * layout::FOUNDATION_X_OFFSET;
        let foundation_y = layout::FOUNDATION_START_Y;
        let foundation_rect = (foundation_x, foundation_y, RENDER_CARD_WIDTH as f32, RENDER_CARD_HEIGHT as f32);
        if is_point_in_rect(x, y, foundation_rect) {
            log(&format!("    Hit Foundation {} area.", i));
            // Foundation のインデックス (0-3) を StackType に含める
            return Some(ClickTarget::Stack(StackType::Foundation(i as u8))); // u8 にキャスト
        }
    }

    // 4. Tableau エリアのチェック (7箇所) - ここではスタックのベース位置のみチェック
    // カード自体へのクリックは find_topmost_clicked_card で処理済みの前提
    for i in 0..7 {
        let tableau_x = layout::TABLEAU_START_X + i as f32 * layout::TABLEAU_X_OFFSET;
        let tableau_y = layout::TABLEAU_START_Y; // ベースのY座標
        let tableau_rect = (tableau_x, tableau_y, RENDER_CARD_WIDTH as f32, RENDER_CARD_HEIGHT as f32);
        if is_point_in_rect(x, y, tableau_rect) {
             log(&format!("    Hit Tableau {} base area.", i));
             // Tableau のインデックス (0-6) を StackType に含める
             return Some(ClickTarget::Stack(StackType::Tableau(i as u8))); // u8 にキャスト
        }
    }

    // 5. どのスタックエリアにもヒットしなかった場合
    log("  No stack area found at the clicked position.");
    None
}

// ここにクリックされた座標からカードやスタックを特定するロジックを書いていくよ！
// 乞うご期待！ ✨ 