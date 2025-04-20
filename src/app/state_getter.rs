//! Gets the current game state from the World and converts it to JSON.

use std::sync::{Arc, Mutex};
use log::{info, error, warn};
use serde_json;
use wasm_bindgen::JsValue;

use crate::ecs::world::World;
use crate::components::{Position, Card, StackInfo};
use crate::protocol::{CardData, GameStateData, PositionData};

/// ワールドの状態を取得し、JSON 文字列として返します。
/// (GameApp::get_world_state_json のロジック)
pub fn get_world_state_json(world_arc: &Arc<Mutex<World>>) -> Result<JsValue, JsValue> {
    let world = match world_arc.try_lock() {
        Ok(w) => w,
        Err(e) => {
            let error_msg = format!("Failed to lock world for getting state: {}", e);
            error!("{}", error_msg); // error! マクロを使用
            // JS 側にエラーを返す方が親切かも
            return Err(JsValue::from_str(&error_msg));
        }
    };

    info!("Getting world state..."); // info! マクロを使用
    let mut cards_data = Vec::new();
    let entities_with_card = world.get_all_entities_with_component::<Card>();
    info!("Found {} entities with Card component.", entities_with_card.len()); // info! マクロを使用

    for entity in entities_with_card {
        let pos_opt = world.get_component::<Position>(entity);
        let card_opt = world.get_component::<Card>(entity);
        let stack_info_opt = world.get_component::<StackInfo>(entity);

        if let (Some(pos), Some(card), Some(stack_info)) = (pos_opt, card_opt, stack_info_opt) {
            let card_data = CardData {
                entity,
                suit: card.suit.into(),
                rank: card.rank.into(),
                is_face_up: card.is_face_up,
                stack_type: stack_info.stack_type.into(),
                position_in_stack: stack_info.position_in_stack,
                position: PositionData { x: pos.x, y: pos.y },
            };
            cards_data.push(card_data);
        } else {
            warn!("Entity {:?} is missing Position, Card, or StackInfo component. Skipping.", entity);
        }
    }
    info!("Collected data for {} cards.", cards_data.len()); // info! マクロを使用

    let game_state_data = GameStateData { players: Vec::new(), cards: cards_data };

    match serde_json::to_string(&game_state_data) {
        Ok(json_string) => {
            info!("Successfully serialized game state to JSON."); // info! マクロを使用
            Ok(JsValue::from_str(&json_string))
        }
        Err(e) => {
            let error_msg = format!("Failed to serialize game state: {}", e);
            error!("{}", error_msg); // error! マクロを使用
            Err(JsValue::from_str(&error_msg))
        }
    }
} 