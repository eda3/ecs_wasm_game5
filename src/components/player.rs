// src/components/player.rs

// serde を使う宣言！プレイヤー情報をネットワークで送受信するかも！
use serde::{Serialize, Deserialize};
// Component トレイトを使う宣言！Player がコンポーネントであることを示す！
use crate::ecs::component::Component;

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
    // ★修正: id フィールドは PlayerId 型 (u32) を使う想定だったかも？
    //         一旦 state_handler の実装に合わせて usize でEntity IDと紐づける？
    //         あるいは state_handler 側で id を使う？
    //         PlayerId は protocol.rs で定義されてる u32。
    // pub id: u32, // ← コメントアウト (サーバーからの PlayerData.id を直接コンポーネントに持たせるかは要検討)
    pub name: String, // ★追加: プレイヤー名！★
    pub is_current_turn: bool, // 現在このプレイヤーのターンか？
    // TODO: 必要なら持ち点とか他の情報も追加
}

// Player 構造体が Component であることを示すマーカー！✅
impl Component for Player {}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // Player を使う
    use crate::ecs::component::Component; // Component トレイト

    #[test]
    fn create_player_component() {
        let player1 = Player {
            // id: 0, // id フィールド削除に伴いコメントアウト
            name: "Player 1".to_string(), // ★追加
            is_current_turn: true,
        };
        let player2 = Player {
            // id: 1,
            name: "Player 2".to_string(), // ★追加
            is_current_turn: false,
        };

        // 値の確認
        // assert_eq!(player1.id, 0);
        assert_eq!(player1.name, "Player 1"); // ★追加
        assert_eq!(player1.is_current_turn, true);
        // assert_eq!(player2.id, 1);
        assert_eq!(player2.name, "Player 2"); // ★追加
        assert_eq!(player2.is_current_turn, false);

        println!("作成したプレイヤー1: {:?}", player1);
        println!("作成したプレイヤー2: {:?}", player2);

        // Component トレイト実装チェック
        fn needs_component<T: Component>(_: T) {}
        needs_component(player1.clone());
        needs_component(player2.clone());

        println!("Player コンポーネント作成テスト、成功！🎉");
    }
} 