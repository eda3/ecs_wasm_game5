// src/app/drag_handler.rs
//! Handles card dragging logic (start, update, end).

use std::sync::{Arc, Mutex};
use log::{info, error, warn};
use wasm_bindgen::JsValue;
use js_sys::Error;
use serde_json;

use crate::ecs::{
    world::World,
    entity::Entity,
};
use crate::components::{
    Position,
    Card,
    StackInfo,
    DraggingInfo,
    StackType,
};
use crate::network::NetworkManager;
use crate::protocol;
use crate::logic::rules;
use crate::app::{event_handler, network_handler, layout_calculator}; // layout_calculator を使う
use crate::{log}; // log マクロを使う (ルートから)


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
    network_manager_arc: &Arc<Mutex<NetworkManager>>, // NetworkManager も必要
    entity_usize: usize,
    end_x: f32,
    end_y: f32,
    // Windowリスナー解除用のクロージャも必要だが、GameApp 側で管理・実行させる
    // window_mousemove_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    // window_mouseup_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
) {
    let entity = Entity(entity_usize);
    log(&format!("handle_drag_end called for entity: {:?}, drop coordinates: ({}, {})", entity, end_x, end_y));

    let mut world = match world_arc.lock() {
         Ok(w) => w,
         Err(e) => {
             error!("Failed to lock world for drag end: {}", e);
             return; // ロック失敗なら終了
         }
    };

    // --- 1. DraggingInfo と元のスタック情報を取得 ---
    let dragging_info_opt = world.remove_component::<DraggingInfo>(entity);
    let dragging_info = match dragging_info_opt {
        Some(info) => {
            log(&format!("  - Successfully removed DraggingInfo: {:?}", info));
            info
        }
        None => {
            log(&format!("  - Warning: DraggingInfo not found for entity {:?}. Ignoring drag end.", entity));
            return;
        }
    };

    // 元のスタック情報を取得 (DraggingInfo に保存された original_stack_entity を使う)
    let original_stack_info = world.get_component::<StackInfo>(dragging_info.original_stack_entity)
                                   .cloned();

    // --- 2. ドロップ先の要素を特定 ---
    log(&format!("  - Finding element at drop coordinates: ({}, {})", end_x, end_y));
    let target_element = event_handler::find_clicked_element(&world, end_x, end_y);
    log(&format!("  - Found target element: {:?}", target_element));

    // --- 3. ドロップ先に基づいて処理を分岐 ---
    match target_element {
        Some(event_handler::ClickTarget::Stack(target_stack_type)) => {
            log(&format!("  - Target is a stack area: {:?}", target_stack_type));
            let target_stack_entity_opt = world.find_entity_by_stack_type(target_stack_type);

            if let Some(target_stack_entity) = target_stack_entity_opt {
                log(&format!("    Found stack entity: {:?}", target_stack_entity));
                let target_stack_info = world.get_component::<StackInfo>(target_stack_entity);

                if let Some(_target_stack_info) = target_stack_info { // target_stack_info は後で使うかもしれないので _ で束縛
                    log(&format!("    Target stack type from component: {:?}", _target_stack_info.stack_type));

                    // --- 3a-i. 移動ルールのチェック ---
                    log("  - Checking move validity...");
                    let is_valid = match target_stack_type {
                        StackType::Foundation(index) => rules::can_move_to_foundation(&world, entity, index),
                        StackType::Tableau(index) => rules::can_move_to_tableau(&world, entity, index),
                        _ => {
                            log(&format!("  - Dropping onto {:?} is not allowed.", target_stack_type));
                            false
                        }
                    };

                    if is_valid {
                        // --- 3a-ii. 移動ルール OK の場合 ---
                        log("  - Move is valid! Updating world and notifying server...");
                        let target_stack_type_for_proto: protocol::StackType = target_stack_type.into();

                        // ★ update_world_and_notify_server を呼び出す ★
                        update_world_and_notify_server(
                            world, // MutexGuard を渡す
                            network_manager_arc, // NetworkManager も渡す
                            entity,
                            target_stack_type, // World 更新用
                            target_stack_type_for_proto,
                            &dragging_info,
                            original_stack_info
                        );
                    } else {
                        // --- 3a-iii. 移動ルール NG の場合 ---
                        log("  - Move is invalid. Resetting card position.");
                        // ★ reset_card_position を呼び出す ★
                        reset_card_position(world, entity, &dragging_info);
                    }
                } else {
                    error!("{}", format!("  - Error: StackInfo not found for target stack entity {:?}. Resetting card position.", target_stack_entity));
                    reset_card_position(world, entity, &dragging_info);
                }
            } else {
                error!("  - Error: Stack entity not found for type {:?}. Resetting card position.", target_stack_type);
                reset_card_position(world, entity, &dragging_info);
            }
        }
        Some(event_handler::ClickTarget::Card(target_card_entity)) => {
            log(&format!("  - Target is a card ({:?}). Invalid drop target. Resetting card position.", target_card_entity));
            reset_card_position(world, entity, &dragging_info);
        }
        None => {
            log("  - Target is empty space. Resetting card position.");
            reset_card_position(world, entity, &dragging_info);
        }
    }
    // World のロック (MutexGuard) はここでスコープを抜けるので自動解除

    // Window リスナーの解除は GameApp 側で行う (ここでは何もしない)
    log("handle_drag_end: Processing finished. Window listener removal should happen in GameApp.");
}


/// World の状態を更新し、サーバーに移動を通知する内部ヘルパー関数。
/// (GameApp::update_world_and_notify_server のロジック)
///
/// # 引数
/// * `world`: World へのミュータブルな参照 (MutexGuard) <- ここがポイント！GameApp から渡してもらう
/// * `network_manager_arc`: NetworkManager への参照 (サーバー通知用)
/// * `moved_entity`: 移動されたカードのエンティティ
/// * `target_stack_type_for_update`: 移動先のスタックタイプ (World の更新用)
/// * `target_stack_type_for_proto`: 移動先のスタックタイプ (サーバー通知用のプロトコル型)
/// * `dragging_info`: ドラッグ開始時の情報 (元の位置など)
/// * `original_stack_info`: 移動元のスタック情報 (Option)
fn update_world_and_notify_server(
    mut world: std::sync::MutexGuard<'_, World>, // MutexGuard を受け取る
    network_manager_arc: &Arc<Mutex<NetworkManager>>, // NetworkManager を受け取る
    moved_entity: Entity,
    target_stack_type_for_update: StackType,
    target_stack_type_for_proto: protocol::StackType,
    dragging_info: &DraggingInfo,
    original_stack_info: Option<StackInfo>
) {
    log(&format!("update_world_and_notify_server called for entity: {:?}, target_stack_type: {:?}", moved_entity, target_stack_type_for_update));

    // --- 1. 移動元スタックのカードを表にする処理 ---
    if let Some(original_info) = original_stack_info {
        if let StackType::Tableau(_) = original_info.stack_type {
            let position_below = dragging_info.original_position_in_stack.saturating_sub(1);
            let mut entity_to_reveal: Option<Entity> = None;
            // ★ World は MutexGuard なので直接メソッドを呼べる ★
            for entity in world.get_all_entities_with_component::<StackInfo>() {
                if let Some(stack_info) = world.get_component::<StackInfo>(entity) {
                    if stack_info.stack_type == original_info.stack_type && stack_info.position_in_stack as usize == position_below {
                        entity_to_reveal = Some(entity);
                        break;
                    }
                }
            }

            if let Some(reveal_entity) = entity_to_reveal {
                 // ★ world から可変参照を取得 ★
                if let Some(mut card) = world.get_component_mut::<Card>(reveal_entity) {
                    if !card.is_face_up {
                        log(&format!("  - Revealing card {:?} in original stack {:?}.", reveal_entity, original_info.stack_type));
                        card.is_face_up = true;
                    }
                }
            }
        }
        else if original_info.stack_type == StackType::Stock {
             log("  - Moved from Stock. Handling reveal logic if necessary...");
             // 必要なら Stock の処理を実装
        }
    }

    // --- 2. 移動先スタックのエンティティを特定 ---
    // ★ world から直接呼ぶ ★
    let target_stack_entity_opt = world.find_entity_by_stack_type(target_stack_type_for_update);
    let target_stack_entity = target_stack_entity_opt.expect("Target stack entity not found despite valid move");
    log(&format!("  - Finding target stack entity for type: {:?} -> Found: {:?}", target_stack_type_for_update, target_stack_entity));

    // --- 3. 移動先スタックでの新しい順序を計算 ---
    let mut max_pos_in_target_stack: i16 = -1;
    // ★ world から直接呼ぶ ★
    for entity in world.get_all_entities_with_component::<StackInfo>() {
        if entity == moved_entity { continue; }
        // ★ world から直接呼ぶ ★
        if let Some(stack_info) = world.get_component::<StackInfo>(entity) {
            if stack_info.stack_type == target_stack_type_for_update {
                max_pos_in_target_stack = max_pos_in_target_stack.max(stack_info.position_in_stack as i16);
            }
        }
    }
    let new_position_in_stack = (max_pos_in_target_stack + 1) as u8;
    log(&format!("  - Calculated new position_in_stack for {:?}: {}", target_stack_type_for_update, new_position_in_stack));

    // --- 4. moved_entity の StackInfo コンポーネントを更新 ---
    // ★ world から可変参照を取得 ★
    if let Some(mut card_stack_info) = world.get_component_mut::<StackInfo>(moved_entity) {
        card_stack_info.stack_type = target_stack_type_for_update;
        card_stack_info.position_in_stack = new_position_in_stack;
        log(&format!("  - Updated StackInfo for moved entity {:?}: type={:?}, position={}", moved_entity, card_stack_info.stack_type, card_stack_info.position_in_stack));
    } else {
        warn!("  - Warning: StackInfo component not found for moved entity {:?}. Cannot update its stack info.", moved_entity);
        // ★ reset_card_position を呼び出す ★ (world を渡す)
        reset_card_position(world, moved_entity, dragging_info);
        return;
    }

    // --- 5. 移動したカードの Position コンポーネントを計算・更新 ---
    // ★ layout_calculator の関数を呼び出す ★ (world を参照で渡す)
    let new_position = layout_calculator::calculate_card_position(target_stack_type_for_update, new_position_in_stack, &world);
    log(&format!("  - Calculated new position for {:?}: {:?}", moved_entity, new_position));
    // ★ world から可変参照を取得 ★
    if let Some(mut pos_comp) = world.get_component_mut::<Position>(moved_entity) {
        *pos_comp = new_position;
        log(&format!("  - Updated Position for moved entity {:?}: {:?}", moved_entity, pos_comp));
    } else {
        error!("  - Error: Position component not found for moved entity {:?}. Cannot update position.", moved_entity);
    }

    // --- 6. サーバーに移動完了を通知 ---
    log(&format!("  - Notifying server about the move: entity {:?}, target stack type {:?}", moved_entity, target_stack_type_for_proto));
    match serde_json::to_string(&target_stack_type_for_proto) {
        Ok(target_stack_json) => {
            // ★ network_handler の関数を呼び出す (Arc を渡す) ★
            network_handler::send_make_move(
                network_manager_arc, // Arc を渡す
                moved_entity.0,
                target_stack_json
            );
            log("  - MakeMove message sent to server.");
        }
        Err(e) => {
            error!("  - Error: Failed to serialize target_stack_type_for_proto to JSON: {}", e);
        }
    }
    // MutexGuard (world) はスコープを抜けるときに自動的にドロップ（アンロック）される
}


/// カードの位置をドラッグ開始時の元の位置に戻す内部ヘルパー関数。
/// (GameApp::reset_card_position のロジック)
///
/// # 引数
/// * `world`: World へのミュータブルな参照 (MutexGuard)
/// * `entity`: 位置をリセットするカードのエンティティ
/// * `dragging_info`: 元の位置情報を持つ DraggingInfo
fn reset_card_position(
    mut world: std::sync::MutexGuard<'_, World>, // MutexGuard を受け取る
    entity: Entity,
    dragging_info: &DraggingInfo
) {
    log(&format!("reset_card_position called for entity: {:?}", entity));
    let original_position = Position {
        x: dragging_info.original_x as f32,
        y: dragging_info.original_y as f32,
    };
    log(&format!("  - Reset position for entity {:?} to {:?}", entity, original_position));
    // ★ world から可変参照を取得 ★
    if let Some(mut pos_comp) = world.get_component_mut::<Position>(entity) {
        *pos_comp = original_position;
    } else {
        error!("  - Error: Position component not found for entity {:?}. Cannot reset position.", entity);
    }
} 