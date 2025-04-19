// src/system.rs

// これまで作った World を使うからインポートするよ。
use crate::world::World;

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
    use crate::component::Component; // テスト用にダミーコンポーネントを作る
    use crate::world::World; // World を使う

    // --- テスト用のダミーコンポーネント ---
    #[derive(Debug, Clone, PartialEq)]
    struct Position { x: i32, y: i32 }
    impl Component for Position {}

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity { dx: i32, dy: i32 }
    impl Component for Velocity {}

    // --- テスト用のダミーシステム ---
    // 全ての Position コンポーネントに Velocity コンポーネントの値を加算するシステム
    struct MovementSystem; // 中にデータを持たないシンプルなシステム

    // System トレイトを実装！
    impl System for MovementSystem {
        fn run(&mut self, world: &mut World) {
            println!("MovementSystem 実行中... 🏃💨");

            // Position と Velocity 両方のストレージへの可変参照を取得する。
            // Option<T> を unwrap() してるけど、テストだからOK！ 本番コードではちゃんとエラー処理しようね！🙏
            // `.expect()` を使った方が、エラーメッセージが出て親切かもね！
            let pos_storage = world.storage_mut::<Position>().expect("Position storage not found!");
            let vel_storage = world.storage::<Velocity>().expect("Velocity storage not found!"); // Velocityは読み取り専用でOK

            // Position ストレージをイテレートして、各エンティティの Position を更新！
            // iter_mut() を使って、Position を直接変更できるようにするよ。
            for (entity, pos) in pos_storage.iter_mut() {
                // 同じエンティティに対応する Velocity があるか確認する。
                if let Some(vel) = vel_storage.get(*entity) {
                    // Velocity があれば、Position に加算！
                    println!("  Entity {:?}: ({}, {}) + ({}, {}) -> ({}, {})",
                             entity, pos.x, pos.y, vel.dx, vel.dy, pos.x + vel.dx, pos.y + vel.dy);
                    pos.x += vel.dx;
                    pos.y += vel.dy;
                } else {
                    println!("  Entity {:?}: Velocity がないので移動しません。", entity);
                }
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