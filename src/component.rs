// src/component.rs

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
// Rust の Any 型を使うためにインポートするよ。
// use std::any::Any; // ComponentStorage で使ってるので必要！残す！
use std::any::Any; // 残す
// HashMap を使うためにインポート！
use std::collections::HashMap;

// さっき作った Entity 型をこのファイルでも使うからインポートするよ。
use crate::entity::Entity; // `crate::` は、このプロジェクト（クレート）のルートから見たパスって意味だよ。

// Component トレイトを使うためにインポート！ (このファイルで定義するので不要)
// // Note: Component is likely defined in world.rs, adjust if needed.
// // use crate::world::Component;
// // If Component trait is in this file, no need to import. Let's assume it's defined below for now.

/// Component（コンポーネント）トレイトだよ！
///
/// 構造体がゲームのコンポーネントとして使われる資格があることを示すマーカーだよ。
/// `Send + Sync + 'static` はマルチスレッド環境でも安全に使えるようにするためのおまじない！
/// `std::fmt::Debug` はデバッグ出力 (`{:?}`) できるようにするためだよ。
pub trait Component: std::fmt::Debug + Send + Sync + 'static {
    // 将来、共通メソッドが必要になったらここに追加できる！
    // fn reset(&mut self);
}

/// ComponentStorage（コンポーネントストレージ）だよ！
/// 特定種類のコンポーネントを `HashMap<Entity, T>` でまとめて保存・管理する箱！📦
/// `T: Component` は Component トレイトを実装した型しか入れられない制約だよ。
#[derive(Debug)] // デバッグ出力できるようにするよ！
pub struct ComponentStorage<T: Component> {
    components: HashMap<Entity, T>,
}

// ComponentStorage の実装ブロック！コンポーネント操作メソッドを定義するよ。
impl<T: Component> ComponentStorage<T> {
    /// 新しい空の ComponentStorage を作るよ！
    pub fn new() -> Self {
        Self {
            components: HashMap::new(), // 空の HashMap で初期化！
        }
    }

    /// エンティティにコンポーネントを追加・更新するよ！(上書き)
    pub fn insert(&mut self, entity: Entity, component: T) {
        self.components.insert(entity, component);
    }

    /// エンティティからコンポーネントを取得するよ！(読み取り専用)
    /// 戻り値: `Some(&T)` or `None`
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.components.get(&entity)
    }

    /// エンティティからコンポーネントを取得するよ！(書き込み可能)
    /// 戻り値: `Some(&mut T)` or `None`
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(&entity)
    }

    /// エンティティからコンポーネントを削除するよ！
    /// 戻り値: `Some(T)` (削除されたデータ) or `None`
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.components.remove(&entity)
    }

    /// 格納されている全コンポーネントのイテレーターを返すよ！(読み取り専用)
    /// `(&Entity, &T)` のタプルのイテレーターを返すよ。
    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.components.iter()
    }

    /// 格納されている全コンポーネントのイテレーターを返すよ！(書き込み可能)
    /// `(&Entity, &mut T)` のタプルのイテレーターを返すよ。
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut T)> {
        self.components.iter_mut()
    }

    /// ストレージが空かどうかを返すよ。
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// ストレージに含まれるコンポーネントの数を返すよ。
    pub fn len(&self) -> usize {
        self.components.len()
    }
}

// ComponentStorage も Default トレイトを実装！ `ComponentStorage::<T>::default()` で初期化！
impl<T: Component> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self::new() // new() 関数を呼ぶだけ！
    }
}

// --- Concrete Component Definitions ---
// ここからは Wasm 公開用、または components/ に定義がないものだけを残す！

// --- Position ---
// components/position.rs に基本的な定義があるので、ここでは #[wasm_bindgen] 付きの定義のみ残すか検討。
// 今のところ Position は #[wasm_bindgen] が付いていないので、一旦コメントアウトまたは削除。
// 必要になったら Wasm 公開用の struct を別途定義し、From 実装を追加する方針が良いかも。
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[wasm_bindgen(getter_with_clone)]
// pub struct Position {
//     pub x: f64,
//     pub y: f64,
// }
// impl Component for Position {}

// --- Suit (Wasm 公開用) ---
// components/card.rs の Suit から変換するための From 実装付き！
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

// --- Rank (Wasm 公開用) ---
// components/card.rs の Rank から変換するための From 実装付き！
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Rank {
    Ace = 1, // エースは 1
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
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

// --- Card ---
// components/card.rs に基本的な定義があるので、ここでは削除。
// 必要なら Wasm 公開用を別途定義する。
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[wasm_bindgen(getter_with_clone)]
// pub struct Card {
//     pub suit: Suit, // このファイル内の Wasm 用 Suit を使う想定だった？
//     pub rank: Rank, // このファイル内の Wasm 用 Rank を使う想定だった？
//     pub is_face_up: bool,
// }
// impl Component for Card {}

// --- StackType (Wasm 公開用) ---
// components/stack.rs の StackType から変換するための From 実装付き！
// こちらは Tableau/Foundation のインデックスを持たないシンプルなバージョン。
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StackType {
    Tableau,   // 場札
    Foundation,// 組札
    Stock,     // 山札
    Waste,     // 捨札
    Hand,      // 手札 (ドラッグ中)
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

// --- StackInfo ---
// components/stack.rs に基本的な定義があるので、ここでは削除。
// 必要なら Wasm 公開用を別途定義する。
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[wasm_bindgen(getter_with_clone)]
// pub struct StackInfo {
//     pub stack_type: StackType, // このファイル内の Wasm 用 StackType を使う想定だった？
//     pub stack_index: u8,
//     pub position_in_stack: u8,
// }
// impl Component for StackInfo {}

// --- Player ---
// components/player.rs に定義があるので、ここでは削除。
// ID の型が違う (u32 vs String) ので注意が必要！ protocol.rs との整合性も要確認。
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[wasm_bindgen(getter_with_clone)]
// pub struct Player {
//     pub id: String, // WebSocket などから割り当てられる一意な ID
// }
// impl Component for Player {}

// --- DraggingInfo (components/ に対応がないので残す) ---
/// ドラッグ中のカードに関する情報を表すコンポーネントだよ！🖱️➡️🃏
/// これは内部的な状態管理に使うもので、Wasm 公開は不要かも？ (一旦 #[wasm_bindgen] は付けない)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DraggingInfo {
    pub original_position_in_stack: usize, // u8 の方が良いかも？
    pub original_stack_entity: Entity, // Entity ID を直接持つ？ u32 がいい？
    pub original_x: f64, // f32 の方が良いかも？
    pub original_y: f64,
}
impl Component for DraggingInfo {} // Component トレイトは実装しておく

// --- GameState (Wasm 公開用) ---
// components/game_state.rs の GameStatus とは別の、よりシンプルなゲーム全体の状態？
// こちらは進行状況を表す enum みたいだね。必要そうなので残す。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum GameState {
    WaitingForPlayers, // プレイヤー待ち
    Dealing,           // カード配布中
    Playing,           // プレイ中
    GameOver,          // ゲーム終了
    Won,               // ★追加: `WinConditionSystem` が使う `Won` 状態も Wasm に公開？
}
// GameState をコンポーネントとして使うかは微妙だけど、Component トレイトを実装しておく。
// (シングルトンエンティティに持たせる設計なら使う)
impl Component for GameState {}

// --- ComponentStorage のテスト ---
// (テストは変更なし)
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの要素を使う宣言
    // EntityManager は entity.rs にあるので、テスト内で直接使う場合はインポートが必要
    use crate::entity::EntityManager;

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
        // HashMap のイテレーション順序は保証されないので、結果のチェック方法を少し変更
        let mut found1 = false;
        let mut found2 = false;
        for (entity, pos) in storage.iter() {
            if *entity == entity1 {
                assert_eq!(pos, &pos1, "エンティティ1のイテレーター結果が違う！");
                found1 = true;
            } else if *entity == entity2 {
                assert_eq!(pos, &pos2, "エンティティ2のイテレーター結果が違う！");
                found2 = true;
            } else {
                panic!("想定外のエンティティが見つかった！");
            }
            count += 1;
        }
        assert!(found1 && found2, "両方のエンティティが見つからなかった！");
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
        // コンポーネントストレージを直接操作するのではなく、World を介して追加するべきだが、
        // このテストは ComponentStorage 単体のテストなのでこのままでOK。
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
// (削除済み)
// Component トレイトと ComponentStorage の定義は src/world.rs や src/storage.rs に
// 移動したほうが構造的に綺麗かもしれない。
// このファイルは純粋にコンポーネントの型定義に専念させるとかね！
// → いや、Component トレイトと ComponentStorage はここで定義するのが自然かも。
//   具体的なコンポーネント定義は components/ に分ける方針は良さそう！ 