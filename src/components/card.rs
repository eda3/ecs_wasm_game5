// src/components/card.rs

// serde を使う宣言！カード情報をネットワークで送ったり保存したりする時に使うかも！
use serde::{Serialize, Deserialize};
// Component トレイトを使う宣言！このファイルで作る構造体がコンポーネントであることを示すため！
use crate::component::Component; // `crate::` はプロジェクトのルートから、って意味ね！
use wasm_bindgen::prelude::*;
use rand::{seq::SliceRandom, thread_rng};

/// カードのスート（マーク）を表す列挙型だよ！❤️♦️♣️♠️
///
/// #[derive(...)] のおまじないも忘れずに！
/// - Debug: デバッグ表示用 (`println!("{:?}", suit);`)
/// - Clone, Copy: 簡単にコピーできるように
/// - PartialEq, Eq: 等しいか比較できるように (`==`)
/// - Hash: HashMap のキーとかで使えるように
/// - Serialize, Deserialize: JSON などに変換できるように
#[wasm_bindgen]
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
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Copy は外したよ。カードの状態は変わる可能性があるからね。
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

// --- デッキ操作関連 (移動済み) ---

// 標準的な52枚のカードデッキ（ソリティア用）を生成する関数は src/logic/deck.rs に移動しました。
// カードデッキをシャッフルする関数は src/logic/deck.rs に移動しました。

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した Card, Suit, Rank を使う

    #[test]
    fn card_creation() {
        let card = Card {
            suit: Suit::Heart,
            rank: Rank::Ace,
            is_face_up: true,
        };
        assert_eq!(card.suit, Suit::Heart);
        assert_eq!(card.rank, Rank::Ace);
        assert_eq!(card.is_face_up, true);
        println!("Card 作成テスト: {:?} - 成功", card);
    }

    #[test]
    fn test_all_suits_size() {
        assert_eq!(ALL_SUITS.len(), 4, "スートの種類が4つじゃない！");
    }

    #[test]
    fn test_all_ranks_size() {
        assert_eq!(ALL_RANKS.len(), 13, "ランクの種類が13個じゃない！");
    }

    // デッキ生成・シャッフルに関するテストは src/logic/deck.rs に移動しました。
} 