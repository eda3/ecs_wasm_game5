//! ルール判定で共通して使うヘルパー関数や型を置くよ。

use crate::components::card::{Suit, Card}; // Card を使う
use crate::components::stack::{StackType, StackInfo}; // StackInfo を使う
use crate::ecs::entity::Entity;
use crate::ecs::world::World;

/// カードの色（赤か黒か）を表すヘルパーenumだよ。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CardColor {
    Red,
    Black,
}

impl CardColor {
    /// スートからカードの色を取得する関数。
    pub fn from_suit(suit: Suit) -> Self {
        match suit {
            Suit::Heart | Suit::Diamond => CardColor::Red,
            Suit::Club | Suit::Spade => CardColor::Black,
        }
    }
}

/// 組札 (Foundation) のインデックス (0-3) から対応するスートを取得する。
/// 約束事: 0: Heart ❤️, 1: Diamond ♦️, 2: Club ♣️, 3: Spade ♠️
/// `pub(crate)` なので、`logic::rules` モジュールとそのサブモジュール内からのみ呼び出せる。
pub(crate) fn get_foundation_suit(foundation_index: u8) -> Option<Suit> {
    match foundation_index {
        0 => Some(Suit::Heart),
        1 => Some(Suit::Diamond),
        2 => Some(Suit::Club),
        3 => Some(Suit::Spade),
        _ => None,
    }
}

/// 指定されたスタック (`target_stack`) の一番上にあるカードのエンティティID (`Entity`) を取得するよ。
/// StackInfo の position_in_stack が最大のものを探す。
pub(crate) fn get_top_card_entity(world: &World, target_stack: StackType) -> Option<Entity> {
    let stack_entities = world.get_all_entities_with_component::<StackInfo>();
    stack_entities
        .into_iter()
        .filter(|&entity| {
            world.get_component::<StackInfo>(entity)
                .map_or(false, |stack_info| stack_info.stack_type == target_stack)
        })
        .max_by_key(|&entity| {
            world.get_component::<StackInfo>(entity)
                .map_or(0, |stack_info| stack_info.position_in_stack)
        })
} 