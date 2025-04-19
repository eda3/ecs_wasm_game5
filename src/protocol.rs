// src/protocol.rs

// このファイルは、クライアント(WASM)とサーバー(WebSocket)の間で
// やり取りするメッセージの形式（プロトコル）を定義するよ！💌
// データ構造 (struct や enum) を定義して、それをJSON形式に変換したり、
// JSON形式から元に戻したりするために、`serde`クレートを使うよ。
// `Serialize` は Rust のデータ構造 -> JSON 文字列 にするやつ、
// `Deserialize` は JSON 文字列 -> Rust のデータ構造 にするやつだよ。
use serde::{Serialize, Deserialize};

// ゲーム内の型もメッセージで使うからインポートしておくね！
// (TODO: もしこれらの型が Serialize/Deserialize を実装してなかったら、後で追加する必要があるよ！)
use crate::entity::Entity; // エンティティID (どのカードかを示すためとか)
use crate::components::card::{Suit, Rank}; // カードのスートやランク
// ★修正: StackType を pub use する！★
pub use crate::components::stack::StackType; // スタックの種類 (場札、組札、山札など)
// ↓↓↓ Position もメッセージで使う可能性があるのでインポートしておく
// (ただし、Position 自体に Serialize/Deserialize が必要になるので注意！)
// use crate::components::position::Position;

// --- クライアントからサーバーへ送るメッセージ (Client-to-Server: C2S) ---

/// クライアントがサーバーに送るメッセージの種類を表すenumだよ。
/// これをJSONにしてサーバーに送る！
#[derive(Serialize, Deserialize, Debug, Clone)] // serde と Debug/Clone derive を追加！
pub enum ClientMessage {
    /// プレイヤーがゲームに参加しようとした時に送るよ。
    /// プレイヤー名とか、何か識別子を送る？今はシンプルに空で！
    JoinGame { player_name: String },

    /// プレイヤーがカードを移動させようとした時に送るよ。
    MakeMove {
        /// 動かしたいカード（またはカードのスタック）のエンティティID。
        /// どのカードを掴んだかを示す！
        moved_entity: Entity,
        /// 移動先のスタックの種類（またはそのスタックの一番上のカードのエンティティID？）。
        /// どこに置こうとしているかを示す！
        /// Option<StackType> や Option<Entity> にするかは、サーバー側の実装と相談かな。
        /// ここではシンプルに StackType にしてみる。
        target_stack: StackType,
        // TODO: 場札の複数枚移動とかも考慮すると、もっと情報が必要かも？
        //       (例: moved_entities: Vec<Entity> とか)
    },

    // TODO: 他にも必要そうなメッセージを追加していく！
    // 例:
    // /// 山札をクリックしてカードをめくるアクション
    // DrawFromStock,
    // /// Waste（めくった札置き場）から山札にカードを戻すアクション (クロンダイクのルールによる)
    // ResetWasteToStock,
    /// ゲームの状態を要求する (接続直後とか？)
    RequestGameState,
    /// 初期ゲーム状態をサーバーに提供するためのメッセージ！
    ProvideInitialState { initial_state: GameStateData },
    /// 生存確認のためのメッセージ（接続が切れてないか確認）
    Ping,
}

// --- サーバーからクライアントへ送るメッセージ (Server-to-Client: S2C) ---

/// サーバーがクライアントに送るメッセージの種類を表すenumだよ。
/// サーバーから送られてきたJSONをこれに変換して処理する！
#[derive(Serialize, Deserialize, Debug, Clone)] // serde と Debug/Clone derive を追加！
pub enum ServerMessage {
    /// ゲームへの参加が成功した時に、サーバーが送ってくるよ。
    GameJoined {
        /// サーバーが割り当てたプレイヤーID。
        your_player_id: PlayerId,
        /// ゲームの初期状態 (もしかしたら GameStateUpdate でまとめて送られてくるかも？)
        initial_game_state: GameStateData, // GameStateData は下で定義！
    },

    /// ゲームの現在の状態をまるごと送ってくるよ。
    /// 誰かがカードを動かしたり、プレイヤーが参加/退出したりした時に送られてくる想定。
    GameStateUpdate {
        /// 最新のゲーム状態。
        current_game_state: GameStateData,
    },

    /// カード移動リクエストが不正だった場合に、サーバーが送ってくるよ。
    MoveRejected {
        /// 不正だった理由を示すメッセージ (デバッグ用とか？)
        reason: String,
    },

    /// 他のプレイヤーがゲームに参加した時に、サーバーが全員に通知するよ。
    PlayerJoined {
        player_id: PlayerId,
        player_name: String,
    },

    /// 他のプレイヤーがゲームから退出した時に、サーバーが全員に通知するよ。
    PlayerLeft {
        player_id: PlayerId,
    },

    /// サーバーからのPongメッセージ（Pingへの応答）。
    Pong,

    /// 何かエラーが発生した時に、サーバーが送ってくるよ。
    Error {
        message: String,
    },
}

// --- メッセージ内で使うデータ構造 --- 
// 上の ClientMessage や ServerMessage の中で使われる、
// ちょっと複雑なデータ構造をここで定義しておくよ。

/// プレイヤーを識別するためのID。サーバー側で管理される想定。
/// u32 のエイリアス (別名) にしてみる。シンプル！
pub type PlayerId = u32;

/// ゲームの状態全体を表すデータ構造だよ。
/// ServerMessage の GameJoined や GameStateUpdate で使われる。
/// サーバーから送られてきたこの情報をもとに、クライアント側の `World` を更新する感じになる。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateData {
    /// ゲームに参加しているプレイヤーの情報リスト。
    pub players: Vec<PlayerData>,
    /// 現在の全てのカードの状態リスト。
    pub cards: Vec<CardData>,
    // TODO: ゲームのステータス（誰かのターン、勝利/敗北状態など）も必要なら追加する。
    // pub game_status: GameStatusData, 
    // TODO: 山札 (Stock) や Waste の状態も個別に持つ必要があるかも？
}

/// プレイヤーの情報を表すデータ構造。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerData {
    pub id: PlayerId,
    pub name: String,
    // TODO: スコアとか、他のプレイヤー情報が必要なら追加！
}

/// カード1枚の状態を表すデータ構造。
/// `GameStateData` の中でたくさん使われるよ。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardData {
    /// このカードに対応するエンティティID。
    /// クライアント側で `World` のエンティティと紐づけるために使う。
    pub entity: Entity,
    /// カードのスート (マーク)。
    pub suit: Suit,
    /// カードのランク (数字)。
    pub rank: Rank,
    /// カードが表向きかどうか。
    pub is_face_up: bool,
    /// このカードが現在どのスタックに属しているか。
    pub stack_type: StackType,
    /// そのスタックの中で何番目に積まれているか (0が一番下)。
    pub position_in_stack: u8,
    // ↓↓↓ カードの位置情報を追加！
    pub position: PositionData,
}

/// 位置情報 (x, y 座標) を表すデータ構造。
/// サーバーとクライアント間で位置情報をやり取りするために使う。
#[derive(Serialize, Deserialize, Debug, Clone)] // serde と Debug/Clone を derive！
pub struct PositionData {
    pub x: f32,
    pub y: f32,
}

/*
// TODO: 必要になったら GameStatus 用のデータ構造も定義
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameStatusData { ... }
*/

// これで基本的なメッセージの型定義はできたかな？
// マルチプレイソリティアに必要な情報は結構たくさんあるね！💦
// 実際にサーバーと通信しながら、必要に応じて追加・修正していく感じになりそう！💪 