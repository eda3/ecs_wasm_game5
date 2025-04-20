// src/logic/rules/move_validation.rs
//! カード移動の全体的な妥当性チェックを行う。

use crate::ecs::world::World;
use crate::ecs::entity::Entity;
use crate::components::stack::StackType;
use crate::components::card::Card;
use crate::logic::rules::{foundation, tableau}; // 各ルール関数を use
use web_sys::console; // ログ出力用
use wasm_bindgen::JsValue;

/// 指定されたエンティティを特定のスタックに移動できるか検証する。
/// (元 MoveCardSystem::check_move_validity)
pub fn is_move_valid(
    world: &World,
    moved_entity: Entity,
    target_stack: StackType,
) -> bool {
    // 移動元カード情報を取得 (エラーチェックは呼び出し元で行う想定でも良いが、ここでも念のため)
    if world.get_component::<Card>(moved_entity).is_none() {
        console::log_1(&JsValue::from_str(&format!("[Rules Validation Error] Moved entity {:?} has no Card component!", moved_entity)));
        return false;
    }
    // 移動元スタック情報はここでは不要なことが多い

    // 移動先スタックの種類に応じてルールチェック
    match target_stack {
        StackType::Tableau(target_index) => {
            // 場札への移動ルールをチェック
            tableau::can_move_to_tableau(world, moved_entity, target_index)
        }
        StackType::Foundation(target_index) => {
            // 組札への移動ルールをチェック
            foundation::can_move_to_foundation(world, moved_entity, target_index)
        }
        StackType::Stock | StackType::Waste | StackType::Hand => {
            // Stock, Waste, Hand への直接移動は通常許可されない
            console::log_1(&JsValue::from_str(&format!("[Rules Validation] Moving to {:?} is not allowed.", target_stack)));
            false
        }
    }
} 