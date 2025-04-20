//! ゲームの勝利条件判定ロジックを定義するよ。

/// ゲームのクリア条件（全てのカードが組札にあるか）を判定する。
pub fn check_win_condition(foundation_card_count: usize) -> bool {
    foundation_card_count == 52 // 標準的な52枚デッキの場合
} 