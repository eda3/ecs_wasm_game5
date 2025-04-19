// src/app/renderer.rs
//! GameApp の描画関連ロジック。

use std::sync::{Arc, Mutex};
use crate::world::World;
use crate::components::{Position, Card, DraggingInfo, StackInfo};
use crate::entity::Entity;
use crate::{log}; // log マクロを使う
use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};

// --- 公開関数 (GameApp から呼び出される) ---

/// Rust側で Canvas にゲーム画面を描画する関数。
/// GameApp::render_game_rust のロジックを移動。
pub fn render_game_rust(
    world_arc: &Arc<Mutex<World>>,
    canvas: &HtmlCanvasElement, // Canvas と Context への参照を受け取る
    context: &CanvasRenderingContext2d
) -> Result<(), JsValue> {
    log("App::Renderer: render_game_rust() called!");

    // --- ステップ1: Canvas 寸法を取得 --- 
    let canvas_width = canvas.width() as f64;
    let canvas_height = canvas.height() as f64;

    // --- ステップ2: Canvas をクリア --- 
    context.clear_rect(0.0, 0.0, canvas_width, canvas_height);

    // --- ステップ3: World からカード情報を取得 & ソート --- 
    let world = world_arc.lock().map_err(|e| JsValue::from_str(&format!("Failed to lock world mutex: {}", e)))?;

    let card_entities = world.get_all_entities_with_component::<Card>();
    let mut cards_to_render: Vec<(Entity, &Position, &Card, Option<DraggingInfo>, Option<&StackInfo>)> = Vec::with_capacity(card_entities.len());

    for &entity in &card_entities {
        if let (Some(pos), Some(card)) = (
            world.get_component::<Position>(entity),
            world.get_component::<Card>(entity)
        ) {
            let dragging_info = world.get_component::<DraggingInfo>(entity).cloned();
            let stack_info = world.get_component::<StackInfo>(entity);
            cards_to_render.push((entity, pos, card, dragging_info, stack_info));
        } else {
            log(&format!("Warning: Skipping entity {:?} in render_game_rust because Card or Position component is missing.", entity));
        }
    }

    // Sort cards by stack and position within the stack, or original position if dragging
    cards_to_render.sort_by(|a, b| {
        let (_, _, _, dragging_info_a, stack_info_a_opt) = a;
        let (_, _, _, dragging_info_b, stack_info_b_opt) = b;

        let order_a = dragging_info_a
            .as_ref()
            .map(|di| di.original_position_in_stack)
            .or_else(|| stack_info_a_opt.map(|si| si.position_in_stack as usize))
            .unwrap_or(0);

        let order_b = dragging_info_b
            .as_ref()
            .map(|di| di.original_position_in_stack)
            .or_else(|| stack_info_b_opt.map(|si| si.position_in_stack as usize))
            .unwrap_or(0);

        order_a.cmp(&order_b)
    });

    // --- ステップ4: 描画処理 (現在の DOM 操作部分は未実装なのでコメントアウト) ---
    log(&format!("App::Renderer: Sorted card render data ({} entities): {:?}", cards_to_render.len(), cards_to_render));
    // TODO: 実際に Canvas API を使って描画する処理を実装する！
    //       cards_to_render の情報を使って、各カードを context に描画する。

    Ok(())
} 