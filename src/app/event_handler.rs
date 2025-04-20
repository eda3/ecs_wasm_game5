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
/// ★ 修正: ドラッグ中のエンティティを無視するための引数を追加 ★
pub fn find_clicked_element(world: &World, x: f32, y: f32, dragged_entity: Option<Entity>) -> Option<ClickTarget> {
    // ★ 修正: カードの判定を先に行う ★
    // ★ 修正: dragged_entity を find_topmost_clicked_card に渡す ★
    let card_target = find_topmost_clicked_card(world, x, y, dragged_entity);
    if card_target.is_some() {
        // カードが見つかればそれを返す
        return card_target;
    }
    // カードが見つからなければ、背景のスタックエリアを探す
    // (スタックエリア判定ではドラッグ中エンティティは無視不要)
    find_clicked_stack_area(world, x, y)
}

/// 指定された座標 (x, y) にある、最も手前 (y 座標が最大) のクリック可能な要素 (カード) を探す。
/// 重なりを考慮し、一番上の要素のみを返す。
/// ★ 修正: ドラッグ中のエンティティを無視するための引数を追加 ★
pub fn find_topmost_clicked_card(world: &World, x: f32, y: f32, dragged_entity_to_ignore: Option<Entity>) -> Option<ClickTarget> {
    // ★★★ 関数の中身を y 座標で比較する元のロジックに戻す ★★★
    log("  Checking for clicked cards...");
    // ★ 追加: 無視するエンティティのログ ★
    if let Some(ignore_entity) = dragged_entity_to_ignore {
        log(&format!("    (Ignoring dragged entity: {:?})", ignore_entity));
    }

    let position_entities = world.get_all_entities_with_component::<Position>();
    if position_entities.is_empty() {
        return None;
    }

    // 2. Position持ちエンティティをフィルタリング & マッピング
    let clicked_cards_iter = position_entities
        .into_iter()
        .filter_map(|entity| { // 条件を満たさないものは None を返して除外
            // ★★★ 追加: ドラッグ中のエンティティを無視する ★★★
            if let Some(ignore_entity) = dragged_entity_to_ignore {
                if entity == ignore_entity {
                    return None; // Skip the dragged entity
                }
            }
            // ★★★ ここまで追加 ★★★

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
pub fn find_clicked_stack_area(_world: &World, _x: f32, _y: f32) -> Option<ClickTarget> {
    // TODO: 各スタックタイプの領域を計算し、(x, y) が含まれるかチェックするロジックを実装
    //       現状は仮実装として常に None を返す
    log("  Checking for clicked stack area...");
    // ここにスタック領域判定のロジックが入るはず...
    None // 仮実装: どのスタックにもヒットしなかったことにする
}