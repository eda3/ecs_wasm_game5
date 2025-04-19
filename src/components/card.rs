// src/components/card.rs

// serde を使う宣言！カード情報をネットワークで送ったり保存したりする時に使うかも！
use serde::{Serialize, Deserialize};
// Component トレイトを使う宣言！このファイルで作る構造体がコンポーネントであることを示すため！
use crate::component::Component; // `crate::` はプロジェクトのルートから、って意味ね！

/// カードのスート（マーク）を表す列挙型だよ！❤️♦️♣️♠️
///
/// #[derive(...)] のおまじないも忘れずに！
/// - Debug: デバッグ表示用 (`println!("{:?}", suit);`)
/// - Clone, Copy: 簡単にコピーできるように
/// - PartialEq, Eq: 等しいか比較できるように (`==`)
/// - Hash: HashMap のキーとかで使えるように
/// - Serialize, Deserialize: JSON などに変換できるように
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Heart,   // ❤️
    Diamond, // ♦️
    Club,    // ♣️
    Spade,   // ♠️
}

/// カードのランク（数字）を表す列挙型だよ！ A, 2, 3, ..., K
///
/// スートと同じように #[derive(...)] を付けておくよ！
/// PartialOrd, Ord も追加して、ランクの大小比較 (`<`, `>`) もできるようにしておこう！ソリティアで使いそう！👍
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rank {
    Ace = 1, // A は 1 として扱うよ (ソリティアのルールによるかもだけど、一旦こうしておく！)
    Two,     // 2
    Three,   // 3
    Four,    // 4
    Five,    // 5
    Six,     // 6
    Seven,   // 7
    Eight,   // 8
    Nine,    // 9
    Ten,     // 10
    Jack,    // J (11 扱い)
    Queen,   // Q (12 扱い)
    King,    // K (13 扱い)
}

/// カードそのものを表すコンポーネントだよ！🃏
///
/// これがエンティティに付けられる「データ」になるんだ。
/// 「このエンティティは、ハート♥️のAだよ！」みたいにね！
///
/// - `suit`: カードのスート
/// - `rank`: カードのランク
/// - `is_face_up`: カードが表向きか裏向きかを示すフラグ (trueなら表向き)
///
/// Component トレイトを実装するのを忘れないでね！ これがないと World に登録できない！🙅‍♀️
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // Copy は外したよ。カードの状態は変わる可能性があるからね。
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
    pub is_face_up: bool, // カードが表向きかどうか
}

// Card 構造体が Component であることを示すよ！
impl Component for Card {}

// Suit の全種類を配列として定義しておくと、後でループ処理とかで便利だよ！
pub const ALL_SUITS: [Suit; 4] = [Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade];

// Rank の全種類も配列で定義！AからKまで！
pub const ALL_RANKS: [Rank; 13] = [
    Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
    Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King,
];

// --- デッキ生成関数 ---

/// 標準的な52枚のカードデッキ（ソリティア用）を生成する関数だよ！🃏
///
/// 返り値は `Vec<Card>` で、カードはスートとランクの組み合わせで全種類作られるよ。
/// 生成された時点では、すべてのカードは裏向き (`is_face_up: false`) になってる！
pub fn create_standard_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52); // 52枚入る容量を確保しておくと効率的！

    // 定義しておいた ALL_SUITS と ALL_RANKS を使ってループ！
    for &suit in ALL_SUITS.iter() { // `&suit` で Suit の値を取得
        for &rank in ALL_RANKS.iter() { // `&rank` で Rank の値を取得
            deck.push(Card {
                suit,
                rank,
                is_face_up: false, // 最初は裏向き
            });
        }
    }
    deck // 完成したデッキを返す！
}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した Suit, Rank, Card, ALL_SUITS, ALL_RANKS, create_standard_deck を使う

    #[test]
    fn create_card_component() {
        let card = Card {
            suit: Suit::Spade,
            rank: Rank::Ace,
            is_face_up: false, // 最初は裏向き
        };

        // 値がちゃんと設定されてるか確認
        assert_eq!(card.suit, Suit::Spade);
        assert_eq!(card.rank, Rank::Ace);
        assert_eq!(card.is_face_up, false);

        // デバッグ表示も確認（これは実行時にコンソールに出るよ）
        println!("作成したカード: {:?}", card);

        // Component トレイトが実装されているかのチェック (コンパイルが通ればOKだけど念のため)
        fn needs_component<T: Component>(_: T) {}
        needs_component(card.clone()); // cloneして渡す

        println!("Card コンポーネント作成テスト、成功！🎉");
    }

    #[test]
    fn rank_comparison() {
        // ランクの大小比較がちゃんとできるか確認
        assert!(Rank::Ace < Rank::Two);
        assert!(Rank::Ten < Rank::Jack);
        assert!(Rank::Queen < Rank::King);
        assert!(Rank::King > Rank::Ace);
        assert_eq!(Rank::Seven, Rank::Seven);

        println!("Rank の比較テスト、成功！🎉");
    }

    #[test]
    fn deck_creation() {
        let deck = create_standard_deck();

        // 1. カードが52枚あるかチェック！
        assert_eq!(deck.len(), 52);
        println!("生成されたデッキの枚数: {}", deck.len());

        // 2. 重複がないかチェック！ (ちょっと大変だけど大事！)
        use std::collections::HashSet;
        let mut unique_cards = HashSet::with_capacity(52);
        let mut duplicates_found = false;
        for card in &deck {
            // HashSet の insert メソッドは、要素が既に追加されていたら false を返すよ！
            if !unique_cards.insert((card.suit, card.rank)) {
                duplicates_found = true;
                println!("重複発見！ {:?}", card);
                break; // 1枚見つかれば十分
            }
        }
        assert!(!duplicates_found, "デッキに重複したカードが見つかりました！");

        // 3. すべてのカードが裏向きかチェック！
        let all_face_down = deck.iter().all(|card| !card.is_face_up);
        assert!(all_face_down, "デッキに表向きのカードが含まれています！");

        println!("create_standard_deck 関数のテスト、成功！🎉 デッキは正しく生成されました！");
    }
} 