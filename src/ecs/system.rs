// src/ecs/system.rs

// これまで作った World を使うからインポートするよ。
use crate::ecs::world::World;
// use std::collections::HashMap; // テスト内でのみ使用するため、ここでは不要
// use crate::ecs::entity::Entity; // テスト内でのみ使用するため、ここでは不要

/// System（システム）トレイトだよ！
///
/// システムは、ゲームのロジック（ルールや振る舞い）を実行する役割を持つんだ。
/// 例えば、「物理演算システム」「敵のAIシステム」「描画システム」みたいに、
/// 特定の関心事に特化したロジックをカプセル化（ひとまとめに）するんだよ。캡슐💊
///
/// このトレイトを実装する構造体は、`run` メソッドを持つ必要があるよ。
/// `run` メソッドは、ゲームのメインループ（後で作る！）から定期的に呼び出されて、
/// World の中のデータ（コンポーネント）を読み取ったり、変更したりするんだ。
///
/// `&mut World` を引数に取るのは、システムが World の中身を自由に変更できるようにするためだよ。
/// 例えば、移動システムは Position コンポーネントを更新したり、
/// 戦闘システムは Health コンポーネントを減らしたりする、みたいな感じ！✏️
pub trait System {
    /// このシステムを実行するよ！
    ///
    /// # 引数
    /// - `world`: ゲーム世界のデータ（エンティティとコンポーネント）を保持する World への可変参照。
    ///           これを使って、必要なコンポーネントを取得したり、変更したりするよ。
    ///
    /// ここに具体的なゲームロジックを実装していくことになるんだ。ワクワクするね！🤩
    fn run(&mut self, world: &mut World);

    // TODO: 将来的には、セットアップ用のメソッドとか、
    //       システムが必要とするコンポーネントを事前に宣言する仕組みとかも追加できるかも？🤔
    // fn setup(&mut self, world: &mut World) {}
}

// --- 簡単な System のテスト ---
// System トレイトだけだとテストしにくいから、簡単なダミーシステムを作って、
// それが World と連携できるか軽く見てみよう！ (本格的なテストは各 System 実装時に！)
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの System トレイトを使う
    use crate::ecs::component::Component; // テスト用にダミーコンポーネントを作る
    use crate::ecs::world::World; // World を使う
    use crate::ecs::entity::Entity; // ★★★ Entity をインポート！ ★★★
    use std::collections::HashMap; // HashMap も使う

    // --- テスト用のダミーコンポーネント ---
    #[derive(Debug, Clone, PartialEq)]
    struct Position { x: i32, y: i32 }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity { dx: i32, dy: i32 }
    impl Component for Velocity {}

    // --- テスト用のダミーシステム ---
    // 全ての Position コンポーネントに Velocity コンポーネントの値を加算するシステム
    struct MovementSystem; // 中にデータを持たないシンプルなシステム

    // System トレイトを実装！
    impl System for MovementSystem {
        fn run(&mut self, world: &mut World) {
            println!("MovementSystem 実行中... 🏃");

            // ★★★ エラー回避策: Velocity 情報を先に集める (不変借用) ★★★
            let mut velocities = HashMap::new();
            if let Some(vel_storage_any) = world.storage::<Velocity>() {
                if let Some(vel_storage) = vel_storage_any.downcast_ref::<HashMap<Entity, Velocity>>() {
                    // 生きている Entity の Velocity だけをコピー
                    for (entity, vel) in vel_storage.iter() {
                        if world.is_entity_alive(*entity) { // Entity が生きているかチェック
                            velocities.insert(*entity, *vel); // Velocity は Copy なのでコピー
                        }
                    }
                } else {
                    panic!("Failed to downcast velocity storage!");
                }
            } else {
                // Velocity ストレージがない場合もある (テストによっては)
                println!("Velocity storage not found, skipping velocity collection.");
            }

            // ★★★ Position を更新する (可変借用) ★★★
            if let Some(pos_storage_any) = world.storage_mut::<Position>() {
                if let Some(pos_storage) = pos_storage_any.downcast_mut::<HashMap<Entity, Position>>() {
                    for (entity, pos) in pos_storage.iter_mut() {
                        // 先ほど集めた Velocity 情報を参照
                        if let Some(vel) = velocities.get(entity) {
                            println!("  Entity {:?}: ({}, {}) + ({}, {}) -> ({}, {})",
                                     entity, pos.x, pos.y, vel.dx, vel.dy, pos.x + vel.dx, pos.y + vel.dy);
                            pos.x += vel.dx;
                            pos.y += vel.dy;
                        } else {
                            println!("  Entity {:?}: Velocity がないので移動しません。", entity);
                        }
                    }
                } else {
                    panic!("Failed to downcast position storage!");
                }
            } else {
                println!("Position storage not found, skipping position update.");
            }

            println!("MovementSystem 実行完了！✨");
        }
    }

    #[test]
    fn dummy_system_runs_and_modifies_world() {
        // World と System を準備
        let mut world = World::new();
        let mut movement_system = MovementSystem; // 可変にするのを忘れずに！

        // コンポーネントを登録
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        // エンティティとコンポーネントを作成
        let entity1 = world.create_entity();
        world.add_component(entity1, Position { x: 0, y: 0 });
        world.add_component(entity1, Velocity { dx: 1, dy: 1 });

        let entity2 = world.create_entity();
        world.add_component(entity2, Position { x: 10, y: 10 });
        // entity2 には Velocity は付けない

        // 最初の状態を確認
        assert_eq!(world.get_component::<Position>(entity1).unwrap(), &Position { x: 0, y: 0 });
        assert_eq!(world.get_component::<Position>(entity2).unwrap(), &Position { x: 10, y: 10 });

        // システムを実行！
        println!("--- 1回目のシステム実行 ---");
        movement_system.run(&mut world);

        // システム実行後の状態を確認！
        // entity1 は (0,0) + (1,1) = (1,1) になっているはず
        assert_eq!(world.get_component::<Position>(entity1).unwrap(), &Position { x: 1, y: 1 });
        // entity2 は Velocity がないので変わらないはず
        assert_eq!(world.get_component::<Position>(entity2).unwrap(), &Position { x: 10, y: 10 });

        // もう一回システムを実行！
        println!("--- 2回目のシステム実行 ---");
        movement_system.run(&mut world);

        // 2回実行後の状態を確認！
        // entity1 は (1,1) + (1,1) = (2,2) になっているはず
        assert_eq!(world.get_component::<Position>(entity1).unwrap(), &Position { x: 2, y: 2 });
        // entity2 はやっぱり変わらない
        assert_eq!(world.get_component::<Position>(entity2).unwrap(), &Position { x: 10, y: 10 });

        println!("ダミーシステムのテスト、成功！🎉");
    }
} 