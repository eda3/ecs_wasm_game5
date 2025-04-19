// src/components/mod.rs

// この components モジュールに属するサブモジュールを宣言するよ！
// 今は card.rs だけだから、これだけ書けばOK！
pub mod card;
pub mod position; // 新しく position.rs を追加！📍
pub mod player; // 新しく player.rs を追加！👤
pub mod game_state; // 新しく game_state.rs を追加！��
pub mod stack;

// 各モジュール内の主要な型を use 宣言しておくと便利かも
pub use card::{Card, Rank, Suit};
pub use game_state::{GameState, GameStatus};
pub use player::Player;
pub use position::Position;
pub use stack::{StackInfo, StackType};

// 次に game_state.rs を作ったら、ここに `pub mod game_state;` を追加する感じ！
// 他のコンポーネントファイルも同様に追加していくよ。整理整頓！🧹✨ 