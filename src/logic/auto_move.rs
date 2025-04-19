// src/logic/auto_move.rs
//! カードの自動移動に関するロジックをまとめるモジュールだよ！🪄✨
//! どのカードがどこに自動で移動できるか、とかを判断するんだ。

// --- 必要なものをインポート ---
use crate::components::card::{Card, Suit, Rank}; // components の Card, Suit, Rank を使う
use crate::components::stack::{StackType, StackInfo}; // components の StackType, StackInfo を使う
use crate::entity::Entity; // Entity ID (crate::entity のもの)
use crate::log;           // ログ出力用 (TODO: logマクロが使えるか確認)
use crate::world::World; // 自作 World を使うため
// use crate::rules::can_move_to_foundation; // ⛔️ 古いパス！
// use crate::logic::rules::can_move_to_foundation; // ✨ 新しいパスに修正！ rules モジュールは logic の下にお引越ししたよ！
// ↑ rules モジュールの関数を直接使うので、use文を追加
use crate::logic::rules;

// --- ヘルパー関数 (このモジュール内でのみ使用) ---
// 不要になったので削除！ (get_foundation_suit と get_foundation_top_card)
// can_move_to_foundation が内部で処理してくれるようになったからね！✨

// --- 公開関数 ---

/// 特定のカードエンティティが、現在のワールドの状態において、
/// 自動的に移動できる組札（Foundation）があるかどうかを探す関数だよ。
/// 見つかった場合は、移動先の StackType (Foundation のインデックス付き) を返す。
///
/// # 引数
/// * `world`: 現在の World の状態への参照 (自作World)。
/// * `card_to_move_entity`: 移動させたいカードのエンティティID (`Entity`)。
///
/// # 戻り値
/// * `Some(StackType)`: 移動可能な組札が見つかった場合、その組札の StackType (`StackType::Foundation(index)`)。
/// * `None`: 移動可能な組札が見つからなかった場合。
pub fn find_automatic_foundation_move(
    world: &World,
    card_to_move_entity: Entity // 引数を &Card から Entity に変更！
) -> Option<StackType> {
    // どのカードをチェックしているか、Entity ID をログに出力するよ。
    log(&format!("[AutoMove] Finding automatic foundation move for Entity {:?}...", card_to_move_entity));

    // 4つの Foundation (インデックス 0 から 3 まで) を順番にチェックするループだよ。
    for i in 0..4u8 { // u8 型の 0 から 3 までループする。

        // *** 修正点 ***
        // 以前はここで移動先の Suit や Top Card を取得していたけど、
        // 新しい `rules::can_move_to_foundation` が内部で全部やってくれるようになったので、
        // それらのヘルパー呼び出しは削除！コードがスッキリ！✨

        // 移動可能かチェック！
        // `rules` モジュールにある `can_move_to_foundation` 関数を呼び出す。
        // 引数には、world への参照、移動させたいカードの Entity ID、
        // そしてチェック対象の Foundation のインデックス `i` を渡すよ。
        if rules::can_move_to_foundation(world, card_to_move_entity, i) {
            // 移動可能な Foundation が見つかった！🎉
            // どの Foundation に移動できるかログに出力する。
            log(&format!("  Found valid foundation [{}] for Entity {:?}.", i, card_to_move_entity));
            // 移動先の Foundation の StackType (例: StackType::Foundation(0)) を
            // Option::Some で包んで返す。これで関数は終了するよ。
            return Some(StackType::Foundation(i));
        }
        // もし↑の if が false なら、この Foundation には移動できないので、
        // ループは次のインデックス (次の Foundation) に進むよ。
    }

    // ループが最後まで終わっても、移動可能な Foundation が見つからなかった場合。
    log(&format!("  No suitable foundation found for Entity {:?}.", card_to_move_entity));
    // Option::None を返して、移動先がなかったことを示すよ。
    None
}

// --- テストコード (rules.rs から移動) ---
#[cfg(test)]
mod tests {
    use super::*; // このモジュール内の要素 (find_automatic_foundation_move など) を使う
    use crate::world::World; // 自作World
    use crate::components::card::{Card, Suit, Rank}; // Card関連
    use crate::components::stack::{StackType, StackInfo}; // Stack関連
    use crate::entity::Entity; // Entity を使う
    use crate::logic::rules; // rules モジュールも使う (テストデータ作成などで)
    use crate::log; // log マクロを使うため (wasm環境外のテストではprintln!の方が良いかも)

    // ヘルパー: テスト用の World に Foundation カードを追加する (仮)
    fn add_card_to_world(world: &mut World, suit: Suit, rank: Rank, stack_type: StackType, pos: u8) -> Entity {
        let entity = world.create_entity();
        let card = Card { suit, rank, is_face_up: true };
        let stack_info = StackInfo { stack_type, position_in_stack: pos };
        world.add_component(entity, card);
        world.add_component(entity, stack_info);
        entity
    }


    // --- find_automatic_foundation_move のテスト ---
    // (テストコード自体も修正が必要！ card_to_move を Entity にしないと！)
    #[test]
    fn test_find_automatic_foundation_move() {
        let mut world = World::new();
        // テスト実行前に必要なコンポーネントを登録しておく
        world.register_component::<Card>();
        world.register_component::<StackInfo>();

        // カードエンティティを作成 (World に追加して Entity ID を取得)
        let ace_hearts_entity = add_card_to_world(&mut world, Suit::Heart, Rank::Ace, StackType::Waste, 0); // 仮に Waste にあるとする
        let two_hearts_entity = add_card_to_world(&mut world, Suit::Heart, Rank::Two, StackType::Waste, 1);
        let ace_spades_entity = add_card_to_world(&mut world, Suit::Spade, Rank::Ace, StackType::Tableau(0), 0); // 仮に Tableau 0 にあるとする
        let three_hearts_entity = add_card_to_world(&mut world, Suit::Heart, Rank::Three, StackType::Tableau(1), 0);

        // --- シナリオ 1: 全 Foundation が空 ---
        log("Scenario 1: All foundations empty");
        assert_eq!(find_automatic_foundation_move(&world, ace_hearts_entity), Some(StackType::Foundation(0)), "Scenario 1: Ace of Hearts entity should move to empty Heart foundation (idx 0)");
        assert_eq!(find_automatic_foundation_move(&world, ace_spades_entity), Some(StackType::Foundation(3)), "Scenario 1: Ace of Spades entity should move to empty Spade foundation (idx 3)");
        assert_eq!(find_automatic_foundation_move(&world, two_hearts_entity), None, "Scenario 1: Two of Hearts entity cannot move to any empty foundation");

        // --- シナリオ 2: Heart Foundation に Ace of Hearts がある ---
        log("Scenario 2: Ace of Hearts on Foundation 0");
        // Foundation にカードを追加 (返り値の Entity ID は使わないけど、追加はする)
        let _foundation_ace_h = add_card_to_world(&mut world, Suit::Heart, Rank::Ace, StackType::Foundation(0), 0);
        assert_eq!(find_automatic_foundation_move(&world, two_hearts_entity), Some(StackType::Foundation(0)), "Scenario 2: Two of Hearts entity should move to Heart foundation with Ace");
        assert_eq!(find_automatic_foundation_move(&world, ace_spades_entity), Some(StackType::Foundation(3)), "Scenario 2: Ace of Spades entity should still move to empty Spade foundation (idx 3)");
        // TODO: テスト後に World の状態をリセットするか、個別に Entity を削除する必要がある
        //       現状だと前のテストの Entity が残ってしまう可能性がある

        // --- シナリオ 3: Heart Foundation に Ace と Two がある ---
        log("Scenario 3: Ace and Two of Hearts on Foundation 0");
        // 前のテストの Foundation Ace は残ってるはず…？ (World リセットしてないので)
        // なので Two だけ追加
        let _foundation_two_h = add_card_to_world(&mut world, Suit::Heart, Rank::Two, StackType::Foundation(0), 1);
        assert_eq!(find_automatic_foundation_move(&world, three_hearts_entity), Some(StackType::Foundation(0)), "Scenario 3: Three of Hearts entity should move to Heart foundation with Two");
        assert_eq!(find_automatic_foundation_move(&world, ace_spades_entity), Some(StackType::Foundation(3)), "Scenario 3: Ace of Spades entity should still move");


        println!("Automatic Foundation Move テスト (修正後)、成功！🎉");
    }
} 