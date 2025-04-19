// src/components/stack.rs

// serde を使うためにインポート！Serialize と Deserialize トレイトを使うよ。
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

/// カードが存在する場所の種類を示す Enum だよ。
/// これを使って、カードが山札にあるのか、場札の何列目にあるのか、などを区別するよ。
/// Clone, Copy: 値を簡単に複製できるようにする。
/// Debug: println! などで中身をデバッグ表示できるようにする。
/// PartialEq, Eq: == 演算子で比較できるようにする。
/// Serialize, Deserialize: この Enum を JSON 形式に変換したり、JSON から戻したりできるようにする！これが重要！✨
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackType {
    /// 場札 (Tableau) だよ。7つの列があるので、列番号 (0-6) を持つ。
    Tableau(u8),
    /// 組札 (Foundation) だよ。スートごとに4つある。
    /// Suit 型を直接使うと依存関係が複雑になるかも？
    /// とりあえず番号 (0-3) で管理してみようかな？
    /// 0: Heart, 1: Diamond, 2: Club, 3: Spade みたいな感じで！
    Foundation(u8),
    /// 山札 (Stock) だよ。プレイヤーがカードを引く元の場所。
    Stock,
    /// 山札からめくったカードを置く場所 (Waste) だよ。
    /// クロンダイクでは通常1つだけど、ゲームによっては複数あるかも？
    Waste,
    // 将来的には： Hand(PlayerId), DiscardPile など他のゲーム用に拡張できる
    Hand,
}

/// カードのスタックに関する情報を持つコンポーネントだよ。
/// カードエンティティにこれを持たせることで、そのカードがどこにあるか、
/// そのスタックの中で何番目か、などを管理するよ。
/// Component トレイトを実装して、ECS で使えるようにする。
use crate::component::Component;

#[derive(Debug, Clone)] // デバッグ表示とクローンができるように
pub struct StackInfo {
    /// カードが属しているスタックの種類。
    pub stack_type: StackType,
    /// そのスタックの中で、カードが下から何番目に積まれているか (0 が一番下)。
    pub position_in_stack: u8,
}

impl StackInfo {
    /// 新しい StackInfo を作成するヘルパー関数。
    pub fn new(stack_type: StackType, position_in_stack: u8) -> Self {
        Self { stack_type, position_in_stack }
    }
}

// StackInfo をコンポーネントとして使えるように、Component トレイトを実装！
// 中身は空でOK！マーカーとして機能するよ。
impl Component for StackInfo {} // これで World に登録できるようになる

// ↓↓↓ 逆方向の StackType の From トレイト実装を追加！ ↓↓↓
impl From<crate::component::StackType> for StackType {
    fn from(component_stack_type: crate::component::StackType) -> Self {
        match component_stack_type {
            crate::component::StackType::Tableau => StackType::Tableau(0), // デフォルトインデックス 0
            crate::component::StackType::Foundation => StackType::Foundation(0), // デフォルトインデックス 0
            crate::component::StackType::Stock => StackType::Stock,
            crate::component::StackType::Waste => StackType::Waste,
            crate::component::StackType::Hand => StackType::Hand,
        }
    }
}
// ↑↑↑ 逆方向の StackType の From トレイト実装を追加！ ↑↑↑

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_info_creation() {
        let info1 = StackInfo::new(StackType::Tableau(2), 5);
        assert_eq!(info1.stack_type, StackType::Tableau(2));
        assert_eq!(info1.position_in_stack, 5);

        let info2 = StackInfo::new(StackType::Foundation(0), 0); // Ace
        assert_eq!(info2.stack_type, StackType::Foundation(0));
        assert_eq!(info2.position_in_stack, 0);

        println!("StackInfo 作成テスト、成功！👍");
    }
} 