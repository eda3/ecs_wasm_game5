// src/components/stack.rs

/// カードが存在する場所の種類を示す Enum だよ。
/// これを使って、カードが山札にあるのか、場札の何列目にあるのか、などを区別するよ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackType {
    Stock,       // 山札 (まだ配られていないカード)
    Waste,       // 山札からめくられたカード置き場 (クロンダイク固有)
    Tableau(u8), // 場札 (列番号 0-6)
    Foundation(u8), // 組札 (置き場番号 0-3, スートとは直接紐付けない方が柔軟かも？)
    // 将来的には： Hand(PlayerId), DiscardPile など他のゲーム用に拡張できる
}

/// カードのスタックに関する情報を持つコンポーネントだよ。
/// カードエンティティにこれを持たせることで、そのカードがどこにあるか、
/// そのスタックの中で何番目か、などを管理するよ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackInfo {
    /// カードが現在属しているスタックの種類。
    pub stack_type: StackType,
    /// そのスタック内での位置 (一番下が 0)。
    /// 例えば、場札の一番上のカードは position_in_stack が大きい値になる。
    pub position_in_stack: u8,
}

impl StackInfo {
    /// 新しい StackInfo を作成するヘルパー関数。
    pub fn new(stack_type: StackType, position_in_stack: u8) -> Self {
        Self { stack_type, position_in_stack }
    }
}

// StackInfo を Component トレイトに適合させる (no-op の実装でOK)
use crate::component::Component;
impl Component for StackInfo {} // これで World に登録できるようになる

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