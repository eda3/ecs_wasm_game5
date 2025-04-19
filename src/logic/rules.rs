//! ソリティアのゲームルール判定ロジックをまとめるモジュールだよ！🃏✅
//!
//! ここに関数を追加していくことで、カードがどこからどこへ移動できるか、
//! といったルールをチェックできるようにするんだ。

// 必要な型をインポートしておくよ！
use crate::components::card::{Card, Suit, Rank}; // ★修正: Color を削除！ (このファイル内で CardColor を定義してるから)
use crate::components::stack::{StackType, StackInfo}; // components の StackInfo, StackType を使う！
// use crate::world::World;                        // ゲーム世界の全体像 <-- これは使わない！
use crate::entity::Entity;                      // エンティティID (これは crate::entity のもの)
use crate::log;
use crate::world::World; // 自作 World を使うため
// use hecs::{World as HecsWorld, Entity as HecsEntity}; // <-- これを削除！

// TODO: 必要に応じて他のコンポーネントや型もインポートする！
// --- ここから自作ECSの型を定義していくことになる ---
// 例: type HecsWorld = crate::world::World; // 仮に自作Worldを使うようにする？
//     type HecsEntity = crate::entity::Entity;

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
// pub(crate) fn get_foundation_suit(foundation_index: u8) -> Option<Suit> {
//     match foundation_index {
//         0 => Some(Suit::Heart),
//         1 => Some(Suit::Diamond),
//         2 => Some(Suit::Club),
//         3 => Some(Suit::Spade),
//         _ => None, // 0-3 以外は無効なインデックス
//     }
// }

/// 指定された組札 (Foundation) の一番上にあるカードを取得するヘルパー関数。
/// World の状態を調べて、StackInfo を持つエンティティから見つける。
/// TODO: 自作Worldからデータを取得するロジックを実装する必要あり！
// pub(crate) fn get_foundation_top_card<'a>(world: &'a World, foundation_index: u8) -> Option<&'a Card> {
//     // 1. StackType::Foundation(foundation_index) の StackInfo を持つ Entity を探す。
//     // 2. その Entity に関連付けられた StackItem コンポーネントのうち、pos_in_stack が最大のものを探す。
//     // 3. 見つかった StackItem の Card コンポーネントへの参照を返す。

//     // 仮実装: とりあえず None を返す
//     None
// }

/// 特定のカードが、現在のワールドの状態において、自動的に移動できる組札（Foundation）があるかどうかを探す関数。
/// 見つかった場合は、移動先の StackType (Foundation のインデックス付き) を返す。
///
/// # 引数
/// - `card_to_move`: 移動させたいカードのコンポーネントへの参照 (`component::Card`)。
/// - `world`: 現在の World の状態への参照 (自作World)。
///
/// # 戻り値
/// - `Some(StackType)`: 移動可能な組札が見つかった場合、その組札の StackType (`component::StackType`)。
///                     注意: StackType::Foundation(index) の形で返すよ！
/// - `None`: 移動可能な組札が見つからなかった場合。
// pub fn find_automatic_foundation_move<'a>(
//     card_to_move: &Card,
//     world: &'a World
// ) -> Option<StackType> {
//     log(&format!("[Rules] Finding automatic foundation move for {:?}...", card_to_move));

//     for i in 0..4u8 { // 4つの Foundation をチェック
//         let foundation_suit = get_foundation_suit(i);

//         if foundation_suit.is_none() { continue; } // 無効なインデックスはスキップ
//         let foundation_suit = foundation_suit.unwrap();

//         // Foundation の一番上のカードを取得
//         let foundation_top_card: Option<&Card> = get_foundation_top_card(world, i);

//         // 移動可能かチェック
//         if can_move_to_foundation(card_to_move, foundation_top_card, foundation_suit) {
//             log(&format!("  Found valid foundation [{}] for {:?}. Top card: {:?}", i, card_to_move, foundation_top_card));
//             // 移動可能な Foundation が見つかったので、StackType::Foundation(i) を返す
//             return Some(StackType::Foundation(i));
//         }
//     }

//     log(&format!("  No suitable foundation found for {:?}.", card_to_move));
//     None // 適切な移動先が見つからなかった
// }

/// 指定されたスタック (`target_stack`) の一番上にあるカードのエンティティID (`Entity`) を取得するよ。
// ... (関数コメント略) ...
fn get_top_card_entity(world: &World, target_stack: StackType) -> Option<Entity> {
    // log(&format!("[Rules Helper] get_top_card_entity for {:?} called", target_stack)); // デバッグログ

    // StackInfo コンポーネントを持つ全てのエンティティを取得するイテレータを作成。
    let stack_entities = world.get_all_entities_with_component::<StackInfo>();

    // イテレータを処理していくよ！
    // ★★★ エラー修正: Vec<Entity> をイテレータにするために .into_iter() を追加！ ★★★
    stack_entities
        .into_iter() // <- これを追加！ Vec をイテレータに変換！
        // filter を使って、各エンティティの StackInfo をチェックする。
        .filter(|&entity| {
            // world から StackInfo コンポーネントを取得。
            world.get_component::<StackInfo>(entity)
                // map_or を使って、Option の中身を処理する。
                .map_or(false, |stack_info| stack_info.stack_type == target_stack)
        })
        // フィルターされたエンティティの中から、position_in_stack が最大のものを探す。
        .max_by_key(|&entity| {
            // world から StackInfo を取得。
            world.get_component::<StackInfo>(entity)
                // map_or を使って、Some(stack_info) なら position_in_stack を返す。
                .map_or(0, |stack_info| stack_info.position_in_stack)
        })
    // max_by_key は Option<Entity> を返すので、それをそのまま関数の戻り値とする。
}

// TODO: 他の移動パターン (Stock -> Waste, Waste -> Tableau/Foundation など) の
//       ルールチェック関数も必要に応じて追加していく！💪

// --- テストコード ---
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの要素を使う
    use crate::component::Rank; // Rank も使う
    use crate::world::World; // 自作Worldを使う (仮)
    use crate::entity::Entity; // 自作Entityを使う (仮)
    use crate::components::card::{Card, Suit}; // Card, Suit 追加
    use crate::components::stack::StackType; // StackType 追加

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

    // --- find_automatic_foundation_move のテストは src/logic/auto_move.rs に移動しました ---

    #[test]
    fn test_can_deal_from_stock() {
        let mut world = World::new(); // 自作World
        // TODO: テストデータ作成

        // --- シナリオ 1: 山札が空 ---
        // assert!(!can_deal_from_stock(&world)); // World を引数にとるように変更？
    }
}

// ▲▲▲ HecsWorld を使っている部分を修正する必要がある ▲▲▲