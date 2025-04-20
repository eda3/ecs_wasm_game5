// src/app/layout_calculator.rs
//! Calculates the drawing position of cards in different stacks.

use crate::ecs::world::World; // World が必要
use crate::components::{Position, StackType};
use crate::config::layout::{
    STACK_PADDING,
    TABLEAU_START_X, TABLEAU_START_Y,
    FOUNDATION_START_X, FOUNDATION_START_Y,
    STOCK_POS_X, STOCK_POS_Y, WASTE_POS_X, WASTE_POS_Y,
    TABLEAU_X_OFFSET, TABLEAU_Y_OFFSET_FACE_DOWN, TABLEAU_Y_OFFSET_FACE_UP,
};

/// スタックタイプとスタック内での順序に基づいて、カードの描画位置を計算します。
/// (元々は GameApp::update_world_and_notify_server 内にあったロジック)
///
/// # 引数
/// * `stack_type`: カードが属するスタックのタイプ。
/// * `position_in_stack`: スタック内でのカードの順序 (0から始まる)。
/// * `world`: World への参照 (必要に応じてスタックの他のカード情報を参照するため)。
///
/// # 戻り値
/// * 計算されたカードの `Position`。
pub fn calculate_card_position(
    stack_type: StackType,
    position_in_stack: u8,
    world: &World, // World を参照で受け取る
) -> Position {
    match stack_type {
        StackType::Stock => Position { x: STOCK_POS_X, y: STOCK_POS_Y },
        StackType::Waste => Position { x: WASTE_POS_X, y: WASTE_POS_Y },
        StackType::Foundation(index) => {
            let base_x = FOUNDATION_START_X + (TABLEAU_X_OFFSET + STACK_PADDING) * index as f32;
            Position { x: base_x, y: FOUNDATION_START_Y }
        }
        StackType::Tableau(index) => {
            let base_x = TABLEAU_START_X + (TABLEAU_X_OFFSET + STACK_PADDING) * index as f32;
            let mut current_y = TABLEAU_START_Y;

            // position_in_stack までにあるカードの is_face_up 状態を見て Y座標を計算
            // (自分自身は含まない)
            let mut calculated_y = TABLEAU_START_Y;
            let stack_entities = world.find_entity_by_stack_type(StackType::Tableau(index));

            // position_in_stack でソートされたリストを作成 (Vec 化してソート)
            let mut sorted_entities: Vec<_> = stack_entities.iter()
                .filter_map(|&e| {
                    world.get_component::<crate::components::StackInfo>(e)
                        .map(|si| (e, si.position_in_stack))
                })
                .collect();
            sorted_entities.sort_by_key(|&(_, pos)| pos);

            for (i, (entity, _pos)) in sorted_entities.iter().enumerate() {
                 // 自分の position_in_stack に到達したら、それが自分の Y 座標
                 if i == position_in_stack as usize {
                    calculated_y = current_y;
                    break;
                 }

                // 次のカードの位置を計算するために Y を加算
                if let Some(card) = world.get_component::<crate::components::Card>(*entity) {
                    current_y += if card.is_face_up { TABLEAU_Y_OFFSET_FACE_UP } else { TABLEAU_Y_OFFSET_FACE_DOWN };
                } else {
                    // カードコンポーネントがない場合(スタック自体など)は Y を変えないか、エラー処理
                    // 基本的にここにはカードしか来ないはず
                    current_y += TABLEAU_Y_OFFSET_FACE_DOWN; // 安全のため FaceDown 扱い
                }

                 // 最後のカードについて処理した場合 (ループの最後)
                 // 最後のカードの次の位置 (つまり、新しく追加されるカードの位置) を計算して終了
                 if i == sorted_entities.len() - 1 {
                     calculated_y = current_y;
                     break; // ループ終了
                 }
            }

             // もし position_in_stack が 0 の場合 (最初のカード) は calculated_y は初期値の TABLEAU_START_Y のまま
             // position_in_stack が既存の最大値+1 の場合、ループの最後の calculated_y が使われる

            Position { x: base_x, y: calculated_y }
        }
        StackType::Hand => todo!("Layout for Hand stack is not implemented yet"),
    }
} 