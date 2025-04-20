// src/app/drag_handler.rs
//! Handles card dragging logic (start, update, end).

use std::sync::{Arc, Mutex};
use log::{error};

use crate::ecs::{
    world::World,
    entity::Entity,
};
use crate::components::{
    Position,
    StackInfo,
    DraggingInfo,
    StackType,
    Card,
};
use crate::app::event_handler::{self, ClickTarget};
use crate::protocol::{self};
use crate::logic::rules;
use crate::{log}; // log マクロを使う (ルートから)
use super::drag_apply_handler; // ★追加: 新しいモジュールを使う
use crate::network::NetworkManager; // ★追加★


/// ドラッグ開始時の処理 (GameApp::handle_drag_start のロジック)
/// ★ 修正: 新しい DraggingInfo 構造体に合わせて処理を変更 ★
pub fn handle_drag_start(
    world_arc: &Arc<Mutex<World>>,
    entity_usize: usize,
    start_x: f32,
    start_y: f32
) {
    // World を try_lock して MutexGuard を取得
    let world_guard = match world_arc.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to lock world in handle_drag_start: {}", e);
            return;
        }
    };

    let clicked_entity = Entity(entity_usize);

    // ★★★ Tableau の裏向きカードはドラッグ不可にするチェック ★★★
    if let (Some(stack_info), Some(card)) = (
        world_guard.get_component::<StackInfo>(clicked_entity),
        world_guard.get_component::<Card>(clicked_entity)
    ) {
        if matches!(stack_info.stack_type, StackType::Tableau(_)) && !card.is_face_up {
            log(&format!("Attempted to drag face-down card {:?} from Tableau. Drag cancelled.", clicked_entity));
            return; // ドラッグ処理をせずに終了
        }
    }
    // ★★★ チェックここまで ★★★

    // try_lock で得たガードは読み取り専用なので、コンポーネント追加のために可変にする必要がある場合、
    // ここで drop するか、最初から lock() を使う。
    // DraggingInfo 追加のために後で可変参照が必要なので、ここでは一旦 drop し、後で lock() する。
    // （ただし、ロックの粒度が大きくなる点に注意）
    drop(world_guard);

    // 再度、今度は書き込み可能なロックを取得
    let mut world = match world_arc.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to obtain mutable lock for world in handle_drag_start: {}", e);
            return;
        }
    };

    let position_opt = world.get_component::<Position>(clicked_entity).cloned(); // Clone して後で使う
    let stack_info_opt = world.get_component::<StackInfo>(clicked_entity).cloned(); // Clone して後で使う
    let card_opt = world.get_component::<Card>(clicked_entity).cloned(); // Clone して後で使う

    // 必要なコンポーネントがあるか確認
    if let (Some(position), Some(stack_info), Some(card)) = (position_opt, stack_info_opt, card_opt) {

        // --- グループドラッグ判定 & グループ特定 --- 
        let mut dragged_group = vec![clicked_entity];
        let mut is_group_drag = false;

        if let StackType::Tableau(tableau_index) = stack_info.stack_type {
            if card.is_face_up {
                is_group_drag = true;
                let clicked_pos_in_stack = stack_info.position_in_stack;

                let potential_group_members: Vec<Entity> = world
                    .get_all_entities_with_component::<StackInfo>()
                    .into_iter()
                    .filter(|&e| e != clicked_entity)
                    .filter(|&e| world.get_component::<Card>(e).is_some())
                    .filter(|&e| {
                        world.get_component::<StackInfo>(e)
                            .map_or(false, |si| {
                                si.stack_type == StackType::Tableau(tableau_index) &&
                                si.position_in_stack > clicked_pos_in_stack
                            })
                    })
                    .collect();
                dragged_group.extend(potential_group_members);
                log(&format!("  Group drag initiated for Tableau {}. Group: {:?}", tableau_index, dragged_group));
            }
        }

        // --- 新しい DraggingInfo のための情報収集 --- 
        let original_stack_type = stack_info.stack_type;
        let mut original_group_positions: Vec<(Entity, u8)> = Vec::new();
        for entity_in_group in &dragged_group {
            if let Some(si) = world.get_component::<StackInfo>(*entity_in_group) {
                original_group_positions.push((*entity_in_group, si.position_in_stack));
            } else {
                error!("  Error: Could not get StackInfo for entity {:?} in group during DraggingInfo creation.", entity_in_group);
                // エラー処理: グループ形成を中止するか、このカードを除外するか？
                // ここでは一旦無視して続行するが、問題が起こる可能性あり。
            }
        }
        // position_in_stack でソート！
        original_group_positions.sort_by_key(|&(_, pos)| pos);

        let offset_x = start_x - position.x;
        let offset_y = start_y - position.y;

        // --- 新しい DraggingInfo の作成 --- 
        let dragging_info = DraggingInfo {
            original_stack_type,
            original_group_positions,
            original_x: position.x as f64, // f64 にキャスト
            original_y: position.y as f64, // f64 にキャスト
            offset_x: offset_x as f64,     // f64 にキャスト
            offset_y: offset_y as f64,     // f64 にキャスト
            dragged_group,              // 特定したグループ
        };

        // --- DraggingInfo をクリックされたエンティティに追加 --- 
        world.add_component(clicked_entity, dragging_info);
        log(&format!("Added DraggingInfo component to entity {:?} (Group drag: {})", clicked_entity, is_group_drag));

    } else {
        error!("Failed to get required components (Position, StackInfo, Card) for entity {:?} in handle_drag_start", clicked_entity);
    }
    // World の可変ロックはスコープを抜けるときに解放される
}

/// ドラッグ中の位置更新 (GameApp::update_dragged_position のロジック)
/// ★ 修正: グループドラッグに対応 ★
pub fn update_dragged_position(
    world_arc: &Arc<Mutex<World>>,
    clicked_entity_id: usize, // クリックされた代表エンティティのID
    mouse_x: f32,
    mouse_y: f32
) {
    let clicked_entity = Entity(clicked_entity_id);
    let mut world_guard = match world_arc.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to lock world in update_dragged_position: {}", e);
            return;
        }
    };

    // DraggingInfo を代表エンティティから取得
    let dragging_info_opt = world_guard.get_component::<DraggingInfo>(clicked_entity).cloned(); // 後で World を可変借用するため clone

    if let Some(dragging_info) = dragging_info_opt {
        // マウス位置に基づいて、クリックされたカードの新しい左上の座標を計算
        let base_card_new_x = mouse_x - dragging_info.offset_x as f32;
        let base_card_new_y = mouse_y - dragging_info.offset_y as f32;

        // グループ内の各カードの位置を更新
        // グループのカードを position_in_stack でソートしておく（Y座標計算のため）
        let mut sorted_group = dragging_info.dragged_group.clone();
        sorted_group.sort_by_key(|&entity| {
            world_guard.get_component::<StackInfo>(entity)
                .map_or(u8::MAX, |si| si.position_in_stack) // StackInfo がなければ最後尾扱い
        });

        let mut current_y_offset = 0.0; // クリックされたカードからの相対Yオフセット

        for entity_in_group in sorted_group {
            if let Some(position_component) = world_guard.get_component_mut::<Position>(entity_in_group) {
                position_component.x = base_card_new_x;
                position_component.y = base_card_new_y + current_y_offset;

                // 次のカードのための Y オフセットを計算 (グループは Tableau のはずなので is_face_up を見る)
                if let Some(card) = world_guard.get_component::<Card>(entity_in_group) {
                    // ★ 注意: ここでのオフセットは、グループ内の相対位置を維持するためのもの ★
                    //       layout_calculator と同じ定数を使うべきかは要検討
                    //       とりあえず layout_calculator に合わせておく
                    current_y_offset += if card.is_face_up {
                        crate::config::layout::TABLEAU_Y_OFFSET_FACE_UP
                    } else {
                        // グループは表向きカードのはずだが、念のため
                        crate::config::layout::TABLEAU_Y_OFFSET_FACE_DOWN
                    };
                } else {
                    // Card がない場合 (エラーケース)
                    current_y_offset += crate::config::layout::TABLEAU_Y_OFFSET_FACE_DOWN;
                }
            } else {
                error!("Failed to get Position component for entity {:?} in dragged group during update", entity_in_group);
            }
        }
    } else {
        // 代表エンティティに DraggingInfo がない (通常発生しないはず)
        error!("DraggingInfo component not found for representative entity {:?} in update_dragged_position", clicked_entity);
    }
}


/// ドラッグ終了時の処理 (GameApp::handle_drag_end のロジック)
pub fn handle_drag_end(
    world_arc: &Arc<Mutex<World>>,
    network_manager_arc: &Arc<Mutex<NetworkManager>>,
    entity_usize: usize,
    end_x: f32,
    end_y: f32,
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
    // ★修正: DraggingInfo がない場合はエラーログを出してリターンする形に変更★
    let dragging_info = match world.remove_component::<DraggingInfo>(entity) {
        Some(info) => {
            log(&format!("  - Successfully removed DraggingInfo: {:?}", info));
            info
        }
        None => {
            // DraggingInfo がない状態で handle_drag_end が呼ばれるのは通常異常系
            error!("  - Error: DraggingInfo not found for entity {:?} in handle_drag_end. Aborting.", entity);
            return; // 処理を中断
        }
    };

    // --- 3. ドロップ先の要素を特定 ---
    log(&format!("  - Finding element at drop coordinates: ({}, {})", end_x, end_y));
    let target_element = event_handler::find_clicked_element(&world, end_x, end_y, Some(entity));
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
                    // ★ 修正: 不要な第3引数 entity を削除 ★
                    drag_apply_handler::update_world_and_notify_server(
                        &mut world,
                        network_manager_arc,
                        target_stack_type,
                        target_stack_type_for_proto,
                        &dragging_info,
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
                         // ★ 修正: 不要な第3引数 entity を削除 ★
                         drag_apply_handler::update_world_and_notify_server(
                             &mut world,
                             network_manager_arc,
                             target_stack_type,
                             target_stack_type_for_proto,
                             &dragging_info,
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
        reset_card_position(&mut world, &dragging_info);
    }

    // World のロックはこのスコープを抜けるときに解放される

    // ★削除: リスナーのデタッチは JS 側で行うので、ここでは何もしない
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
/// ★ 修正: グループドラッグ対応 + 新しい DraggingInfo を使用 ★
pub fn reset_card_position(
    world: &mut World,
    dragging_info: &DraggingInfo,
) {
    log(&format!("reset_card_position called for group starting with: {:?}", dragging_info.dragged_group.first()));

    // --- 新しい DraggingInfo を使ってグループ全員の位置を復元 --- 
    let original_stack_type = dragging_info.original_stack_type;

    // original_group_positions にはソート済みの (Entity, original_position_in_stack) が入っているはず
    for (entity_in_group, original_pos_in_stack) in &dragging_info.original_group_positions {

        // 元の位置を layout_calculator で再計算
        // ★ World の可変借用ができないため、一旦 clone する (非効率かも) ★
        //    または layout_calculator が &World を取るように修正する
        //    現状の実装 (calculate_card_position が &World を取る) なら clone 不要なはず
        let original_position = crate::app::layout_calculator::calculate_card_position(
            original_stack_type, 
            *original_pos_in_stack, // u8 を渡す
            &*world, // World の不変参照を渡す
        );
        log(&format!("  - Resetting entity {:?} (original stack: {:?}, pos: {}) to {:?}", 
            entity_in_group, original_stack_type, original_pos_in_stack, original_position));

        if let Some(pos_comp) = world.get_component_mut::<Position>(*entity_in_group) {
            *pos_comp = original_position;
        } else {
            error!("  - Error: Position component not found for entity {:?}", entity_in_group);
        }
    }

    // --- 古いコメントや仮実装を削除 --- 

} 