// src/app/state_handler.rs
//! GameApp の状態更新（主にサーバーからの情報反映）に関するロジック。

use std::sync::{Arc, Mutex}; // Arc と Mutex を使う
// use std::collections::VecDeque; // 現状未使用
use crate::ecs::world::World;
use crate::ecs::entity::Entity;
// use crate::network::{NetworkManager, ConnectionStatus}; // 現状未使用
use crate::protocol::{/*self,*/ GameStateData}; // protocol モジュールと GameStateData をインポート (selfは不要)
use crate::components::{
    card::{Card, /*Rank, Suit*/}, // Rank, Suit は未使用
    position::Position,
    stack::{StackInfo, /*StackType*/}, // StackTypeは apply_card_data 内で直接は使わない
    player::Player, // Player コンポーネントも使う
    dragging_info::DraggingInfo, // DraggingInfo もクリア対象
};
// use crate::protocol::{ServerMessage, ClientMessage, PlayerId, PlayerData, CardData, PositionData}; // GameStateData 以外は未使用
use crate::{log, /*error*/}; // error は未使用

/// サーバーから受け取った GameStateData を World に反映させる内部関数。
/// (lib.rs の GameApp::apply_game_state から移動)
/// 状態が更新された場合は true を返すように変更！
pub fn apply_game_state(
    world_arc: &Arc<Mutex<World>>, // World への参照を受け取る
    game_state: GameStateData
) -> bool {
    log("App::State: Applying game state update...");
    let mut world = match world_arc.lock() { // poison 対応
        Ok(guard) => guard,
        Err(poisoned) => {
            log(&format!("World mutex poisoned in apply_game_state: {:?}. Recovering...", poisoned));
            poisoned.into_inner()
        }
    };

    // ★状態変更があったかどうかのフラグ (クリア処理や追加処理があれば true)
    let mut state_changed = false;

    // --- 1. 既存のプレイヤーとカード情報をクリア ---
    log("  Clearing existing player and card entities...");
    let existing_player_entities: Vec<Entity> =
        world.get_all_entities_with_component::<Player>()
            .into_iter()
            .collect();
    if !existing_player_entities.is_empty() { state_changed = true; }
    for entity in existing_player_entities {
        world.remove_component::<Player>(entity);
    }
    let existing_card_entities: Vec<Entity> = world
        .get_all_entities_with_component::<Card>()
        .into_iter()
        .collect();
    if !existing_card_entities.is_empty() { state_changed = true; }
    for entity in existing_card_entities {
        world.remove_component::<Card>(entity);
        world.remove_component::<Position>(entity);
        world.remove_component::<StackInfo>(entity);
        world.remove_component::<DraggingInfo>(entity); // ドラッグ情報もクリア
        // TODO: エンティティ自体を destroy するべきか？
        //       現状はコンポーネントを削除するだけ。
        //       サーバーからの GameStateData が常に全カード情報を含むならこれで良い。
        //       差分更新の場合は destroy が必要になる。
    }

    // --- 2. 新しいプレイヤー情報を反映 --- 
    if !game_state.players.is_empty() { state_changed = true; }
    log(&format!("  Applying {} players...", game_state.players.len()));
    for player_data in game_state.players {
        log(&format!("    Player ID: {}, Name: {}", player_data.id, player_data.name));
        // プレイヤーエンティティを ID で作成または取得
        let player_entity = Entity(player_data.id as usize); // PlayerId(u32) を usize にキャスト
        world.create_entity_with_id(player_entity); // 存在しなければ作成
        // Player コンポーネントを追加/更新
        world.add_component(player_entity, Player { name: player_data.name, is_current_turn: false });
    }

    // --- 3. 新しいカード情報を反映 --- 
    if !game_state.cards.is_empty() { state_changed = true; }
    log(&format!("  Applying {} cards...", game_state.cards.len()));
    for card_data in game_state.cards {
        let entity = card_data.entity;
        world.create_entity_with_id(entity); // 存在保証

        // Card コンポーネント
        let card_component = Card {
            suit: card_data.suit.into(), // protocol::Suit -> components::card::Suit
            rank: card_data.rank.into(), // protocol::Rank -> components::card::Rank
            is_face_up: card_data.is_face_up,
        };
        world.add_component(entity, card_component);

        // StackInfo コンポーネント
        let stack_info_component = StackInfo {
            stack_type: card_data.stack_type.into(), // protocol::StackType -> components::stack::StackType
            position_in_stack: card_data.position_in_stack,
        };
        world.add_component(entity, stack_info_component);

        // Position コンポーネント
        let position_component = Position {
            x: card_data.position.x,
            y: card_data.position.y,
        };
        world.add_component(entity, position_component);
    }

    log("App::State: Game state update applied.");
    state_changed // 変更があったかどうかを返す
} 