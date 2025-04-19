// src/systems/win_condition_system.rs
use crate::{
    component::Component,
    components::{card::Card, game_state::{GameState, GameStatus}},
    entity::Entity,
    system::System,
    world::World,
};
// TODO: 将来的に StackType など、カードの場所を示すコンポーネントが必要になる
// use crate::components::stack::StackType;

/// ゲームの勝利条件をチェックするシステムだよ！🏆🎉
///
/// 現在の World の状態を見て、勝利条件（すべてのカードが組札にあるか）
/// を満たしているか判定し、満たしていれば GameState を更新するよ。
pub struct WinConditionSystem {
    // 状態は持たない
}

impl WinConditionSystem {
    /// 新しい WinConditionSystem を作成するよ。
    pub fn new() -> Self {
        Self {}
    }

    /// カードが組札にあるかどうかを判定する（仮実装）
    /// TODO: 本来は Card エンティティに紐づく StackType コンポーネントなどをチェックする
    fn is_card_in_foundation(&self, world: &World, card_entity: Entity) -> bool {
        // --- 仮実装 ---
        // ここでは、将来的に StackType::Foundation のような情報が
        // Card エンティティに関連付けられていることを想定している。
        // 今はダミーとして false を返す。
        // 正しく実装するには、MoveCardSystem で StackType を更新し、
        // ここでそれを読み取る必要がある。
        // world.get_component::<StackTypeComponent>(card_entity)
        //      .map_or(false, |st| matches!(st.stack_type, StackType::Foundation(_)))
        false // 仮
    }
}

impl System for WinConditionSystem {
    /// 勝利条件をチェックして、必要ならゲーム状態を更新するよ！
    fn run(&mut self, world: &mut World) {
        // --- 0. ゲーム状態の確認 ---
        let game_state_entity = Entity(0); // 仮のID
        let game_status = world.get_component::<GameState>(game_state_entity)
            .map(|gs| gs.status.clone());

        // すでにゲームが終了しているか、GameState がなければ何もしない
        if game_status != Some(GameStatus::Playing) {
            return;
        }

        // --- 1. 組札にあるカードの枚数を数える ---
        let mut foundation_card_count = 0;
        // World 内のすべての Card コンポーネントを持つエンティティをイテレート
        // TODO: world.iter() のような、特定のコンポーネントを持つ全エンティティを
        //       効率的に取得するメソッドが World に必要になるかも
        for entity in world.get_all_entities_with_component::<Card>() { // Entity の Vec を返す
             if self.is_card_in_foundation(world, entity) { // 仮のチェック
                 foundation_card_count += 1;
             }
        }
        println!("WinConditionSystem: 組札のカード数をチェック中... (現在 {} 枚 - 仮)", foundation_card_count);


        // --- 2. 勝利条件の判定 ---
        // クロンダイクソリティアの場合、52枚すべてのカードが組札に移動したら勝利
        if foundation_card_count == 52 { // TODO: 正しいカウントができればここが機能する
            println!("WinConditionSystem: 勝利条件達成！🏆 ゲーム状態を更新します。");
            // GameState コンポーネントを更新
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

    #[test]
    fn test_win_condition_not_met() {
        let mut world = World::new();
        let mut system = WinConditionSystem::new();

        // GameState を Playing でセットアップ
        let game_state_entity = world.create_entity();
        world.add_component(game_state_entity, GameState::new()); // 初期状態は Playing
        assert_eq!(game_state_entity, Entity(0)); // IDが0であることを確認 (仮定)

        // カードをいくつか追加 (ただし、is_card_in_foundation が常に false を返すので、枚数は関係ない)
        let _card1 = world.create_entity();
        world.add_component(_card1, Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true });

        // システムを実行
        system.run(&mut world);

        // GameState が Won になっていないことを確認
        let game_state = world.get_component::<GameState>(game_state_entity).unwrap();
        assert_eq!(game_state.status, GameStatus::Playing);
        println!("勝利条件未達成テスト、成功！👍");
    }

    // TODO: 勝利条件達成時のテストを追加する
    //       そのためには、is_card_in_foundation が正しく機能するか、
    //       またはテストダブル（常に true を返すモック版 is_card_in_foundation）を使う必要がある。
    //       World::get_all_entities_with_component の仮実装も直す必要がある。
} 