// src/logic/rules/tests.rs
//! rules モジュール内の関数のユニットテスト。

use super::*; // 親モジュール (rules/mod.rs 経由で各ルール関数が re-export されてるはず) の要素を使う
use crate::components::card::{Rank, Suit, Card}; // Card, Suit, Rank を使う
use crate::ecs::world::World; // World を使う
use crate::ecs::entity::Entity; // Entity を使う
use crate::components::stack::{StackType, StackInfo}; // StackType, StackInfo を使う

// --- テスト用ヘルパー関数 ---
/// テストワールドにカードエンティティを追加するヘルパー関数だよ。
fn add_card_for_test(world: &mut World, suit: Suit, rank: Rank, stack_type: StackType, pos: u8) -> Entity {
    // 新しいエンティティを作成
    let entity = world.create_entity();
    // カードコンポーネントを作成 (is_face_up は常に true でテストするよ)
    let card = Card { suit, rank, is_face_up: true };
    // スタック情報コンポーネントを作成
    let stack_info = StackInfo { stack_type, position_in_stack: pos };
    // 作成したエンティティにコンポーネントを追加
    world.add_component(entity, card);
    world.add_component(entity, stack_info);
    // 作成したエンティティの ID を返す
    entity
}

// --- 各ルール関数のテスト ---

#[test]
fn test_card_color() {
    assert_eq!(CardColor::from_suit(Suit::Heart), CardColor::Red);
    assert_eq!(CardColor::from_suit(Suit::Diamond), CardColor::Red);
    assert_eq!(CardColor::from_suit(Suit::Club), CardColor::Black);
    assert_eq!(CardColor::from_suit(Suit::Spade), CardColor::Black);
    println!("CardColor テスト、成功！🎉");
}

/* // TODO: World を使う can_move_to_foundation のテストを実装する！
#[test]
fn test_can_move_to_foundation_rules() {
    // ... (World をセットアップするコード)
    println!("Foundation 移動ルールテスト、成功！🎉");
}
*/

#[test]
fn test_stock_waste_rules() {
    // ストックがある場合
    assert!(can_deal_from_stock(false), "ストックがあれば配れるはず");
    assert!(!can_reset_stock_from_waste(false, false), "ストックがある場合はリセットできないはず");
    assert!(!can_reset_stock_from_waste(false, true), "ストックがある場合はリセットできないはず");

    // ストックが空の場合
    assert!(!can_deal_from_stock(true), "ストックが空なら配れないはず");
    // ★修正: waste_is_empty が false (つまりウェストにカードがある) 場合に true を期待する
    assert!(can_reset_stock_from_waste(true, false), "ストックが空でウェストにあればリセットできるはず");
    assert!(!can_reset_stock_from_waste(true, true), "ストックもウェストも空ならリセットできないはず");
    println!("Stock/Waste ルールテスト、成功！🎉");
}

#[test]
fn test_win_condition() {
    assert!(check_win_condition(52), "カードが52枚あればクリアなはず！🏆");
    assert!(!check_win_condition(51), "カードが51枚ではクリアじゃないはず！🙅");
    assert!(!check_win_condition(0), "カードが0枚ではクリアじゃないはず！🙅");
    println!("ゲームクリア判定テスト、成功！🎉");
}

// --- World を使うテスト --- 

#[test]
fn test_can_move_to_tableau_world() {
    println!("--- test_can_move_to_tableau_world 開始 ---");
    // --- 準備 ---
    let mut world = World::new();
    world.register_component::<Card>();
    world.register_component::<StackInfo>();

    // --- テストカードエンティティの作成 ---
    let king_spades_entity = add_card_for_test(&mut world, Suit::Spade, Rank::King, StackType::Waste, 0);
    let queen_hearts_entity = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Waste, 1);
    let jack_spades_entity = add_card_for_test(&mut world, Suit::Spade, Rank::Jack, StackType::Waste, 2);
    let jack_diamonds_entity = add_card_for_test(&mut world, Suit::Diamond, Rank::Jack, StackType::Waste, 3);
    let ten_spades_entity = add_card_for_test(&mut world, Suit::Spade, Rank::Ten, StackType::Waste, 4);

    // --- シナリオ 1: 空の Tableau への移動 ---
    println!("Scenario 1: 空の Tableau への移動");
    assert!(
        can_move_to_tableau(&world, king_spades_entity, 0),
        "空の Tableau 0 に King of Spades は置けるはず"
    );
    assert!(
        !can_move_to_tableau(&world, queen_hearts_entity, 1),
        "空の Tableau 1 に Queen of Hearts は置けないはず"
    );

    // --- シナリオ 2: 空でない Tableau への有効な移動 ---
    println!("Scenario 2: 空でない Tableau への有効な移動");
    let _target_q_hearts_t2 = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Tableau(2), 0);
    assert!(
        can_move_to_tableau(&world, jack_spades_entity, 2),
        "Tableau 2 (Q❤️) に Jack of Spades (黒) は置けるはず"
    );

    // --- シナリオ 3: 空でない Tableau への無効な移動 (同色) ---
    println!("Scenario 3: 空でない Tableau への無効な移動 (同色)");
    let _target_q_hearts_t3 = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Tableau(3), 0);
    assert!(
        !can_move_to_tableau(&world, jack_diamonds_entity, 3),
        "Tableau 3 (Q❤️) に Jack of Diamonds (赤) は置けないはず (同色)"
    );

    // --- シナリオ 4: 空でない Tableau への無効な移動 (ランク違い) ---
    println!("Scenario 4: 空でない Tableau への無効な移動 (ランク違い)");
    let _target_q_hearts_t4 = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Tableau(4), 0);
    assert!(
        !can_move_to_tableau(&world, ten_spades_entity, 4),
        "Tableau 4 (Q❤️) に Ten of Spades (黒) は置けないはず (ランク違い)"
    );

    println!("--- test_can_move_to_tableau_world 完了 ---");
}

// TODO: World を使う can_move_to_foundation のテストを追加する
// TODO: World を使う can_move_from_waste_to_tableau/foundation のテストを追加する 