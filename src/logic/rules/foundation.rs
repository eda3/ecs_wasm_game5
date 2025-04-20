//! 組札 (Foundation) へのカード移動ルールを定義するよ。

use crate::components::card::{Card, Rank};
use crate::components::stack::StackType;
use crate::ecs::entity::Entity;
use crate::ecs::world::World;
// 共通ヘルパーを使うためにインポート
use super::common::{get_foundation_suit, get_top_card_entity};
// console::log を使うためにインポート
use wasm_bindgen::JsValue;
use web_sys::console;

/// 指定されたカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
pub fn can_move_to_foundation(
    world: &World,
    card_to_move_entity: Entity,
    target_foundation_index: u8,
) -> bool {
    let card_to_move = match world.get_component::<Card>(card_to_move_entity) {
        Some(card) => card,
        None => {
            console::log_1(&JsValue::from_str(&format!("[Rules Error] 移動元エンティティ {:?} に Card コンポーネントが見つかりません！", card_to_move_entity)));
            return false;
        }
    };

    let target_suit = match get_foundation_suit(target_foundation_index) {
        Some(suit) => suit,
        None => {
            console::log_1(&JsValue::from_str(&format!("[Rules Error] 無効な Foundation インデックス {} が指定されました！", target_foundation_index)));
            return false;
        }
    };

    if card_to_move.suit != target_suit {
        return false;
    }

    let target_stack_type = StackType::Foundation(target_foundation_index);
    let target_top_card_entity_option = get_top_card_entity(world, target_stack_type);

    match target_top_card_entity_option {
        None => {
            card_to_move.rank == Rank::Ace
        }
        Some(target_top_card_entity) => {
            let target_top_card = match world.get_component::<Card>(target_top_card_entity) {
                Some(card) => card,
                None => {
                    console::log_1(&JsValue::from_str(&format!("[Rules Error] 移動先トップエンティティ {:?} に Card コンポーネントが見つかりません！", target_top_card_entity)));
                    return false;
                }
            };
            (card_to_move.rank as usize) == (target_top_card.rank as usize) + 1
        }
    }
} 