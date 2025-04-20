// src/app/event_handler.rs
//! ユーザー入力やUIイベントに関連する GameApp のロジック。

use std::sync::{Arc, Mutex};
use crate::ecs::world::World;
use crate::network::NetworkManager;
use crate::ecs::entity::Entity;
use crate::components::card::Card;
// use crate::components::stack::StackType; // 未使用
use crate::logic::auto_move::find_automatic_foundation_move;
use crate::protocol::{self, ClientMessage}; // protocol モジュールと ClientMessage をインポート
use crate::{log, error}; // log と error マクロをインポート (lib.rs から)
use serde_json;
// use crate::app::AppEvent; // ★ AppEvent が見つからないため一旦コメントアウト
// use crate::components::position::Position; // 現状未使用
// use crate::components::dragging_info::DraggingInfo; // 現状未使用
// use web_sys::MouseEvent; // 現状未使用
// use wasm_bindgen::JsValue; // 現状未使用
// use web_sys::console; // 現状未使用

/// ダブルクリック時の実際のロジック (lib.rs の GameApp::handle_double_click_logic から移動)
pub fn handle_double_click_logic(
    entity_id: usize,
    world_arc: Arc<Mutex<World>>,
    network_manager_arc: Arc<Mutex<NetworkManager>>
) {
    log(&format!("  Executing double-click logic for entity_id: {}", entity_id));
    let entity = Entity(entity_id);

    // World をロックして、必要な情報を取得
    let world_guard = match world_arc.lock() {
        Ok(w) => w,
        Err(e) => {
            error(&format!("Error locking world in handle_double_click_logic: {}", e));
            return;
        }
    };

    // ダブルクリックされたカードを取得
    let card_to_move = match world_guard.get_component::<Card>(entity) {
        Some(card) => card.clone(), // Clone する!
        None => {
            error(&format!("Card component not found for entity {:?} in handle_double_click_logic", entity));
            return;
        }
    };

    // 自動移動先を探す！🔍
    // find_automatic_foundation_move 関数を呼び出して、指定されたカードエンティティ (entity) が
    // 自動的に移動できる Foundation があるか探す。
    // 引数には World の参照 (`&*world_guard`) とカードの Entity ID (`entity`) を渡すよ！
    let target_stack_opt = find_automatic_foundation_move(&*world_guard, entity);
    // World のロックを早めに解除！ これ以降 World の状態は読み書きできないけど、
    // ロック時間が短くなって、他の処理をブロックする可能性が減るんだ。👍
    drop(world_guard);

    match target_stack_opt {
        Some(target_stack) => {
            // 移動先が見つかった！🎉 MakeMove メッセージを送信！🚀
            log(&format!("  Found automatic move target: {:?} for card {:?}", target_stack, card_to_move));
            // components::stack::StackType を protocol::StackType に変換
            let protocol_target_stack: protocol::StackType = target_stack.into(); // From トレイトを実装済みと仮定 (protocol.rsで実装必要かも)
            let message = ClientMessage::MakeMove { moved_entity: entity, target_stack: protocol_target_stack };

            // メッセージ送信
            match serde_json::to_string(&message) {
                Ok(json_message) => {
                     match network_manager_arc.lock() {
                         Ok(nm) => {
                             if let Err(e) = nm.send_message(&json_message) {
                                 error(&format!("  Failed to send MakeMove message from logic: {}", e));
                             } else {
                                 log("  MakeMove message sent successfully from logic.");
                             }
                         },
                         Err(e) => error(&format!("Failed to lock NetworkManager in logic: {}", e))
                     }
                }
                Err(e) => error(&format!("Failed to serialize MakeMove message in logic: {}", e))
            }
        }
        None => {
            // 移動先は見つからなかった...😢
            log("  No automatic foundation move found for this card (logic).");
        }
    }
}

// TODO: ドラッグ開始、ドラッグ中、ドラッグ終了のイベントハンドラロジックもここに移動する 