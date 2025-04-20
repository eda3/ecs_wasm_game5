// src/app/drag_handler.rs
//! Handles card dragging logic (start, update, end).

use std::sync::{Arc, Mutex};
use log::{info, error, warn};

use crate::ecs::{
    world::World,
    entity::Entity,
};
use crate::components::{
    Position,
    StackInfo,
    DraggingInfo,
    StackType,
};
use crate::app::event_handler::{self, ClickTarget};
use crate::protocol::{self};
use crate::logic::rules;
use crate::{log}; // log マクロを使う (ルートから)
use super::drag_apply_handler; // ★追加: 新しいモジュールを使う
use crate::network::NetworkManager; // ★追加★


/// ドラッグ開始時の処理 (GameApp::handle_drag_start のロジック)
pub fn handle_drag_start(
    world_arc: &Arc<Mutex<World>>,
    entity_usize: usize,
    start_x: f32,
    start_y: f32
) {
    if let Ok(mut world) = world_arc.try_lock() {
        let entity = Entity(entity_usize);
        let position_opt = world.get_component::<Position>(entity);
        let stack_info_opt = world.get_component::<StackInfo>(entity);

        if let (Some(position), Some(stack_info)) = (position_opt, stack_info_opt) {
            let offset_x = start_x - position.x;
            let offset_y = start_y - position.y;

            // ★修正: 元のスタックエンティティを正しく取得する★
            let original_stack_entity = world.find_entity_by_stack_type(stack_info.stack_type)
                .unwrap_or_else(|| {
                    // 見つからない場合は警告を出し、ダミーを使う (本来はエラー処理すべき)
                    warn!("Could not find original stack entity for type {:?} in handle_drag_start. Using dummy.", stack_info.stack_type);
                    Entity(usize::MAX) // ダミーID
                });


            let dragging_info = DraggingInfo {
                original_x: position.x.into(),
                original_y: position.y.into(),
                offset_x: offset_x.into(),
                offset_y: offset_y.into(),
                original_position_in_stack: stack_info.position_in_stack as usize,
                original_stack_entity, // 正しく取得した (or ダミーの) エンティティを設定
            };

            world.add_component(entity, dragging_info);
            info!("Added DraggingInfo component to entity {:?}", entity);

        } else {
            error!("Failed to get Position or StackInfo for entity {:?} in handle_drag_start", entity);
        }
    } else {
        error!("Failed to lock world in handle_drag_start");
    }
}

/// ドラッグ中の位置更新 (GameApp::update_dragged_position のロジック)
pub fn update_dragged_position(
    world_arc: &Arc<Mutex<World>>,
    entity_id: usize,
    mouse_x: f32,
    mouse_y: f32
) {
    let entity = Entity(entity_id);
    let mut world_guard = match world_arc.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to lock world in update_dragged_position: {}", e);
            return;
        }
    };

    let dragging_info_opt = world_guard.get_component::<DraggingInfo>(entity);

    if let Some(dragging_info) = dragging_info_opt {
        let new_card_x = mouse_x - dragging_info.offset_x as f32;
        let new_card_y = mouse_y - dragging_info.offset_y as f32;

        if let Some(position_component) = world_guard.get_component_mut::<Position>(entity) {
            position_component.x = new_card_x;
            position_component.y = new_card_y;
        } else {
            error!("Failed to get Position component for dragged entity {:?} during update", entity);
        }
    } else {
        error!("DraggingInfo component not found for entity {:?} in update_dragged_position", entity);
    }
}


/// ドラッグ終了時の処理 (GameApp::handle_drag_end のロジック)
pub fn handle_drag_end(
    world_arc: &Arc<Mutex<World>>,
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    entity_usize: usize,
    end_x: f32,
    end_y: f32,
    // ★削除: Closure Arc は handle_drag_end では直接使わない ★
    // window_mousemove_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    // window_mouseup_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
) {
    let entity = Entity(entity_usize);
    log(&format!("handle_drag_end logic started for entity: {:?}, end: ({}, {})", entity, end_x, end_y));

    // --- 1. World のロックを取得 --- 
    let mut world = match world_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("World mutex poisoned in handle_drag_end: {:?}. Recovering...", poisoned);
            // TODO: 適切なエラーハンドリング or リカバリー処理
            // ここでリターンするか、リカバリーを試みるか。
            // 今回はリカバリーして続行してみる。
            poisoned.into_inner()
        }
    };

    // --- 2. DraggingInfo と元のスタック情報を取得 ---
    let dragging_info_opt = world.remove_component::<DraggingInfo>(entity);
    let dragging_info = match dragging_info_opt {
        Some(info) => {
            log(&format!("  - Successfully removed DraggingInfo: {:?}", info));
            info
        }
        None => {
            log(&format!("  - Warning: DraggingInfo not found for entity {:?}. Ignoring drag end.", entity));
            // リスナーのデタッチは mouseup クロージャ側で行われるはずなので、ここでは何もしないでリターン
            return;
        }
    };

    // 移動元のスタック情報を取得 (ルールチェックや復元用)
    // ★注意: DraggingInfo の original_stack_entity は、スタック自体ではなく、
    //         そのスタックの「一番上のカード」のエンティティIDを指している可能性がある？
    //         DealInitialCardsSystem の実装などから要確認。もしそうなら、そのカードの StackInfo を見る。
    //         ここでは、original_stack_entity が「スタックを表すエンティティ」または
    //         「スタック内のカードエンティティ」のどちらかであり、StackInfo を持っていると仮定する。
    let original_stack_info: Option<StackInfo> = world
        .get_component::<StackInfo>(entity) // DraggingInfo を削除する「前」の Entity の StackInfo を見る (移動前の状態)
        .cloned();                         // 移動前の情報を複製して保持

    // --- 3. ドロップ先の要素を特定 ---
    log(&format!("  - Finding element at drop coordinates: ({}, {})", end_x, end_y));
    let target_element = event_handler::find_clicked_element(&world, end_x, end_y);
    log(&format!("  - Found target element: {:?}", target_element));

    // --- 4. ドロップ先に基づいて処理を分岐 ---
    let mut move_successful = false; // ★移動が成功したかどうかのフラグ★

    if let Some(click_target) = target_element {
        match click_target {
            ClickTarget::Stack(target_stack_type) => {
                log(&format!("  - Target is a stack area: {:?}", target_stack_type));
                // --- 4a-i. 移動ルールのチェック ---
                log("    Checking move validity...");
                let is_valid = check_move_validity(&world, entity, target_stack_type);

                if is_valid {
                    // --- 4a-ii. 移動ルール OK の場合 ---
                    log("    Move is valid! Updating world and notifying server...");
                    let target_stack_type_for_proto: protocol::StackType = target_stack_type.into();
                    // ★ 修正: drag_apply_handler の関数を呼び出す ★
                    drag_apply_handler::update_world_and_notify_server(
                        &mut world, // 可変の MutexGuard を渡す
                        network_manager_arc,
                        entity,
                        target_stack_type, // World 更新には Component の StackType
                        target_stack_type_for_proto, // プロトコルには Protocol の StackType
                        &dragging_info,
                        &original_stack_info, // Option<StackInfo> の参照を渡す
                    );
                    move_successful = true; // ★移動成功フラグを立てる★
                } else {
                    log("    Move is invalid.");
                }
            }
            ClickTarget::Card(target_card_entity) => {
                 log(&format!("  - Target is a card ({:?}). Checking validity...", target_card_entity));
                 // カードへのドロップもチェックする (Tableau/Foundation)
                 if let Some(target_stack_info) = world.get_component::<StackInfo>(target_card_entity) {
                     let target_stack_type = target_stack_info.stack_type;
                     let is_valid = check_move_validity(&world, entity, target_stack_type);
                     if is_valid {
                         log("    Move onto card's stack is valid! Updating world and notifying server...");
                         let target_stack_type_for_proto: protocol::StackType = target_stack_type.into();
                         // ★ 修正: drag_apply_handler の関数を呼び出す ★
                         drag_apply_handler::update_world_and_notify_server(
                             &mut world,
                             network_manager_arc,
                             entity,
                             target_stack_type,
                             target_stack_type_for_proto,
                             &dragging_info,
                             &original_stack_info,
                         );
                         move_successful = true;
                     } else {
                         log("    Move onto card's stack is invalid.");
                     }
                 } else {
                     log("    Could not get StackInfo for target card. Move invalid.");
                 }
            }
        }
    }

    // --- 5. 移動が成功しなかった場合は元の位置に戻す ---
    if !move_successful {
        log("  - Move failed or target was invalid. Resetting card position.");
        reset_card_position(&mut world, entity, &dragging_info);
    }

    // World のロックはこのスコープを抜けるときに解放される

    // ★削除: リスナーのデタッチは mouseup クロージャで行うので、ここでは何もしない
    // detach_drag_listeners(window_mousemove_closure_arc, window_mouseup_closure_arc).unwrap_or_else(|e| {
    //     error!("Error detaching listeners in handle_drag_end: {:?}", e);
    // });
    log("handle_drag_end logic finished.");
}

/// ヘルパー関数: 指定された移動が有効かチェックする
fn check_move_validity(world: &World, moved_entity: Entity, target_stack_type: StackType) -> bool {
    match target_stack_type {
        StackType::Foundation(index) => {
            rules::can_move_to_foundation(world, moved_entity, index)
        }
        StackType::Tableau(index) => {
            rules::can_move_to_tableau(world, moved_entity, index)
        }
        _ => {
            log(&format!("    Dropping onto {:?} is not a valid target type.", target_stack_type));
            false
        }
    }
}

/// カードの位置をドラッグ開始時の座標に戻す。
/// (handle_drag_end から移動)
pub fn reset_card_position(
    world: &mut World, // ★ 可変の MutexGuard を受け取るように変更 ★
    entity: Entity,
    dragging_info: &DraggingInfo,
) {
    log(&format!("reset_card_position called for entity: {:?}", entity));
    let original_position = Position {
        x: dragging_info.original_x as f32,
        y: dragging_info.original_y as f32,
    };
    log(&format!("  - Reset position for entity {:?} to {:?}", entity, original_position));
    if let Some(pos_comp) = world.get_component_mut::<Position>(entity) {
        *pos_comp = original_position;
    } else {
        error!("  - Error: Position component not found for entity {:?}. Cannot reset position.", entity);
    }
    // ★ DraggingInfo を元に戻す処理も必要？ ★
    //    handle_drag_end で既に remove_component しているので、ここで再度追加する必要はなさそう。
    //    ただし、もし移動ルールチェック前に DraggingInfo を削除しない設計なら、
    //    ここで remove_component する必要がある。
    //    現状の handle_drag_end の流れでは、ここでの DraggingInfo の再追加や削除は不要。
} 