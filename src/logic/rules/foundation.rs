//! 組札 (Foundation) へのカード移動ルールを定義するよ。

use crate::components::card::{Card, Rank};
use crate::components::stack::StackType;
use crate::ecs::entity::Entity;
use crate::ecs::world::World;
// 共通ヘルパーを使うためにインポート
use super::common::{get_foundation_suit, get_top_card_entity};
// console::log を使うためにインポート
// use wasm_bindgen::JsValue;
// use web_sys::console;
// ★ log マクロを使うためにインポート ★
use crate::log; 

/// 指定されたカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
pub fn can_move_to_foundation(
    world: &World,
    card_to_move_entity: Entity,
    target_foundation_index: u8,
) -> bool {
    // ★ 追加: 関数の開始ログ ★
    log(&format!("[Foundation Rule] Checking move: {:?} to Foundation({})", card_to_move_entity, target_foundation_index));

    let card_to_move = match world.get_component::<Card>(card_to_move_entity) {
        Some(card) => card,
        None => {
            // console::log_1(&JsValue::from_str(&format!("[Rules Error] 移動元エンティティ {:?} に Card コンポーネントが見つかりません！", card_to_move_entity)));
            log(&format!("[Foundation Rule Error] No Card component found for {:?}!", card_to_move_entity)); // ★ log に変更 ★
            return false;
        }
    };
    // ★ 追加: 移動元カード情報ログ ★
    log(&format!("[Foundation Rule]  - Card to move: {:?} {:?}", card_to_move.rank, card_to_move.suit));

    let target_suit = match get_foundation_suit(target_foundation_index) {
        Some(suit) => suit,
        None => {
            // console::log_1(&JsValue::from_str(&format!("[Rules Error] 無効な Foundation インデックス {} が指定されました！", target_foundation_index)));
            log(&format!("[Foundation Rule Error] Invalid Foundation index: {}!", target_foundation_index)); // ★ log に変更 ★
            return false;
        }
    };
    // ★ 追加: ターゲットスート情報ログ ★
    log(&format!("[Foundation Rule]  - Target suit for Foundation({}): {:?}", target_foundation_index, target_suit));

    // ★ 追加: スートチェック前のログ ★
    log(&format!("[Foundation Rule]  - Checking suit match... (Card: {:?}, Target: {:?})", card_to_move.suit, target_suit));
    if card_to_move.suit != target_suit {
        // ★ 追加: スート不一致ログ ★
        log("[Foundation Rule]  - Result: Suit mismatch! Move invalid.");
        return false;
    }
    // ★ 追加: スート一致ログ ★
    log("[Foundation Rule]  - Result: Suit matches.");

    let target_stack_type = StackType::Foundation(target_foundation_index);
    // ★ 追加: トップカード取得前のログ ★
    log(&format!("[Foundation Rule]  - Checking top card of target stack: {:?}...", target_stack_type));
    let target_top_card_entity_option = get_top_card_entity(world, target_stack_type);

    let result = match target_top_card_entity_option {
        None => {
            // ★ 追加: ターゲット空ログ ★
            log("[Foundation Rule]  - Target foundation is empty.");
            let is_ace = card_to_move.rank == Rank::Ace;
            // ★ 追加: Ace チェック結果ログ ★
            log(&format!("[Foundation Rule]  - Checking if card is Ace... Result: {}", is_ace));
            is_ace
        }
        Some(target_top_card_entity) => {
            // ★ 追加: ターゲットにカードありログ ★
            log(&format!("[Foundation Rule]  - Target foundation has top card: {:?}", target_top_card_entity));
            let target_top_card = match world.get_component::<Card>(target_top_card_entity) {
                Some(card) => card,
                None => {
                    // console::log_1(&JsValue::from_str(&format!("[Rules Error] 移動先トップエンティティ {:?} に Card コンポーネントが見つかりません！", target_top_card_entity)));
                    log(&format!("[Foundation Rule Error] No Card component found for top entity {:?}!", target_top_card_entity)); // ★ log に変更 ★
                    return false; // ★早期リターン時のログ抜け防止のため、ここでは false を直接返さず、下の最終ログで返す★
                    // return false;
                }
            };
            // ★ 追加: ランクチェック前のログ ★
            log(&format!("[Foundation Rule]  - Checking rank sequence... (Card: {:?}, Top card: {:?})", card_to_move.rank, target_top_card.rank));
            let is_next_rank = (card_to_move.rank as usize) == (target_top_card.rank as usize) + 1;
            // ★ 追加: ランクチェック結果ログ ★
            log(&format!("[Foundation Rule]  - Checking rank sequence... Result: {}", is_next_rank));
            is_next_rank
        }
    };

    // ★ 追加: 最終結果ログ ★
    log(&format!("[Foundation Rule] Final result for move {:?} to Foundation({}): {}", card_to_move_entity, target_foundation_index, result));
    result
} 