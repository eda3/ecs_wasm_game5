// src/component.rs

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
// Rust の Any 型を使うためにインポートするよ。
// これを使うと、具体的な型が分からなくても、型情報を扱えるようになるんだ！
// コンポーネントストレージを管理する時にちょっと役立つテクニックだよ。(後で使うかも？🤔)
// use std::any::Any;
// HashMap を使うためにインポート！キーと値のペアを効率的に格納できるデータ構造だよ。
// Entity ID をキーにして、コンポーネントのデータを値として保存するのにピッタリ！👍
use std::collections::HashMap;

// さっき作った Entity 型をこのファイルでも使うからインポートするよ。
use crate::entity::Entity; // `crate::` は、このプロジェクト（クレート）のルートから見たパスって意味だよ。
// Component トレイトを使うためにインポート！
// Note: Component is likely defined in world.rs, adjust if needed.
// use crate::world::Component;
// If Component trait is in this file, no need to import. Let's assume it's defined below for now.

/// Component（コンポーネント）トレイトだよ！
///
/// トレイトっていうのは、特定の機能を実装するための「契約」みたいなものだよ。
/// この `Component` トレイトは、構造体がゲームのコンポーネントとして
/// 使われる資格があることを示すマーカー（目印）として機能するんだ。
///
/// 今はメソッド（具体的な機能）は何もないけど、将来的に共通の処理が必要になったら、
/// ここに追加できるよ！拡張性があるってことだね！🚀
///
/// `Send + Sync + 'static` っていうのは、ちょっと難しいけど、
/// マルチスレッド（複数の処理を同時に動かす）環境でも安全に使えるようにするための制約だよ。
/// `'static` は、コンポーネントがプログラムの実行中ずっと存在する可能性があることを示すよ。
/// これらを付けておくと、後で困ることが少なくなるんだ！😌
pub trait Component: std::fmt::Debug + Send + Sync + 'static {
    // 将来、全てのコンポーネントに共通するメソッドが必要になったら、ここに追加できるよ！
    // 例えば、コンポーネントをリセットする機能とか？🤔
    // fn reset(&mut self);
}

/// ComponentStorage（コンポーネントストレージ）だよ！
///
/// これは、特定の種類のコンポーネント（例えば Position コンポーネントとか）を
/// たくさんまとめて保存しておくための箱みたいなものだよ。📦
///
/// `HashMap<Entity, T>` を使ってるのは、
/// - キー: `Entity` (どのエンティティのコンポーネントかを示すID)
/// - 値: `T` (実際のコンポーネントデータ。`T` はジェネリクスで、Position とか Card とか、色々な型が入るよ！)
/// こうすることで、「エンティティIDが X の Position コンポーネントはこれ！」みたいに、
/// 素早くデータを取り出せるんだ！⚡️
///
/// `T: Component` っていうのは、「このストレージに入れられる型 `T` は、
/// 必ず `Component` トレイトを実装してないといけないよ！」っていう制約だよ。
/// これで、関係ないデータが紛れ込まないようにしてるんだ。賢い！😎
#[derive(Debug)] // デバッグ出力できるようにするよ！
pub struct ComponentStorage<T: Component> {
    // `components` フィールドが、実際のデータを保持する HashMap だよ。
    components: HashMap<Entity, T>,
}

// ComponentStorage の実装ブロックだよ！
// ここに、コンポーネントを操作するためのメソッド（関数）を定義していくよ。
impl<T: Component> ComponentStorage<T> {
    /// 新しい空の ComponentStorage を作るよ！
    pub fn new() -> Self {
        Self {
            components: HashMap::new(), // 空の HashMap で初期化！
        }
    }

    /// エンティティにコンポーネントを追加・更新するよ！
    ///
    /// もし `entity` が既にこのストレージにコンポーネントを持っていたら、
    /// 新しい `component` データで上書きされるよ。
    ///
    /// # 引数
    /// - `entity`: コンポーネントを追加したいエンティティのID
    /// - `component`: 追加するコンポーネントのデータ
    pub fn insert(&mut self, entity: Entity, component: T) {
        // HashMap の insert メソッドを使うだけ！簡単！😊
        self.components.insert(entity, component);
    }

    /// エンティティからコンポーネントを取得するよ！(読み取り専用)
    ///
    /// # 引数
    /// - `entity`: コンポーネントを取得したいエンティティのID
    ///
    /// # 戻り値
    /// - `Some(&T)`: エンティティがコンポーネントを持っていれば、その参照を返すよ。
    /// - `None`: エンティティがコンポーネントを持っていなければ、None を返すよ。
    ///
    /// `&T` っていうのは「参照」だよ。データのコピーを作らずに、データそのものを指し示すんだ。
    /// これで効率的にデータにアクセスできるよ！💨
    pub fn get(&self, entity: Entity) -> Option<&T> {
        // HashMap の get メソッドを使うだけ！便利！👍
        self.components.get(&entity)
    }

    /// エンティティからコンポーネントを取得するよ！(書き込み可能)
    ///
    /// # 引数
    /// - `entity`: コンポーネントを取得したいエンティティのID
    ///
    /// # 戻り値
    /// - `Some(&mut T)`: エンティティがコンポーネントを持っていれば、その可変参照を返すよ。これで中身を変更できる！✏️
    /// - `None`: エンティティがコンポーネントを持っていなければ、None を返すよ。
    ///
    /// `&mut T` っていうのは「可変参照」だよ。参照先のデータを変更できる特別な参照なんだ！
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        // HashMap の get_mut メソッドを使うだけ！これも便利！💪
        self.components.get_mut(&entity)
    }

    /// エンティティからコンポーネントを削除するよ！
    ///
    /// # 引数
    /// - `entity`: コンポーネントを削除したいエンティティのID
    ///
    /// # 戻り値
    /// - `Some(T)`: 削除されたコンポーネントのデータを返すよ。(もし必要なら使える！)
    /// - `None`: エンティティがコンポーネントを持っていなければ、None を返すよ。
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        // HashMap の remove メソッドを使うだけ！🗑️
        self.components.remove(&entity)
    }

    /// このストレージに格納されている全てのコンポーネント（と対応するエンティティ）
    /// を順番に処理するためのイテレーターを返すよ！
    ///
    /// イテレーターっていうのは、要素を一つずつ順番に取り出せる便利な仕組みだよ。
    /// for ループとかでよく使う！🔄
    ///
    /// `(&Entity, &T)` のタプルのイテレーターを返すよ。（読み取り専用）
    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.components.iter()
    }

    /// このストレージに格納されている全てのコンポーネント（と対応するエンティティ）
    /// を順番に処理するためのイテレーターを返すよ！(書き込み可能)
    ///
    /// `(&Entity, &mut T)` のタプルのイテレーターを返すよ。（書き込み可能）
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut T)> {
        self.components.iter_mut()
    }

    /// このストレージが空かどうかを返すよ。
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// このストレージに含まれるコンポーネントの数を返すよ。
    pub fn len(&self) -> usize {
        self.components.len()
    }
}

// ComponentStorage も Default トレイトを実装しておこう！
// これで `ComponentStorage::<Position>::default()` みたいに簡単に初期化できる！
impl<T: Component> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self::new() // new() 関数を呼ぶだけ！
    }
}

// --- Concrete Component Definitions ---
// ここから具体的なコンポーネントを定義していくよ！

/// 位置情報を表すコンポーネントだよ。エンティティがどこにいるかを示す！📍
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Position {
    pub x: f64, // X座標。f64 は倍精度浮動小数点数。JS の Number 型と互換性があるよ。
    pub y: f64, // Y座標。
}
// Position はコンポーネントだよ、ということを示すために Component トレイトを実装！
impl Component for Position {}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Heart,   // ❤️
    Diamond, // ♦️
    Club,    // ♣️
    Spade,   // ♠️
}

impl From<crate::components::card::Suit> for Suit {
    fn from(other_suit: crate::components::card::Suit) -> Self {
        match other_suit {
            crate::components::card::Suit::Heart => Suit::Heart,
            crate::components::card::Suit::Diamond => Suit::Diamond,
            crate::components::card::Suit::Club => Suit::Club,
            crate::components::card::Suit::Spade => Suit::Spade,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)] // Ord/PartialOrd で順序付けできるように
pub enum Rank {
    Ace = 1, // エースは 1
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,  // ジャック
    Queen, // クイーン
    King,  // キング
}

impl From<crate::components::card::Rank> for Rank {
    fn from(other_rank: crate::components::card::Rank) -> Self {
        match other_rank {
            crate::components::card::Rank::Ace => Rank::Ace,
            crate::components::card::Rank::Two => Rank::Two,
            crate::components::card::Rank::Three => Rank::Three,
            crate::components::card::Rank::Four => Rank::Four,
            crate::components::card::Rank::Five => Rank::Five,
            crate::components::card::Rank::Six => Rank::Six,
            crate::components::card::Rank::Seven => Rank::Seven,
            crate::components::card::Rank::Eight => Rank::Eight,
            crate::components::card::Rank::Nine => Rank::Nine,
            crate::components::card::Rank::Ten => Rank::Ten,
            crate::components::card::Rank::Jack => Rank::Jack,
            crate::components::card::Rank::Queen => Rank::Queen,
            crate::components::card::Rank::King => Rank::King,
        }
    }
}

/// カード情報を表すコンポーネントだよ。どんなカードかを示す！🃏
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Card {
    pub suit: Suit, // カードのスート（マーク）
    pub rank: Rank, // カードのランク（数字）
    pub is_face_up: bool, // カードが表向きか裏向きか
    // 必要なら他の情報（例：どのスタックに属しているか）も追加できる
}
// Card もコンポーネントだよ！
impl Component for Card {}

/// スタック（カードの山）の種類を示すよ！
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StackType {
    Tableau,   // 場札 (7列のやつ)
    Foundation,// 組札 (AからKまで積むところ)
    Stock,     // 山札 (まだ配られてないカード)
    Waste,     // 捨札 (山札からめくったカード)
    Hand,      // 手札 (ドラッグ中のカード)
}

impl From<crate::components::stack::StackType> for StackType {
    fn from(other_stack_type: crate::components::stack::StackType) -> Self {
        match other_stack_type {
            // インデックス情報は無視して、種類だけをマッピングする
            crate::components::stack::StackType::Tableau(_) => StackType::Tableau,
            crate::components::stack::StackType::Foundation(_) => StackType::Foundation,
            crate::components::stack::StackType::Stock => StackType::Stock,
            crate::components::stack::StackType::Waste => StackType::Waste,
            crate::components::stack::StackType::Hand => StackType::Hand,
        }
    }
}

/// スタック情報を表すコンポーネントだよ。カードの山に関する情報！⛰️
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct StackInfo {
    pub stack_type: StackType, // スタックの種類
    pub stack_index: u8,      // スタックのインデックス（例：Tableau の何列目か）
    pub position_in_stack: u8, // スタックの中での順番（0が一番下）
}
// StackInfo もコンポーネントだよ！
impl Component for StackInfo {}

/// プレイヤー情報を表すコンポーネントだよ。接続してきたクライアントの情報！👤
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct Player {
    pub id: String, // WebSocket などから割り当てられる一意な ID
    // 必要ならプレイヤー名なども追加できる
}
// Player もコンポーネントだよ！
impl Component for Player {}

/// ドラッグ中のカードに関する情報を表すコンポーネントだよ！🖱️➡️🃏
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DraggingInfo {
    pub original_position_in_stack: usize,
    pub original_stack_entity: u32, // Changed from Entity to u32 for simplicity
    pub original_x: f64,
    pub original_y: f64,
}

impl Component for DraggingInfo {}

/// ゲーム全体の状態を表すコンポーネントだよ！🎮
/// 通常、こういう「全体の状態」はエンティティは持たないことが多いけど、
/// 特定のエンティティ（例：シングルトンエンティティ）に持たせる設計もあるよ。
/// 今回はサーバーの状態管理のために使うかもしれないので定義しておく。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum GameState {
    WaitingForPlayers, // プレイヤー待ち
    Dealing,           // カード配布中
    Playing,           // プレイ中
    GameOver,          // ゲーム終了
}
// GameState もコンポーネントだよ！
impl Component for GameState {}

// --- ComponentStorage のテスト ---
// (Tests should ideally be in their own module or file)
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの要素を使う宣言
    use crate::entity::EntityManager; // Entity を作るために EntityManager も使う

    // テストで使うためのダミーコンポーネントを定義するよ！
    // 位置情報を表す Position コンポーネント (Local to tests)
    #[derive(Debug, PartialEq, Clone)] // テストで比較したりクローンしたりできるようにする
    struct TestPosition { // Renamed to avoid conflict if Position struct is also used in tests directly
        x: f32,
        y: f32,
    }
    // Component トレイトを実装！
    impl Component for TestPosition {}

    // テストで使うためのダミーコンポーネント その２！
    // 速度情報を表す Velocity コンポーネント (Local to tests)
    #[derive(Debug, PartialEq, Clone)]
    struct TestVelocity { // Renamed
        dx: f32,
        dy: f32,
    }
    // Component トレイトを実装！
    impl Component for TestVelocity {}

    #[test]
    fn insert_and_get_component() {
        // EntityManager と Position 用のストレージを作る
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<TestPosition>::default(); // Use TestPosition

        // エンティティをいくつか作る
        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();

        // コンポーネントのデータを作る
        let pos1 = TestPosition { x: 10.0, y: 20.0 };
        let pos2 = TestPosition { x: 30.0, y: 40.0 };

        // ストレージにコンポーネントを追加！
        storage.insert(entity1, pos1.clone()); // clone() でコピーして渡す
        storage.insert(entity2, pos2.clone());

        // ちゃんと取得できるか確認！
        assert_eq!(storage.get(entity1), Some(&pos1), "エンティティ1のPositionが違う！😱");
        assert_eq!(storage.get(entity2), Some(&pos2), "エンティティ2のPositionが違う！😱");

        // 存在しないエンティティのコンポーネントを取得しようとしたら None になるか確認！
        let entity3 = manager.create_entity();
        assert_eq!(storage.get(entity3), None, "存在しないはずのコンポーネントが見つかった！👻");

        println!("コンポーネントの追加・取得テスト、成功！🎉");
    }

    #[test]
    fn get_mut_component() {
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<TestPosition>::default(); // Use TestPosition
        let entity1 = manager.create_entity();
        let pos1 = TestPosition { x: 10.0, y: 20.0 };
        storage.insert(entity1, pos1);

        // get_mut で可変参照を取得して、中身を変更してみる！✏️
        if let Some(pos_mut) = storage.get_mut(entity1) {
            pos_mut.x = 15.0; // x 座標を変更！
        } else {
            panic!("get_mut でコンポーネントを取得できなかった！😭");
        }

        // 変更が反映されてるか確認！
        assert_eq!(storage.get(entity1), Some(&TestPosition { x: 15.0, y: 20.0 }), "コンポーネントの変更が反映されてない！🤔");

        println!("コンポーネントの変更テスト、成功！🎉");
    }

    #[test]
    fn remove_component() {
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<TestPosition>::default(); // Use TestPosition
        let entity1 = manager.create_entity();
        let pos1 = TestPosition { x: 10.0, y: 20.0 };
        storage.insert(entity1, pos1.clone());

        // ちゃんと入ってることを確認
        assert!(storage.get(entity1).is_some(), "削除前にコンポーネントが存在しない！🥺");

        // コンポーネントを削除！🗑️
        let removed_component = storage.remove(entity1);

        // 削除されたコンポーネントが正しいか確認
        assert_eq!(removed_component, Some(pos1), "削除されたコンポーネントが違う！🤔");

        // 削除されたか確認
        assert!(storage.get(entity1).is_none(), "コンポーネントが削除されていない！😨");

        println!("コンポーネントの削除テスト、成功！🎉");
    }

    #[test]
    fn iter_components() {
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<TestPosition>::default(); // Use TestPosition
        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();
        let pos1 = TestPosition { x: 10.0, y: 20.0 };
        let pos2 = TestPosition { x: 30.0, y: 40.0 };
        storage.insert(entity1, pos1.clone());
        storage.insert(entity2, pos2.clone());

        let mut count = 0;
        // iter() でイテレーターを取得してループ！
        for (entity, pos) in storage.iter() {
            // 正しい組み合わせが見つかるかチェック
            if *entity == entity1 {
                assert_eq!(pos, &pos1, "エンティティ1のイテレーター結果が違う！");
            } else if *entity == entity2 {
                assert_eq!(pos, &pos2, "エンティティ2のイテレーター結果が違う！");
            } else {
                panic!("想定外のエンティティが見つかった！");
            }
            count += 1;
        }
        assert_eq!(count, 2, "イテレーターで見つかったコンポーネント数が違う！");

        println!("コンポーネントのイテレーションテスト、成功！🎉");
    }

    #[test]
    fn iter_mut_components() {
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<TestPosition>::default(); // Use TestPosition
        let entity1 = manager.create_entity();
        let pos1 = TestPosition { x: 10.0, y: 20.0 };
        storage.insert(entity1, pos1);

        // iter_mut() で可変参照を取得して変更！
        for (_entity, pos) in storage.iter_mut() {
            pos.x += 1.0;
        }

        // 変更が反映されているか確認
        assert_eq!(storage.get(entity1), Some(&TestPosition { x: 11.0, y: 20.0 }), "iter_mut による変更が反映されていない！");

        println!("コンポーネントの可変イテレーションテスト、成功！🎉");
    }

    #[test]
    fn different_component_types() {
        let manager = EntityManager::default();
        // Position 用と Velocity 用のストレージをそれぞれ作る
        let mut pos_storage = ComponentStorage::<TestPosition>::default(); // Use TestPosition
        let mut vel_storage = ComponentStorage::<TestVelocity>::default(); // Use TestVelocity

        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();

        let pos1 = TestPosition { x: 1.0, y: 2.0 };
        let vel1 = TestVelocity { dx: 3.0, dy: 4.0 };
        let pos2 = TestPosition { x: 5.0, y: 6.0 };

        // エンティティ1 には Position と Velocity の両方を追加
        pos_storage.insert(entity1, pos1.clone());
        vel_storage.insert(entity1, vel1.clone());
        // エンティティ2 には Position のみ追加
        pos_storage.insert(entity2, pos2.clone());

        // ちゃんと取得できるか確認
        assert_eq!(pos_storage.get(entity1), Some(&pos1));
        assert_eq!(vel_storage.get(entity1), Some(&vel1));
        assert_eq!(pos_storage.get(entity2), Some(&pos2));
        // エンティティ2 は Velocity を持っていないはず
        assert_eq!(vel_storage.get(entity2), None);

        println!("複数コンポーネントタイプのテスト、成功！🎉");
    }
}
// --- ここから下は削除 ---

// Component トレイトと ComponentStorage の定義は src/world.rs や src/storage.rs に
// 移動したほうが構造的に綺麗かもしれない。
// このファイルは純粋にコンポーネントの型定義に専念させるとかね！ 