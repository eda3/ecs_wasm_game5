//! 場札 (Tableau) へのカード移動ルールを定義するよ。

use crate::components::card::{Card, Rank};
use crate::components::stack::StackType;
use crate::ecs::entity::Entity;
use crate::ecs::world::World;
// 共通ヘルパーを使うためにインポート
use super::common::{CardColor, get_top_card_entity};
// console::log を使うためにインポート
use wasm_bindgen::JsValue;
use web_sys::console;

/// 指定されたカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
pub fn can_move_to_tableau(
    world: &World,
    card_to_move_entity: Entity,
    target_tableau_index: u8,
) -> bool {
    let card_to_move = match world.get_component::<Card>(card_to_move_entity) {
        Some(card) => card,
        None => {
            console::log_1(&JsValue::from_str(&format!("[Rules Error] 移動元エンティティ {:?} に Card コンポーネントが見つかりません！", card_to_move_entity)));
            return false;
        }
    };

    let target_stack_type = StackType::Tableau(target_tableau_index);
    let target_top_card_entity_option = get_top_card_entity(world, target_stack_type);

    match target_top_card_entity_option {
        Some(target_top_card_entity) => {
            let target_top_card = match world.get_component::<Card>(target_top_card_entity) {
                Some(card) => card,
                None => {
                    console::log_1(&JsValue::from_str(&format!("[Rules Error] 移動先トップエンティティ {:?} に Card コンポーネントが見つかりません！", target_top_card_entity)));
                    return false;
                }
            };

            let move_rank = card_to_move.rank;
            let move_suit = card_to_move.suit;
            let move_color = CardColor::from_suit(move_suit);
            let target_rank = target_top_card.rank;
            let target_suit = target_top_card.suit;
            let target_color = CardColor::from_suit(target_suit);

            let colors_different = move_color != target_color;
            let rank_is_one_less = (move_rank as usize) == (target_rank as usize).saturating_sub(1);

            console::log_1(&JsValue::from_str(&format!(
                "    [Rule Check] Moving {:?}({:?}) onto {:?}({:?}). Colors different: {}. Rank is one less: {}.",
                move_rank, move_color, target_rank, target_color, colors_different, rank_is_one_less
            )));

            if !colors_different || !rank_is_one_less {
                console::log_1(&JsValue::from_str("      -> Move invalid based on rank/color."));
                return false;
            }
            console::log_1(&JsValue::from_str("      -> Move valid based on rank/color."));
            true
        }
        None => {
            let move_rank = card_to_move.rank;
            let is_king = move_rank == Rank::King;
            console::log_1(&JsValue::from_str(&format!(
                "    [Rule Check] Moving {:?} onto empty Tableau. Is King: {}.",
                move_rank, is_king
            )));
            is_king
        }
    }
} 