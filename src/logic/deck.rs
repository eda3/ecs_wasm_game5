// src/logic/deck.rs

use crate::components::card::{Card, Rank, Suit, ALL_RANKS, ALL_SUITS};
use rand::{seq::SliceRandom, thread_rng};

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

/// カードデッキをシャッフルする関数だよ。
///
/// # 引数
/// * `deck` - シャッフルしたいカードデッキ (`Vec<Card>`) への可変参照。
pub fn shuffle_deck(deck: &mut Vec<Card>) {
    let mut rng = thread_rng(); // 乱数生成器を取得
    deck.shuffle(&mut rng); // デッキをシャッフル！
}

// --- テスト (移動した関数のテストもこちらに移動) ---
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した関数と、インポートした Card, Suit, Rank を使う

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

    #[test]
    fn test_create_deck_size() {
        let deck = create_standard_deck();
        assert_eq!(deck.len(), 52, "デッキのカード数が52枚じゃない！");
    }

    #[test]
    fn test_create_deck_uniqueness() {
        let deck = create_standard_deck();
        let mut seen_cards = std::collections::HashSet::new();
        let mut duplicates = Vec::new();

        for card in deck {
            if !seen_cards.insert(card.clone()) {
                duplicates.push(card);
            }
        }

        assert!(duplicates.is_empty(), "デッキに重複カードあり！: {:?}", duplicates);
    }

    #[test]
    fn test_shuffle_deck_changes_order() {
        let initial_deck = create_standard_deck();
        let mut shuffled_deck = initial_deck.clone(); // コピーしてシャッフルする
        shuffle_deck(&mut shuffled_deck);

        // シャッフルしたら元の順番とは (ほぼ確実に) 変わるはず
        // ただし、ごく稀に同じ順番になる可能性もあるので、完全なテストではない
        assert_ne!(initial_deck, shuffled_deck, "シャッフルしても順番が変わってない (稀に起こりうる)");
        // サイズは変わらないはず
        assert_eq!(initial_deck.len(), shuffled_deck.len(), "シャッフルでカード数が変わった！");
    }
} 