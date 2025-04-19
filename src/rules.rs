//! ソリティアのゲームルール判定ロジックをまとめるモジュールだよ！🃏✅
//!
//! ここに関数を追加していくことで、カードがどこからどこへ移動できるか、
//! といったルールをチェックできるようにするんだ。

// 必要な型をインポートしておくよ！
use crate::component::{Card, Suit, Rank}; // カード情報 (component に変更)
use crate::component::{StackInfo, StackType}; // スタックの情報と種類 (component に変更)
// use crate::world::World;                        // ゲーム世界の全体像 <-- これは使わない！
use crate::entity::Entity;                      // エンティティID (これは crate::entity のもの)
use crate::log;
use hecs::{World as HecsWorld, Entity as HecsEntity}; // <-- hecs の型に別名をつける！

// TODO: 必要に応じて他のコンポーネントや型もインポートする！

/// カードの色（赤か黒か）を表すヘルパーenumだよ。
/// 場札 (Tableau) への移動ルール (色違い) で使う！❤️🖤
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CardColor {
    Red,
    Black,
}

impl CardColor {
    /// スートからカードの色を取得する関数。
    pub fn from_suit(suit: Suit) -> Self {
        match suit {
            Suit::Heart | Suit::Diamond => CardColor::Red, // ハートとダイヤは赤！♦️❤️
            Suit::Club | Suit::Spade => CardColor::Black,  // クラブとスペードは黒！♣️♠️
        }
    }
}

// --- カード移動の基本ルールチェック関数 ---
// これからここに具体的なルールチェック関数を追加していくよ！

/// 指定されたカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
///
/// # 引数
/// * `card_to_move`: 移動させようとしているカード (component::Card)。
/// * `foundation_top_card`: 移動先の組札の一番上にあるカード (component::Card, なければ None)。
/// * `foundation_suit`: 移動先の組札のスート (component::Suit)。
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_to_foundation(
    card_to_move: &Card, // component::Card を参照
    foundation_top_card: Option<&Card>, // component::Card を参照
    foundation_suit: Suit, // component::Suit を参照
) -> bool {
    // 1. スートが一致しているか？ (component::Suit 同士の比較)
    if card_to_move.suit != foundation_suit {
        return false; // スートが違うなら置けない！🙅‍♀️
    }

    // 2. ランクが正しいか？
    match foundation_top_card {
        // 組札が空の場合 (一番上のカードがない場合)
        None => {
            // エース (A) なら置ける！👑 (component::Rank 同士の比較)
            card_to_move.rank == Rank::Ace
        }
        // 組札に既にカードがある場合
        Some(top_card) => {
            // 移動するカードのランクが、一番上のカードのランクの「次」なら置ける！
            // (例: 上が A なら 2、上が 10 なら J)
            // Rank enum は Ord を実装してるので、大小比較ができる！
            // `as usize` で数値に変換して比較する方が確実かも？🤔
            // (component::Rank 同士の比較)
            (card_to_move.rank as usize) == (top_card.rank as usize) + 1
        }
    }
}

/// 指定されたカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
///
/// # 引数
/// * `card_to_move`: 移動させようとしているカード (component::Card)。
/// * `tableau_top_card`: 移動先の場札の一番上にあるカード (component::Card, 空の列なら None)。
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_to_tableau(
    card_to_move: &Card, // component::Card を参照
    tableau_top_card: Option<&Card>, // component::Card を参照
) -> bool {
    match tableau_top_card {
        // 場札の列が空の場合
        None => {
            // キング (K) なら置ける！🤴 (component::Rank 同士の比較)
            card_to_move.rank == Rank::King
        }
        // 場札の列に既にカードがある場合
        Some(top_card) => {
            // 1. 色が違うか？ (赤と黒)
            let move_color = CardColor::from_suit(card_to_move.suit); // component::Suit を使用
            let target_color = CardColor::from_suit(top_card.suit); // component::Suit を使用
            if move_color == target_color {
                return false; // 同じ色なら重ねられない！🟥🟥 or ⬛️⬛️ はダメ！
            }

            // 2. ランクが連続しているか？ (移動するカードが1つ小さい)
            // (例: 上が Q なら J、上が 7 なら 6)
            // (component::Rank 同士の比較)
            (card_to_move.rank as usize) == (top_card.rank as usize) - 1
        }
    }
}

/// ストック（山札）からウェスト（捨て札）にカードを配れるかチェックする。
/// (この関数は単純化されており、実際には World の状態を見る必要があるかもしれない)
///
/// # 引数
/// * `stock_is_empty`: ストックが現在空かどうか。
///
/// # 戻り値
/// * ストックから配れるなら `true`、そうでなければ `false`。
pub fn can_deal_from_stock(stock_is_empty: bool) -> bool {
    !stock_is_empty // ストックが空でなければ配れる
}

/// ストック（山札）が空のときに、ウェスト（捨て札）からストックにカードを戻せるかチェックする。
/// (この関数は単純化されており、実際には World の状態を見る必要があるかもしれない)
///
/// # 引数
/// * `stock_is_empty`: ストックが現在空かどうか。
/// * `waste_is_empty`: ウェストが現在空かどうか。
///
/// # 戻り値
/// * ウェストからストックに戻せる（リセットできる）なら `true`、そうでなければ `false`。
pub fn can_reset_stock_from_waste(stock_is_empty: bool, waste_is_empty: bool) -> bool {
    stock_is_empty && !waste_is_empty // ストックが空で、ウェストにカードがあればリセットできる
}

/// ウェスト（捨て札）の一番上のカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
///
/// # 引数
/// * `waste_top_card`: 移動させようとしているウェストの一番上のカード。
/// * `tableau_top_card`: 移動先の場札の一番上にあるカード (空の列なら None)。
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_from_waste_to_tableau(
    waste_top_card: &Card, // component::Card を参照
    tableau_top_card: Option<&Card>, // component::Card を参照
) -> bool {
    // 基本的には Tableau への移動ルールと同じだよ！✨
    can_move_to_tableau(waste_top_card, tableau_top_card)
}

/// ウェスト（捨て札）の一番上のカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
///
/// # 引数
/// * `waste_top_card`: 移動させようとしているウェストの一番上のカード。
/// * `foundation_top_card`: 移動先の組札の一番上にあるカード (なければ None)。
/// * `foundation_suit`: 移動先の組札のスート。
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_from_waste_to_foundation(
    waste_top_card: &Card, // component::Card を参照
    foundation_top_card: Option<&Card>, // component::Card を参照
    foundation_suit: Suit, // component::Suit を参照
) -> bool {
    // 基本的には Foundation への移動ルールと同じだよ！💖
    can_move_to_foundation(waste_top_card, foundation_top_card, foundation_suit)
}

/// ゲームのクリア条件（全てのカードが組札にあるか）を判定する。
///
/// # 引数
/// * `foundation_card_count`: 現在、全ての組札（Foundation）にあるカードの合計枚数。
///
/// # 戻り値
/// * クリア条件を満たしていれば `true`、そうでなければ `false`。
pub fn check_win_condition(foundation_card_count: usize) -> bool {
    foundation_card_count == 52 // 標準的な52枚デッキの場合
}

// --- 自動移動関連のヘルパー関数 ---

/// 組札 (Foundation) のインデックス (0-3) から対応するスートを取得する。
/// 約束事: 0: Heart, 1: Diamond, 2: Club, 3: Spade
/// 戻り値も component::Suit にする！
fn get_foundation_suit(foundation_index: u8) -> Option<Suit> {
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
fn get_foundation_top_card<'a>(world: &'a HecsWorld, foundation_index: u8) -> Option<&'a Card> { // 引数を HecsWorld に！
    let mut top_entity: Option<HecsEntity> = None; // Option<HecsEntity> に！
    let mut max_pos_in_stack: i16 = -1;

    // StackInfo を持つエンティティを走査 (world は HecsWorld なので query が使える！) 
    for (entity, stack_info) in world.query::<&StackInfo>().iter() {
        // 目的の Foundation スタック (タイプとインデックスが一致) かつ最大の position_in_stack を持つものを探す
        if stack_info.stack_type == StackType::Foundation && stack_info.stack_index == foundation_index {
            if (stack_info.position_in_stack as i16) > max_pos_in_stack {
                max_pos_in_stack = stack_info.position_in_stack as i16;
                top_entity = Some(entity); // entity は HecsEntity
            }
        }
    }

    // 見つかった一番上のエンティティから Card コンポーネントを取得 (world は HecsWorld なので query_one が使える！) 
    top_entity.and_then(|entity| world.query_one::<&Card>(entity).ok().map(|mut query| query.get().expect("Top entity should have Card")))
}

/// 特定のカードが、現在のワールドの状態において、自動的に移動できる組札（Foundation）があるかどうかを探す関数。
/// 見つかった場合は、移動先の StackType (Foundation のインデックス付き) を返す。
///
/// # 引数
/// - `card_to_move`: 移動させたいカードのコンポーネントへの参照 (`component::Card`)。
/// - `world`: 現在の World の状態への参照 (`hecs::World`)。 
///
/// # 戻り値
/// - `Some(StackType)`: 移動可能な組札が見つかった場合、その組札の StackType (`component::StackType`)。
///                     注意: 現在の StackType::Foundation はインデックスを持たないため、どの Foundation かは別途判断が必要。
/// - `None`: 移動可能な組札が見つからなかった場合。
pub fn find_automatic_foundation_move<'a>(
    card_to_move: &Card, // component::Card
    world: &'a HecsWorld // 引数を HecsWorld に！ 
) -> Option<StackType> { // component::StackType
    log(&format!("[Rules] Finding automatic foundation move for {:?}...", card_to_move));

    for i in 0..4u8 { // 4つの Foundation をチェック
        let target_stack_type = StackType::Foundation; // StackType::Foundation を直接使う
        let foundation_suit = get_foundation_suit(i);

        if foundation_suit.is_none() { continue; } // 無効なインデックスはスキップ
        let foundation_suit = foundation_suit.unwrap(); // component::Suit

        // Foundation の一番上のカードを取得 (get_foundation_top_card は HecsWorld を取るように修正済み) 
        let foundation_top_card: Option<&Card> = get_foundation_top_card(world, i);

        // 移動可能かチェック (can_move_to_foundation は component の型を期待するように修正済み)
        if can_move_to_foundation(card_to_move, foundation_top_card, foundation_suit) {
            log(&format!("  Found valid foundation [{}] for {:?}. Top card: {:?}", i, card_to_move, foundation_top_card));
            // 移動可能な Foundation が見つかったので、StackType::Foundation を返す
            return Some(target_stack_type);
        }
    }

    log(&format!("  No suitable foundation found for {:?}.", card_to_move));
    None // 適切な移動先が見つからなかった
}

// TODO: 他の移動パターン (Stock -> Waste, Waste -> Tableau/Foundation など) の
//       ルールチェック関数も必要に応じて追加していく！💪

// --- テストコード ---
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの要素を使う
    use crate::component::Rank; // Rank も使う
    // use hecs::World; // <-- コメントアウトのまま or HecsWorld を使う
    use hecs::{World as HecsWorld, Entity as HecsEntity}; // テスト内でも HecsWorld を使う！

    // --- 既存のテスト ... ---
    #[test]
    fn test_card_color() {
        assert_eq!(CardColor::from_suit(Suit::Heart), CardColor::Red);
        assert_eq!(CardColor::from_suit(Suit::Diamond), CardColor::Red);
        assert_eq!(CardColor::from_suit(Suit::Club), CardColor::Black);
        assert_eq!(CardColor::from_suit(Suit::Spade), CardColor::Black);
        println!("CardColor テスト、成功！🎉");
    }

    #[test]
    fn test_can_move_to_foundation_rules() {
        // テスト用のカードを作成 (component::Card)
        let ace_hearts = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_hearts = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let three_hearts = Card { suit: Suit::Heart, rank: Rank::Three, is_face_up: true };
        let ace_spades = Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true };

        // --- Foundation が空の場合 ---
        assert!(can_move_to_foundation(&ace_hearts, None, Suit::Heart), "空のHeart Foundation に Ace of Hearts は置けるはず");
        assert!(!can_move_to_foundation(&two_hearts, None, Suit::Heart), "空のHeart Foundation に 2 of Hearts は置けないはず");
        assert!(!can_move_to_foundation(&ace_spades, None, Suit::Heart), "空のHeart Foundation に Ace of Spades は置けないはず");

        // --- Foundation に Ace がある場合 ---
        assert!(can_move_to_foundation(&two_hearts, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 2 of Hearts は置けるはず");
        assert!(!can_move_to_foundation(&three_hearts, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 3 of Hearts は置けないはず");
        let two_spades = Card { suit: Suit::Spade, rank: Rank::Two, is_face_up: true };
        assert!(!can_move_to_foundation(&two_spades, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 2 of Spades は置けないはず");

        // --- Foundation に 2 がある場合 ---
        assert!(can_move_to_foundation(&three_hearts, Some(&two_hearts), Suit::Heart), "Heart Foundation (Two) に 3 of Hearts は置けるはず");

        println!("Foundation 移動ルールテスト、成功！🎉");
    }

     #[test]
    fn test_can_move_to_tableau_rules() {
        // テスト用カード (component::Card)
        let king_spades = Card { suit: Suit::Spade, rank: Rank::King, is_face_up: true };
        let queen_hearts = Card { suit: Suit::Heart, rank: Rank::Queen, is_face_up: true };
        let jack_diamonds = Card { suit: Suit::Diamond, rank: Rank::Jack, is_face_up: true };
        let jack_spades = Card { suit: Suit::Spade, rank: Rank::Jack, is_face_up: true };
        let ten_hearts = Card { suit: Suit::Heart, rank: Rank::Ten, is_face_up: true };

        // --- Tableau が空の場合 ---
        assert!(can_move_to_tableau(&king_spades, None), "空の Tableau に King of Spades は置けるはず");
        assert!(!can_move_to_tableau(&queen_hearts, None), "空の Tableau に Queen of Hearts は置けないはず");

        // --- Tableau に Queen of Hearts (赤) がある場合 ---
        assert!(can_move_to_tableau(&jack_spades, Some(&queen_hearts)), "Tableau (Q❤️) に J♠️ は置けるはず");
        assert!(!can_move_to_tableau(&jack_diamonds, Some(&queen_hearts)), "Tableau (Q❤️) に J♦️ は置けないはず (同色)");
        let ten_clubs = Card { suit: Suit::Club, rank: Rank::Ten, is_face_up: true };
        assert!(!can_move_to_tableau(&ten_clubs, Some(&queen_hearts)), "Tableau (Q❤️) に 10♣️ は置けないはず (ランク違い)");

        // --- Tableau に Jack of Spades (黒) がある場合 ---
        assert!(can_move_to_tableau(&ten_hearts, Some(&jack_spades)), "Tableau (J♠️) に 10❤️ は置けるはず");

        println!("Tableau 移動ルールテスト、成功！🎉");
    }

    #[test]
    fn test_stock_waste_rules() {
        // ストックがある場合
        assert!(can_deal_from_stock(false), "ストックがあれば配れるはず");
        assert!(!can_reset_stock_from_waste(false, false), "ストックがある場合はリセットできないはず");
        assert!(!can_reset_stock_from_waste(false, true), "ストックがある場合はリセットできないはず");

        // ストックが空の場合
        assert!(!can_deal_from_stock(true), "ストックが空なら配れないはず");
        assert!(can_reset_stock_from_waste(true, false), "ストックが空でウェストにあればリセットできるはず");
        assert!(!can_reset_stock_from_waste(true, true), "ストックもウェストも空ならリセットできないはず");
        println!("Stock/Waste ルールテスト、成功！🎉");
    }

    #[test]
    fn test_can_move_from_waste_rules() {
        // テスト用カード (component::Card)
        let queen_hearts = Card { suit: Suit::Heart, rank: Rank::Queen, is_face_up: true };
        let jack_spades = Card { suit: Suit::Spade, rank: Rank::Jack, is_face_up: true };
        let king_spades = Card { suit: Suit::Spade, rank: Rank::King, is_face_up: true };

        let ace_hearts = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_hearts = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let ace_clubs = Card { suit: Suit::Club, rank: Rank::Ace, is_face_up: true };

        // --- Waste から Tableau への移動テスト ---
        // 基本的に can_move_to_tableau と同じロジックなので、代表的なケースを確認
        assert!(can_move_from_waste_to_tableau(&jack_spades, Some(&queen_hearts)), "Waste(J♠️) から Tableau(Q❤️) へ移動できるはず");
        assert!(!can_move_from_waste_to_tableau(&jack_spades, Some(&king_spades)), "Waste(J♠️) から Tableau(K♠️) へは移動できないはず (同色)");
        assert!(can_move_from_waste_to_tableau(&king_spades, None), "Waste(K♠️) から 空の Tableau へ移動できるはず");
        assert!(!can_move_from_waste_to_tableau(&queen_hearts, None), "Waste(Q❤️) から 空の Tableau へは移動できないはず");

        // --- Waste から Foundation への移動テスト ---
        // 基本的に can_move_to_foundation と同じロジックなので、代表的なケースを確認
        assert!(can_move_from_waste_to_foundation(&ace_hearts, None, Suit::Heart), "Waste(A❤️) から 空の Heart Foundation へ移動できるはず");
        assert!(!can_move_from_waste_to_foundation(&ace_clubs, None, Suit::Heart), "Waste(A♣️) から 空の Heart Foundation へは移動できないはず (スート違い)");
        assert!(can_move_from_waste_to_foundation(&two_hearts, Some(&ace_hearts), Suit::Heart), "Waste(2❤️) から Heart Foundation(A❤️) へ移動できるはず");
        assert!(!can_move_from_waste_to_foundation(&two_hearts, Some(&ace_clubs), Suit::Club), "Waste(2❤️) から Club Foundation(A♣️) へは移動できないはず (スート違い)");

        println!("Waste からの移動ルールテスト、成功！🎉");
    }

    #[test]
    fn test_win_condition() {
        assert!(check_win_condition(52), "カードが52枚あればクリアなはず！🏆");
        assert!(!check_win_condition(51), "カードが51枚ではクリアじゃないはず！🙅");
        assert!(!check_win_condition(0), "カードが0枚ではクリアじゃないはず！🙅");
        println!("ゲームクリア判定テスト、成功！🎉");
    }

    // --- find_automatic_foundation_move のテストを追加 --- 
    #[test]
    fn test_find_automatic_foundation_move() {
        let mut world = HecsWorld::new(); // HecsWorld::new() で初期化！ 

        // Foundation 用のエンティティとStackInfo (component::StackInfo)
        // entity も HecsEntity 
        let _foundation0_entity: HecsEntity = world.spawn((StackInfo { stack_type: StackType::Foundation, stack_index: 0, position_in_stack: 0 },));
        let _foundation1_entity: HecsEntity = world.spawn((StackInfo { stack_type: StackType::Foundation, stack_index: 1, position_in_stack: 0 },));
        let _foundation2_entity: HecsEntity = world.spawn((StackInfo { stack_type: StackType::Foundation, stack_index: 2, position_in_stack: 0 },));
        let _foundation3_entity: HecsEntity = world.spawn((StackInfo { stack_type: StackType::Foundation, stack_index: 3, position_in_stack: 0 },));


        // カードの準備 (component::Card)
        let ace_hearts = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_hearts = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let ace_spades = Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true };
        let three_hearts = Card { suit: Suit::Heart, rank: Rank::Three, is_face_up: true }; // シナリオ3で使用

        // --- シナリオ 1: 全 Foundation が空 ---
        log("Scenario 1: All foundations empty");
        // assert 文では world (&HecsWorld) を渡す 
        assert_eq!(find_automatic_foundation_move(&ace_hearts, &world), Some(StackType::Foundation), "Ace of Hearts should move to empty Heart foundation (idx 0)");
        assert_eq!(find_automatic_foundation_move(&ace_spades, &world), Some(StackType::Foundation), "Ace of Spades should move to empty Spade foundation (idx 3)");
        assert_eq!(find_automatic_foundation_move(&two_hearts, &world), None, "Two of Hearts cannot move to any empty foundation");


        // --- シナリオ 2: Heart Foundation に Ace of Hearts がある ---
        log("Scenario 2: Ace of Hearts on Foundation 0");
        // entity も HecsEntity 
        let card_entity_ace_h: HecsEntity = world.spawn((ace_hearts.clone(), StackInfo { stack_type: StackType::Foundation, stack_index: 0, position_in_stack: 1 }));
        // assert 文では world (&HecsWorld) を渡す 
        assert_eq!(find_automatic_foundation_move(&two_hearts, &world), Some(StackType::Foundation), "Two of Hearts should move to Heart foundation (idx 0) with Ace");
        assert_eq!(find_automatic_foundation_move(&ace_spades, &world), Some(StackType::Foundation), "Ace of Spades should move to empty Spade foundation (idx 3)");
        world.despawn(card_entity_ace_h).unwrap(); // entity も HecsEntity 


        // --- シナリオ 3: Heart Foundation に Two of Hearts がある (Ace の上に) ---
        log("Scenario 3: Two of Hearts on Foundation 0");
        // entity も HecsEntity 
        let _card_entity_ace_h: HecsEntity = world.spawn((ace_hearts.clone(), StackInfo { stack_type: StackType::Foundation, stack_index: 0, position_in_stack: 1 }));
        let _card_entity_two_h: HecsEntity = world.spawn((two_hearts.clone(), StackInfo { stack_type: StackType::Foundation, stack_index: 0, position_in_stack: 2 }));
        // assert 文では world (&HecsWorld) を渡す 
        assert_eq!(find_automatic_foundation_move(&three_hearts, &world), Some(StackType::Foundation), "Three of Hearts should move to Heart foundation (idx 0) with Two");

        println!("Automatic Foundation Move テスト、成功！🎉");

    }
} 