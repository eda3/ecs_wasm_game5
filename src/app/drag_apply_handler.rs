// src/app/drag_apply_handler.rs
//! Handles applying the consequences of a successful drag-and-drop move.

use std::sync::{Arc, Mutex};
use crate::ecs::world::World;
use crate::ecs::entity::Entity;
use crate::components::{Position, Card, StackInfo, StackType};
use crate::network::NetworkManager;
use crate::protocol::{self, ClientMessage}; // Import ClientMessage specifically
use crate::app::network_handler;
use crate::app::layout_calculator;
use crate::components::dragging_info::DraggingInfo; // Needed for the function signature, though not used now
use crate::log;
use log::error;

/// World の状態を更新し、サーバーに移動を通知する。
/// (handle_drag_end から移動)
/// ★ pub に変更 ★
pub fn update_world_and_notify_server(
    world: &mut World,
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    moved_entity: Entity,
    target_stack_type: StackType,
    target_stack_type_for_proto: protocol::StackType,
    _dragging_info: &DraggingInfo,
    original_stack_info: &Option<StackInfo>,
) {
    log(&format!("update_world_and_notify_server for entity: {:?}, target: {:?}", moved_entity, target_stack_type));

    // --- 1. 新しいスタック内での位置 (position_in_stack) を決定 --- 
    let new_position_in_stack = world
        .get_all_entities_with_component::<StackInfo>()
        .iter()
        .filter(|&&entity| {
            world.get_component::<StackInfo>(entity)
                .map_or(false, |si| si.stack_type == target_stack_type)
        })
        .count() as u8;
    log(&format!("  - Calculated new position_in_stack: {}", new_position_in_stack));

    // --- 2. 新しい描画位置 (Position) を計算 --- 
    let new_position = layout_calculator::calculate_card_position(
        target_stack_type,
        new_position_in_stack,
        world, 
    );
    log(&format!("  - Calculated new Position: ({}, {})", new_position.x, new_position.y));

    // --- 3. World の状態を更新 --- 
    apply_card_move_to_world(
        world, 
        moved_entity,
        target_stack_type,
        &new_position,
        new_position_in_stack,
    );

    // --- 4. 移動元のスタックのカードを必要なら表にする --- 
    reveal_underlying_card_if_needed(
        world, 
        original_stack_info,
    );

    // --- 5. サーバーに移動を通知 --- 
    notify_move_to_server(
        network_manager_arc,
        moved_entity,
        target_stack_type_for_proto,
    );

    log("update_world_and_notify_server finished.");
}

/// World 内のカードの StackInfo と Position を更新する。
/// (ファイル内プライベート関数)
fn apply_card_move_to_world(
    world: &mut World,
    moved_entity: Entity,
    target_stack_type: StackType,
    new_position: &Position,
    new_position_in_stack: u8,
) {
    log(&format!("  Applying card move to world for {:?}", moved_entity));
    // StackInfo コンポーネントを更新
    if let Some(stack_info) = world.get_component_mut::<StackInfo>(moved_entity) {
        log(&format!("    Updating StackInfo from {:?}({}) to {:?}({})",
            stack_info.stack_type, stack_info.position_in_stack,
            target_stack_type, new_position_in_stack));
        stack_info.stack_type = target_stack_type;
        stack_info.position_in_stack = new_position_in_stack;
    } else {
        error!("    Error: StackInfo component not found for moved entity {:?}", moved_entity);
        return; 
    }

    // Position コンポーネントを更新
    if let Some(pos_comp) = world.get_component_mut::<Position>(moved_entity) {
        log(&format!("    Updating Position to ({}, {})", new_position.x, new_position.y));
        *pos_comp = new_position.clone();
    } else {
        error!("    Error: Position component not found for moved entity {:?}", moved_entity);
    }
}

/// 移動元のスタックが Tableau で、移動したカードの下にカードがあれば、それを表にする。
/// (ファイル内プライベート関数)
fn reveal_underlying_card_if_needed(
    world: &mut World,
    original_stack_info_opt: &Option<StackInfo>,
) {
    log("  Checking if underlying card needs reveal...");
    if let Some(original_stack_info) = original_stack_info_opt {
        if let StackType::Tableau(original_tableau_index) = original_stack_info.stack_type {
            log(&format!("    Original stack was Tableau {}. Finding card below position {}.", original_tableau_index, original_stack_info.position_in_stack));
            if original_stack_info.position_in_stack > 0 {
                let position_below = original_stack_info.position_in_stack - 1;
                let card_below_entity_opt = world
                    .get_all_entities_with_component::<StackInfo>()
                    .into_iter()
                    .find(|&entity| {
                        world.get_component::<StackInfo>(entity)
                            .map_or(false, |si| {
                                si.stack_type == StackType::Tableau(original_tableau_index) &&
                                si.position_in_stack == position_below
                            })
                    });

                if let Some(card_below_entity) = card_below_entity_opt {
                    log(&format!("    Found card below: {:?}", card_below_entity));
                    if let Some(card_below) = world.get_component_mut::<Card>(card_below_entity) {
                        if !card_below.is_face_up {
                            log(&format!("    Revealing card {:?}", card_below_entity));
                            card_below.is_face_up = true;
                        } else {
                            log("    Card below was already face up.");
                        }
                    } else {
                        error!("    Error: Card component not found for card below {:?}", card_below_entity);
                    }
                } else {
                    log(&format!("    No card found at position {} in Tableau {}.", position_below, original_tableau_index));
                }
            } else {
                log("    Moved card was the bottom card in Tableau, nothing to reveal.");
            }
        } else {
             log("    Original stack was not Tableau, no need to reveal.");
        }
    } else {
        log("    Original stack info not available, cannot check for reveal.");
    }
}

/// サーバーにカード移動メッセージを送信する。
/// (ファイル内プライベート関数)
fn notify_move_to_server(
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    moved_entity: Entity,
    target_stack_type_for_proto: protocol::StackType,
) {
    log(&format!("  Notifying server about move for {:?} to {:?}", moved_entity, target_stack_type_for_proto));
    let message = ClientMessage::MakeMove {
        moved_entity,
        target_stack: target_stack_type_for_proto,
    };
    if let Err(e) = network_handler::send_serialized_message(network_manager_arc, message) {
        error!("    Error sending MakeMove message: {}", e);
    }
} 