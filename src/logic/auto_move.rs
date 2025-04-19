// src/logic/auto_move.rs
//! カードの自動移動に関するロジックをまとめるモジュールだよ！🪄✨
//! どのカードがどこに自動で移動できるか、とかを判断するんだ。

// --- 必要なものをインポート ---
use crate::components::card::{Card, Suit, Rank}; // components の Card, Suit, Rank を使う
use crate::components::stack::{StackType, StackInfo}; // components の StackType, StackInfo を使う
use crate::entity::Entity; // Entity ID (crate::entity のもの)
use crate::log;           // ログ出力用 (TODO: logマクロが使えるか確認)
use crate::world::World; // 自作 World を使うため
use crate::rules::can_move_to_foundation; // 基本的なルールチェック関数を rules から使う

// --- ヘルパー関数 (このモジュール内でのみ使用) ---

/// 組札 (Foundation) のインデックス (0-3) から対応するスートを取得する。
/// 約束事: 0: Heart, 1: Diamond, 2: Club, 3: Spade
/// (rules.rs から移動してきたよ！)
pub(crate) fn get_foundation_suit(foundation_index: u8) -> Option<Suit> {
    match foundation_index {
        0 => Some(Suit::Heart),
        1 => Some(Suit::Diamond),
        2 => Some(Suit::Club),
        3 => Some(Suit::Spade),
        _ => None, // 0-3 以外は無効なインデックス
    }
}

/// 指定された組札 (Foundation) の一番上にあるカードを取得するヘルパー関数。
/// World の状態を調べて、StackInfo を持つエンティティから見つける。
/// (rules.rs から移動してきたよ！)
/// TODO: この実装は自作Worldの get_all_entities_with_component や get_component に依存。動作確認が必要。
pub(crate) fn get_foundation_top_card<'a>(world: &'a World, foundation_index: u8) -> Option<&'a Card> {
    // 1. Card と StackInfo コンポーネントを持つ全てのエンティティを取得する。
    let entities_with_card = world.get_all_entities_with_component::<Card>();

    // 2. StackInfo を見て、stack_type が Foundation(foundation_index) に一致するものを探す。
    // 3. 見つかったエンティティの中で、position_in_stack が最大のものを探す。
    let top_entity = entities_with_card
        .iter()
        // StackInfo を持つエンティティのみをフィルタリングし、(Entity, &StackInfo) のタプルにする
        .filter_map(|&entity| {
            world.get_component::<StackInfo>(entity).map(|stack_info| (entity, stack_info))
        })
        // 指定された Foundation インデックスに一致するスタックを持つものだけをフィルタリング
        .filter(|(_, stack_info)| stack_info.stack_type == StackType::Foundation(foundation_index))
        // position_in_stack が最大のものを探す
        .max_by_key(|(_, stack_info)| stack_info.position_in_stack)
        // 最大の pos_in_stack を持つエンティティ (Entity) を返す (なければ None)
        .map(|(entity, _)| entity);

    // 4. そのエンティティの Card コンポーネントへの参照を返す。
    top_entity.and_then(|entity| world.get_component::<Card>(entity))
}

// --- 公開関数 ---

/// 特定のカードが、現在のワールドの状態において、自動的に移動できる組札（Foundation）があるかどうかを探す関数。
/// 見つかった場合は、移動先の StackType (Foundation のインデックス付き) を返す。
/// (rules.rs から移動してきたよ！)
///
/// # 引数
/// - `card_to_move`: 移動させたいカードのコンポーネントへの参照 (`component::Card`)。
/// - `world`: 現在の World の状態への参照 (自作World)。
///
/// # 戻り値
/// - `Some(StackType)`: 移動可能な組札が見つかった場合、その組札の StackType (`component::StackType`)。
///                     注意: StackType::Foundation(index) の形で返すよ！
/// - `None`: 移動可能な組札が見つからなかった場合。
pub fn find_automatic_foundation_move<'a>(
    card_to_move: &Card,
    world: &'a World
) -> Option<StackType> {
    log(&format!("[AutoMove] Finding automatic foundation move for {:?}...", card_to_move));

    for i in 0..4u8 { // 4つの Foundation をチェック
        let foundation_suit = get_foundation_suit(i); // このモジュール内の関数を呼ぶ

        if foundation_suit.is_none() { continue; } // 無効なインデックスはスキップ
        let foundation_suit = foundation_suit.unwrap();

        // Foundation の一番上のカードを取得 (このモジュール内のヘルパー関数を呼ぶ)
        let foundation_top_card: Option<&Card> = get_foundation_top_card(world, i);

        // 移動可能かチェック (rules モジュールの関数を呼ぶ)
        if can_move_to_foundation(card_to_move, foundation_top_card, foundation_suit) {
            log(&format!("  Found valid foundation [{}] for {:?}. Top card: {:?}", i, card_to_move, foundation_top_card));
            // 移動可能な Foundation が見つかったので、StackType::Foundation(i) を返す
            return Some(StackType::Foundation(i));
        }
    }

    log(&format!("  No suitable foundation found for {:?}.", card_to_move));
    None // 適切な移動先が見つからなかった
}

// --- テストコード (rules.rs から移動) ---
#[cfg(test)]
mod tests {
    use super::*; // このモジュール内の要素 (find_automatic_foundation_move など) を使う
    use crate::world::World; // 自作World
    use crate::components::card::{Card, Suit, Rank}; // Card関連
    use crate::components::stack::{StackType, StackInfo}; // Stack関連
    use crate::entity::Entity; // Entity を使う
    use crate::log; // log マクロを使うため (wasm環境外のテストではprintln!の方が良いかも)

    // ヘルパー: テスト用の World に Foundation カードを追加する (仮)
    fn add_card_to_foundation(world: &mut World, suit: Suit, rank: Rank, index: u8, pos: u8) -> Entity {
        let entity = world.create_entity();
        let card = Card { suit, rank, is_face_up: true };
        let stack_info = StackInfo { stack_type: StackType::Foundation(index), position_in_stack: pos };
        world.add_component(entity, card);
        world.add_component(entity, stack_info);
        entity
    }


    // --- find_automatic_foundation_move のテスト ---
    #[test]
    fn test_find_automatic_foundation_move() {
        let mut world = World::new();
        // テスト実行前に必要なコンポーネントを登録しておく
        world.register_component::<Card>();
        world.register_component::<StackInfo>();

        // カードの準備
        let ace_hearts = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_hearts = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let ace_spades = Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true };
        let three_hearts = Card { suit: Suit::Heart, rank: Rank::Three, is_face_up: true };

        // --- シナリオ 1: 全 Foundation が空 ---
        log("Scenario 1: All foundations empty");
        // get_foundation_top_card が (実装により) 正しく None を返すはず
        assert_eq!(find_automatic_foundation_move(&ace_hearts, &world), Some(StackType::Foundation(0)), "Scenario 1: Ace of Hearts should move to empty Heart foundation (idx 0)");
        assert_eq!(find_automatic_foundation_move(&ace_spades, &world), Some(StackType::Foundation(3)), "Scenario 1: Ace of Spades should move to empty Spade foundation (idx 3)");
        assert_eq!(find_automatic_foundation_move(&two_hearts, &world), None, "Scenario 1: Two of Hearts cannot move to any empty foundation");

        // --- シナリオ 2: Heart Foundation に Ace of Hearts がある ---
        log("Scenario 2: Ace of Hearts on Foundation 0");
        let entity_ace_h_s2 = add_card_to_foundation(&mut world, Suit::Heart, Rank::Ace, 0, 0); // pos 0
        // get_foundation_top_card が Ace を返すはず
        assert_eq!(find_automatic_foundation_move(&two_hearts, &world), Some(StackType::Foundation(0)), "Scenario 2: Two of Hearts should move to Heart foundation with Ace");
        assert_eq!(find_automatic_foundation_move(&ace_spades, &world), Some(StackType::Foundation(3)), "Scenario 2: Ace of Spades should still move to empty Spade foundation (idx 3)");
        assert!(world.destroy_entity(entity_ace_h_s2), "Scenario 2: Failed to destroy test entity"); // テスト後にエンティティを削除

        // --- シナリオ 3: Heart Foundation に Two of Hearts がある ---
        log("Scenario 3: Two of Hearts on Foundation 0");
        let entity_ace_h_s3 = add_card_to_foundation(&mut world, Suit::Heart, Rank::Ace, 0, 0);
        let entity_two_h_s3 = add_card_to_foundation(&mut world, Suit::Heart, Rank::Two, 0, 1); // pos 1
        // get_foundation_top_card が Two を返すはず
        assert_eq!(find_automatic_foundation_move(&three_hearts, &world), Some(StackType::Foundation(0)), "Scenario 3: Three of Hearts should move to Heart foundation with Two");
        // 他のカードが影響を受けないことも確認
        assert_eq!(find_automatic_foundation_move(&ace_spades, &world), Some(StackType::Foundation(3)), "Scenario 3: Ace of Spades should still move");
        assert!(world.destroy_entity(entity_ace_h_s3), "Scenario 3: Failed to destroy test entity ace");
        assert!(world.destroy_entity(entity_two_h_s3), "Scenario 3: Failed to destroy test entity two");


        println!("Automatic Foundation Move テスト (get_foundation_top_card実装後)、成功！🎉");
    }
} 