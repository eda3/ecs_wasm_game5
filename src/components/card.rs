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

// --- テスト ---
// 簡単なテストを書いておこう！
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した Suit, Rank, Card を使う

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
} 