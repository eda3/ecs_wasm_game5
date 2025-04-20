// src/systems/win_condition_system.rs
use crate::components::{
    card::Card,
    stack::{StackInfo, StackType},
    game_state::{GameState, GameStatus},
};
use crate::ecs::{
    entity::Entity,
    system::System,
    world::World,
};
// use crate::logic::rules::check_win_condition; // 未使用 (WinConditionSystem内の同名メソッドを使うため)
// use crate::log; // 未使用
// TODO: 将来的に StackType など、カードの場所を示すコンポーネントが必要になる
// use crate::components::stack::StackType;

/// ゲームの勝利条件をチェックするシステムだよ！🏆🎉
///
/// 現在の World の状態を見て、勝利条件（すべてのカードが組札にあるか）
/// を満たしているか判定し、満たしていれば GameState を更新するよ。
pub struct WinConditionSystem;

impl WinConditionSystem {
    /// 新しい WinConditionSystem を作成するよ。
    pub fn new() -> Self {
        Self {}
    }

    /// ゲームの勝利条件が満たされているかチェックする関数だよ。
    fn check_win_condition(&self, world: &World) -> bool {
        let card_entities = world.get_all_entities_with_component::<Card>();
        if card_entities.len() != 52 {
            // カードが52枚揃ってない場合は勝利ではない (Deal直後など)
            return false; 
        }
        // 全てのカードについて is_card_in_foundation をチェック！
        // all() を使うと、イテレータの全要素が条件を満たすかチェックできてスマート！✨
        card_entities.iter().all(|&card_entity| {
            self.is_card_in_foundation(world, card_entity) // 引数名を元に戻した関数を呼ぶ
        })
    }

    /// 特定のカードエンティティが組札 (Foundation) にあるかチェックするヘルパー関数。
    // 引数名のアンダースコアを削除！
    fn is_card_in_foundation(&self, world: &World, card_entity: Entity) -> bool {
        // World から StackInfo コンポーネントを取得する。
        world.get_component::<StackInfo>(card_entity)
             // Option 型の map_or メソッドを使うよ！
             // Some(stack_info) があれば、クロージャ |stack_info| ... を実行。
             // None なら、デフォルト値 false を返す。
             .map_or(false, |stack_info| {
                 // matches! マクロで stack_type が Foundation かどうかを判定！
                 matches!(stack_info.stack_type, StackType::Foundation(_))
             })
    }
}

impl System for WinConditionSystem {
    /// 勝利条件をチェックして、必要ならゲーム状態を更新するよ！
    fn run(&mut self, world: &mut World) {
        let game_state_entity = Entity(0); 
        let game_status = world.get_component::<GameState>(game_state_entity)
            .map(|gs| gs.status.clone());

        if game_status != Some(GameStatus::Playing) {
            return;
        }

        // 勝利条件をチェック！ (check_win_condition を使う)
        if self.check_win_condition(world) {
            println!("WinConditionSystem: 勝利条件達成！🏆 ゲーム状態を更新します。");
            if let Some(game_state) = world.get_component_mut::<GameState>(game_state_entity) {
                game_state.status = GameStatus::Won;
            } else {
                eprintln!("WinConditionSystem: GameState が見つかりません！状態を更新できませんでした。");
            }
        }
    }
}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::card::{Suit, Rank}; // Card は WinConditionSystem で使ってるので不要
    use crate::entity::Entity;
    use crate::world::World; // World は WinConditionSystem で使ってるので不要

    // テスト用のヘルパー関数 (World にカードを追加)
    fn add_card_to_world(world: &mut World, entity_id: usize, stack_type: StackType, pos_in_stack: u8) -> Entity {
        let entity = Entity(entity_id);
        world.create_entity_with_id(entity); // 特定のIDで作成/予約
        // 仮のカードデータ。勝利条件チェックには関係ないけど、Card コンポーネントは必要
        world.add_component(entity, Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true }); 
        world.add_component(entity, StackInfo::new(stack_type, pos_in_stack));
        entity
    }

    #[test]
    fn test_win_condition_not_met_real() {
        let mut world = World::new();
        world.register_component::<Card>();
        world.register_component::<StackInfo>();
        world.register_component::<GameState>();

        let mut system = WinConditionSystem::new();

        // GameState を Playing でセットアップ
        let game_state_entity = Entity(0);
        world.create_entity_with_id(game_state_entity);
        world.add_component(game_state_entity, GameState { status: GameStatus::Playing });

        // 51枚を Foundation に、1枚を Tableau に置く
        for i in 1..=51 {
            add_card_to_world(&mut world, i, StackType::Foundation((i % 4) as u8), 0);
        }
        add_card_to_world(&mut world, 52, StackType::Tableau(0), 0);

        // システムを実行
        system.run(&mut world);

        // GameState が Won になっていないことを確認
        let game_state = world.get_component::<GameState>(game_state_entity).unwrap();
        assert_eq!(game_state.status, GameStatus::Playing);
        println!("勝利条件未達成テスト (実装版 is_card_in_foundation), 成功！👍");
    }

    #[test]
    fn test_win_condition_met_real() {
        let mut world = World::new();
        world.register_component::<Card>();
        world.register_component::<StackInfo>();
        world.register_component::<GameState>();
        let mut system = WinConditionSystem::new();

        let game_state_entity = Entity(0);
        world.create_entity_with_id(game_state_entity);
        world.add_component(game_state_entity, GameState { status: GameStatus::Playing });

        // 52枚すべてを Foundation に置く
        for i in 1..=52 {
            add_card_to_world(&mut world, i, StackType::Foundation((i % 4) as u8), (i / 4) as u8);
        }

        system.run(&mut world);

        // GameState が Won になっていることを確認
        let game_state = world.get_component::<GameState>(game_state_entity).unwrap();
        assert_eq!(game_state.status, GameStatus::Won);
        println!("勝利条件達成テスト (実装版 is_card_in_foundation), 成功！🏆");
    }
    
    #[test]
    fn test_win_condition_not_met_not_enough_cards() {
        let mut world = World::new();
        world.register_component::<Card>();
        world.register_component::<StackInfo>();
        world.register_component::<GameState>();
        let mut system = WinConditionSystem::new();

        let game_state_entity = Entity(0);
        world.create_entity_with_id(game_state_entity);
        world.add_component(game_state_entity, GameState { status: GameStatus::Playing });

        // 51枚だけ Foundation に置く
        for i in 1..=51 {
            add_card_to_world(&mut world, i, StackType::Foundation((i % 4) as u8), (i / 4) as u8);
        }

        system.run(&mut world);

        // GameState が Won になっていないことを確認
        let game_state = world.get_component::<GameState>(game_state_entity).unwrap();
        assert_eq!(game_state.status, GameStatus::Playing);
        println!("勝利条件未達成 (カード不足) テスト, 成功！👍");
    }
} 