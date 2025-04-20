// src/components/game_state.rs

// serde を使う宣言！ゲーム状態を保存したり通信したりするかも！
use serde::{Serialize, Deserialize};
// Component トレイトを使うからインポートするよ
use crate::ecs::component::Component;

/// ゲーム全体の現在の状態を表す列挙型だよ！
///
/// ゲームがまだプレイ中なのか、それとも誰かが勝って終わったのか、
/// みたいな状況を示すのに使うよ！🏆🏁
///
/// このコンポーネントは、普通はゲーム全体で一つだけ存在する特別なエンティティ
/// （例えば、エンティティIDが0とか、特別な名前をつけたエンティティとか）に
/// アタッチされることが多いよ。（リソースとかシングルトンって呼ばれたりもする）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameStatus {
    /// ゲームが進行中の状態
    Playing,
    /// ゲームが終了した状態
    GameOver {
        /// 勝者のプレイヤーID (もし引き分けとかなら None になるかも？)
        winner_id: Option<u32>, // PlayerコンポーネントのIDに対応させる想定
    },
    /// 勝利！🏆
    Won,
    // TODO: 必要なら、ゲーム開始前の待機状態 (WaitingForPlayers) とか、
    //       ポーズ中 (Paused) とか、他の状態も追加できるね！
}

/// ゲーム状態を保持するコンポーネント。
///
/// 中身はシンプルに GameStatus enum を持つだけ！
/// これを World に登録して、一つのエンティティに持たせることで、
/// どこからでも現在のゲーム状態を参照・更新できるようにするんだ。便利！💡
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub status: GameStatus,
}

// GameState 構造体が Component であることを示すマーカー！✅
impl Component for GameState {}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した GameStatus, GameState を使う
    use crate::component::Component; // Component トレイトもテストで使う

    #[test]
    fn create_game_state_component() {
        // 最初はプレイ中の状態
        let initial_state = GameState {
            status: GameStatus::Playing,
        };

        assert_eq!(initial_state.status, GameStatus::Playing);
        println!("初期ゲーム状態: {:?}", initial_state);

        // ゲームオーバーの状態も作ってみる (プレイヤー1が勝利！)
        let game_over_state = GameState {
            status: GameStatus::GameOver { winner_id: Some(1) },
        };

        assert_eq!(game_over_state.status, GameStatus::GameOver { winner_id: Some(1) });
        println!("ゲームオーバー状態: {:?}", game_over_state);

        // Component トレイトが実装されているかチェック
        fn needs_component<T: Component>(_: T) {}
        needs_component(initial_state.clone());
        needs_component(game_over_state.clone());

        println!("GameState コンポーネント作成テスト、成功！🎉");
    }

    #[test]
    fn game_status_comparison() {
        let playing = GameStatus::Playing;
        let over_p1_wins = GameStatus::GameOver { winner_id: Some(1) };
        let over_p2_wins = GameStatus::GameOver { winner_id: Some(2) };
        let over_draw = GameStatus::GameOver { winner_id: None }; // 引き分けの場合

        assert_eq!(playing, GameStatus::Playing);
        assert_ne!(playing, over_p1_wins);
        assert_eq!(over_p1_wins, GameStatus::GameOver { winner_id: Some(1) });
        assert_ne!(over_p1_wins, over_p2_wins);
        assert_ne!(over_p1_wins, over_draw);

        println!("GameStatus の比較テスト、成功！🎉");
    }
} 