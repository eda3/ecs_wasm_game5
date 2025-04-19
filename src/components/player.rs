// src/components/player.rs

// serde を使う宣言！プレイヤー情報をネットワークで送受信するかも！
use serde::{Serialize, Deserialize};
// Component トレイトを使う宣言！Player がコンポーネントであることを示す！
use crate::component::Component;

/// プレイヤーを表すコンポーネントだよ！👤
///
/// マルチプレイゲームなので、どのエンティティがプレイヤーなのか、
/// そして今誰のターンなのか、といった情報を管理する必要があるね！
///
/// - `id`: プレイヤーを識別するための一意なID。ここでは単純に数値 (`u32`) にしてみるね！
///         ネットワーク接続とかと紐づけることも考えられるけど、まずはシンプルに！
/// - `is_current_turn`: このプレイヤーが現在操作可能かどうかを示すフラグ。
///
/// #[derive(...)] のおまじない！
/// - Debug: デバッグ表示用
/// - Clone: コピー可能に
/// - PartialEq: 等しいか比較できるように
/// - Serialize, Deserialize: JSON などに変換できるように
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,          // プレイヤーID (0, 1, ...みたいに割り振る想定)
    pub is_current_turn: bool, // 現在このプレイヤーのターンか？
    // TODO: 必要ならプレイヤー名とか、持ち点とか、他の情報も追加できるね！
}

// Player 構造体が Component であることを示すマーカー！✅
impl Component for Player {}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した Player を使う
    use crate::component::Component; // Component トレイトもテストで使う

    #[test]
    fn create_player_component() {
        let player1 = Player {
            id: 0,
            is_current_turn: true, // 最初のプレイヤーは操作可能
        };
        let player2 = Player {
            id: 1,
            is_current_turn: false, // 2番目のプレイヤーは待機
        };

        // 値がちゃんと設定されてるか確認
        assert_eq!(player1.id, 0);
        assert_eq!(player1.is_current_turn, true);
        assert_eq!(player2.id, 1);
        assert_eq!(player2.is_current_turn, false);

        // デバッグ表示も確認
        println!("作成したプレイヤー1: {:?}", player1);
        println!("作成したプレイヤー2: {:?}", player2);

        // Component トレイトが実装されているかチェック
        fn needs_component<T: Component>(_: T) {}
        needs_component(player1.clone());
        needs_component(player2.clone());

        println!("Player コンポーネント作成テスト、成功！🎉");
    }
} 