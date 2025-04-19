//! ソリティアのゲームルール判定ロジックをまとめるモジュールだよ！🃏✅
//!
//! ここに関数を追加していくことで、カードがどこからどこへ移動できるか、
//! といったルールをチェックできるようにするんだ。

// 必要な型をインポートしておくよ！
use crate::components::card::{Card, Suit, Rank}; // カード情報
use crate::components::stack::{StackInfo, StackType};        // スタックの情報と種類
use crate::world::World;                        // ゲーム世界の全体像
use crate::entity::Entity;                      // エンティティID
use crate::component::Component;                // Component トレイト (テスト用)

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
/// * `card_to_move`: 移動させようとしているカード。
/// * `foundation_top_card`: 移動先の組札の一番上にあるカード (なければ None)。
/// * `foundation_suit`: 移動先の組札のスート (Foundation(0) なら Heart みたいに事前に解決しておく)。
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_to_foundation(
    card_to_move: &Card,
    foundation_top_card: Option<&Card>,
    foundation_suit: Suit,
) -> bool {
    // 1. スートが一致しているか？
    if card_to_move.suit != foundation_suit {
        return false; // スートが違うなら置けない！🙅‍♀️
    }

    // 2. ランクが正しいか？
    match foundation_top_card {
        // 組札が空の場合 (一番上のカードがない場合)
        None => {
            // エース (A) なら置ける！👑
            card_to_move.rank == Rank::Ace
        }
        // 組札に既にカードがある場合
        Some(top_card) => {
            // 移動するカードのランクが、一番上のカードのランクの「次」なら置ける！
            // (例: 上が A なら 2、上が 10 なら J)
            // Rank enum は Ord を実装してるので、大小比較ができる！
            // `as usize` で数値に変換して比較する方が確実かも？🤔
            (card_to_move.rank as usize) == (top_card.rank as usize) + 1
        }
    }
}

/// 指定されたカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
///
/// # 引数
/// * `card_to_move`: 移動させようとしているカード。
/// * `tableau_top_card`: 移動先の場札の一番上にあるカード (空の列なら None)。
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_to_tableau(
    card_to_move: &Card,
    tableau_top_card: Option<&Card>,
) -> bool {
    match tableau_top_card {
        // 場札の列が空の場合
        None => {
            // キング (K) なら置ける！🤴
            card_to_move.rank == Rank::King
        }
        // 場札の列に既にカードがある場合
        Some(top_card) => {
            // 1. 色が違うか？ (赤と黒)
            let move_color = CardColor::from_suit(card_to_move.suit);
            let target_color = CardColor::from_suit(top_card.suit);
            if move_color == target_color {
                return false; // 同じ色なら重ねられない！🟥🟥 or ⬛️⬛️ はダメ！
            }

            // 2. ランクが連続しているか？ (移動するカードが1つ小さい)
            // (例: 上が Q なら J、上が 7 なら 6)
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
    waste_top_card: &Card,
    tableau_top_card: Option<&Card>,
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
    waste_top_card: &Card,
    foundation_top_card: Option<&Card>,
    foundation_suit: Suit,
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
fn get_foundation_suit(foundation_index: u8) -> Option<Suit> {
    match foundation_index {
        0 => Some(Suit::Heart),
        1 => Some(Suit::Diamond),
        2 => Some(Suit::Club),
        3 => Some(Suit::Spade),
        _ => None, // 0-3 以外は無効なインデックス
    }
}

/// 指定された組札 (Foundation) の一番上にあるカードを取得する。
/// World の状態を調べて、StackInfo を持つエンティティから見つけるよ！
fn get_foundation_top_card<'a>(world: &'a World, foundation_index: u8) -> Option<&'a Card> {
    let target_stack_type = StackType::Foundation(foundation_index);
    let mut top_entity: Option<Entity> = None;
    let mut max_pos_in_stack: i16 = -1; // u8 より大きい型で比較

    // StackInfo コンポーネントを持つエンティティを全て調べる
    if let Some(stack_infos) = world.storage::<StackInfo>() {
        for (entity, stack_info) in stack_infos.iter() {
            // 目的の Foundation スタックに属しているか？
            if stack_info.stack_type == target_stack_type {
                // 現在の最大位置よりも大きいか？ (より上にあるか？)
                if (stack_info.position_in_stack as i16) > max_pos_in_stack {
                    max_pos_in_stack = stack_info.position_in_stack as i16;
                    top_entity = Some(*entity); // 一番上のエンティティ候補を更新
                }
            }
        }
    }

    // 見つかった一番上のエンティティから Card コンポーネントを取得する
    top_entity.and_then(|entity| world.get_component::<Card>(entity))
}

/// 特定のカードが、現在のワールドの状態において、自動的に移動できる組札（Foundation）があるかどうかを探す関数。
/// 見つかった場合は、移動先の StackType (Foundation のインデックス付き) を返す。
///
/// # 引数
/// - `card_to_move`: 移動させたいカードのコンポーネントへの参照。
/// - `world`: 現在の World の状態への参照。
///
/// # 戻り値
/// - `Some(StackType)`: 移動可能な組札が見つかった場合、その組札の StackType (Foundation(index))。
/// - `None`: 移動可能な組札が見つからなかった場合。
pub fn find_automatic_foundation_move<'a>(
    card_to_move: &crate::component::Card,
    world: &'a World
) -> Option<StackType> {
    log(&format!("[Rules] Finding automatic foundation move for {:?}...", card_to_move));

    // 移動元カードが A (エース) の場合
    if card_to_move.rank == Rank::Ace {
        // 空の Foundation を探す
        for i in 0..4u8 { // 4つの Foundation をチェック
            let target_stack = StackType::Foundation(i);
            if is_foundation_empty(target_stack, world) {
                log(&format!("  Found empty foundation [{}] for Ace.", i));
                return Some(target_stack);
            }
        }
        log("  No empty foundation found for Ace.");
        return None; // 空きがなければ移動できない
    }

    // 移動元カードが A 以外の場合
    // 同じスートでランクが1つ下のカードが一番上にある Foundation を探す
    for i in 0..4u8 { // 4つの Foundation をチェック
        let target_stack = StackType::Foundation(i);
        // Foundation の一番上のカードを取得
        if let Some(top_card_entity) = get_top_card_entity_in_stack(target_stack, world) {
            if let Some(top_card) = world.get_component::<Card>(top_card_entity) {
                // ↓↓↓ ここの can_move_to_foundation を使う！ (引数は component::Card になってるはず)
                if can_move_to_foundation(card_to_move, Some(top_card)) {
                    log(&format!("  Found valid foundation [{}] for {:?}. Top card: {:?}", i, card_to_move, top_card));
                    return Some(target_stack);
                }
            }
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
    use crate::components::card::Rank; // Rank も使う

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
        // テスト用のカードを作成
        let ace_hearts = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_hearts = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let three_hearts = Card { suit: Suit::Heart, rank: Rank::Three, is_face_up: true };
        let ace_spades = Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true };

        // --- Foundation が空の場合 ---
        // Ace は置ける
        assert!(can_move_to_foundation(&ace_hearts, None, Suit::Heart), "空のHeart Foundation に Ace of Hearts は置けるはず");
        // Ace 以外は置けない
        assert!(!can_move_to_foundation(&two_hearts, None, Suit::Heart), "空のHeart Foundation に 2 of Hearts は置けないはず");
        // スートが違う Ace は置けない
        assert!(!can_move_to_foundation(&ace_spades, None, Suit::Heart), "空のHeart Foundation に Ace of Spades は置けないはず");

        // --- Foundation に Ace がある場合 ---
        // 同じスートの 2 は置ける
        assert!(can_move_to_foundation(&two_hearts, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 2 of Hearts は置けるはず");
        // 同じスートの 3 は置けない
        assert!(!can_move_to_foundation(&three_hearts, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 3 of Hearts は置けないはず");
        // 違うスートの 2 は置けない
        let two_spades = Card { suit: Suit::Spade, rank: Rank::Two, is_face_up: true };
        assert!(!can_move_to_foundation(&two_spades, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 2 of Spades は置けないはず");

        // --- Foundation に 2 がある場合 ---
        assert!(can_move_to_foundation(&three_hearts, Some(&two_hearts), Suit::Heart), "Heart Foundation (Two) に 3 of Hearts は置けるはず");

        println!("Foundation 移動ルールテスト、成功！🎉");
    }

     #[test]
    fn test_can_move_to_tableau_rules() {
        // テスト用カード
        let king_spades = Card { suit: Suit::Spade, rank: Rank::King, is_face_up: true };
        let queen_hearts = Card { suit: Suit::Heart, rank: Rank::Queen, is_face_up: true };
        let queen_clubs = Card { suit: Suit::Club, rank: Rank::Queen, is_face_up: true };
        let jack_diamonds = Card { suit: Suit::Diamond, rank: Rank::Jack, is_face_up: true };
        let jack_spades = Card { suit: Suit::Spade, rank: Rank::Jack, is_face_up: true };
        let ten_hearts = Card { suit: Suit::Heart, rank: Rank::Ten, is_face_up: true };

        // --- Tableau が空の場合 ---
        // King は置ける
        assert!(can_move_to_tableau(&king_spades, None), "空の Tableau に King of Spades は置けるはず");
        // King 以外は置けない
        assert!(!can_move_to_tableau(&queen_hearts, None), "空の Tableau に Queen of Hearts は置けないはず");

        // --- Tableau に Queen of Hearts (赤) がある場合 ---
        // 黒の Jack は置ける
        assert!(can_move_to_tableau(&jack_spades, Some(&queen_hearts)), "Tableau (Q❤️) に J♠️ は置けるはず");
        // 赤の Jack は置けない (色違い違反)
        assert!(!can_move_to_tableau(&jack_diamonds, Some(&queen_hearts)), "Tableau (Q❤️) に J♦️ は置けないはず (同色)");
        // 黒の 10 は置けない (ランク連続違反)
        let ten_clubs = Card { suit: Suit::Club, rank: Rank::Ten, is_face_up: true };
        assert!(!can_move_to_tableau(&ten_clubs, Some(&queen_hearts)), "Tableau (Q❤️) に 10♣️ は置けないはず (ランク違い)");

        // --- Tableau に Jack of Spades (黒) がある場合 ---
        // 赤の 10 は置ける
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
        // テスト用カード (既存のテストから拝借 or 新規作成)
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

    // --- find_automatic_foundation_move のテスト ---
    #[test]
    fn test_find_automatic_foundation_move() {
        // テスト用の World を準備
        let mut world = World::new();
        world.register_component::<Card>();
        world.register_component::<StackInfo>();

        // --- Foundation の状態を設定 ---
        // Foundation 0 (Heart): 空
        // Foundation 1 (Diamond): Ace of Diamonds
        let f1_ace = world.create_entity();
        world.add_component(f1_ace, Card { suit: Suit::Diamond, rank: Rank::Ace, is_face_up: true });
        world.add_component(f1_ace, StackInfo::new(StackType::Foundation(1), 0));
        // Foundation 2 (Club): Ace, 2 of Clubs
        let f2_ace = world.create_entity();
        world.add_component(f2_ace, Card { suit: Suit::Club, rank: Rank::Ace, is_face_up: true });
        world.add_component(f2_ace, StackInfo::new(StackType::Foundation(2), 0));
        let f2_two = world.create_entity();
        world.add_component(f2_two, Card { suit: Suit::Club, rank: Rank::Two, is_face_up: true });
        world.add_component(f2_two, StackInfo::new(StackType::Foundation(2), 1)); // 2番目が上
        // Foundation 3 (Spade): Ace of Spades
        let f3_ace = world.create_entity();
        world.add_component(f3_ace, Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true });
        world.add_component(f3_ace, StackInfo::new(StackType::Foundation(3), 0));


        // --- テストケース ---
        // 1. Ace of Hearts (空の Foundation 0 に移動できるはず)
        let move_card1 = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        assert_eq!(
            find_automatic_foundation_move(&move_card1, &world),
            Some(StackType::Foundation(0)), // Heart の Foundation は 0 番目
            "Ace of Hearts は空の Foundation 0 に移動できるはず"
        );

        // 2. Two of Diamonds (Foundation 1 の Ace の上に移動できるはず)
        let move_card2 = Card { suit: Suit::Diamond, rank: Rank::Two, is_face_up: true };
        assert_eq!(
            find_automatic_foundation_move(&move_card2, &world),
            Some(StackType::Foundation(1)), // Diamond の Foundation は 1 番目
            "Two of Diamonds は Foundation 1 (Ace) の上に移動できるはず"
        );

        // 3. Three of Clubs (Foundation 2 の Two の上に移動できるはず)
        let move_card3 = Card { suit: Suit::Club, rank: Rank::Three, is_face_up: true };
        assert_eq!(
            find_automatic_foundation_move(&move_card3, &world),
            Some(StackType::Foundation(2)), // Club の Foundation は 2 番目
            "Three of Clubs は Foundation 2 (Two) の上に移動できるはず"
        );

         // 4. Two of Spades (Foundation 3 の Ace の上に移動できるはず)
         let move_card4 = Card { suit: Suit::Spade, rank: Rank::Two, is_face_up: true };
         assert_eq!(
             find_automatic_foundation_move(&move_card4, &world),
             Some(StackType::Foundation(3)), // Spade の Foundation は 3 番目
             "Two of Spades は Foundation 3 (Ace) の上に移動できるはず"
         );

        // 5. Ace of Clubs (既に Foundation 2 にあるので、他の空き Foundation には行けない)
        let move_card5 = Card { suit: Suit::Club, rank: Rank::Ace, is_face_up: true };
        assert_eq!(
            find_automatic_foundation_move(&move_card5, &world),
            None, // 移動先はないはず (Foundation 0 (Heart) には置けない)
            "Ace of Clubs は他の Foundation には移動できないはず"
        );

        // 6. Four of Clubs (Foundation 2 の Two の上には置けない)
        let move_card6 = Card { suit: Suit::Club, rank: Rank::Four, is_face_up: true };
        assert_eq!(
            find_automatic_foundation_move(&move_card6, &world),
            None,
            "Four of Clubs は Foundation 2 (Two) の上には置けないはず"
        );

        // 7. King of Hearts (どの Foundation にも直接は置けない)
        let move_card7 = Card { suit: Suit::Heart, rank: Rank::King, is_face_up: true };
         assert_eq!(
            find_automatic_foundation_move(&move_card7, &world),
            None,
            "King of Hearts はどの Foundation にも自動移動できないはず"
        );

        println!("自動移動先探索 (Foundation) テスト、成功！🎉");
    }
} 