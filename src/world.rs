// src/world.rs

// Rust の標準ライブラリから、Any と TypeId を使うよ。
// Any: さっき component.rs でも出てきたけど、具体的な型を隠蔽して扱えるようにするやつ。
// TypeId: 型ごとにユニークなIDを取得するためのもの。これで ComponentStorage を型安全に管理する！
use std::any::{Any, TypeId};
// HashMap: キーと値のペアを効率的に格納するデータ構造。ComponentStorage を管理するのに使う！
use std::collections::HashMap;
// use std::rc::Rc; // 未使用なので削除！

// 自作の entity モジュールから Entity を使う。
use crate::entity::Entity;
// 自作の component モジュールから Component と ComponentStorage を使う。
use crate::component::{Component, ComponentStorage};

/// ゲーム世界の全てのエンティティとコンポーネントを管理する中心的な存在 (自作版！)
#[derive(Default)] // Default トレイトで new() を簡単に実装
pub struct World {
    // エンティティIDのカウンター
    pub(crate) next_entity_id: usize,
    // コンポーネントストレージを保持する HashMap
    // キー: TypeId (コンポーネントの型)
    // 値: Box<dyn Any> (任意の ComponentStorage を保持)
    pub(crate) components: HashMap<TypeId, Box<dyn Any>>,
    // TODO: 他に必要なフィールド (例: 削除済みエンティティリスト) を追加
}

impl World {
    /// 新しい空の World を作成するよ。
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            components: HashMap::new(),
            // TODO: 他のフィールドの初期化
        }
    }

    // --- ここから自作ECSのメソッドを実装していく！ ---
    // 例: pub fn create_entity(&mut self) -> Entity { ... }
    // 例: pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) { ... }
    // 例: pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> { ... }
    // ... などなど

    /// 新しいエンティティを作成するよ。
    pub fn create_entity(&mut self) -> Entity {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        // log(&format!("World: Created entity with ID {}", entity_id)); // log マクロはここでは使えないのでコメントアウト
        Entity(entity_id) // entity.rs で Entity が pub struct Entity(pub usize); と定義されている前提
    }

    // とりあえず hecs にあったメソッドのダミーをいくつか追加（エラー解消のため）
    // TODO: これらをちゃんと実装する！
    pub fn register_component<T: Component>(&mut self) { unimplemented!(); }
    pub fn get_all_entities_with_component<T: Component + 'static>(&self) -> Vec<Entity> { unimplemented!(); }
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> { unimplemented!(); }
    pub fn create_entity_with_id(&mut self, entity: Entity) { unimplemented!(); }
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) { unimplemented!(); }
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> { unimplemented!(); }
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> { unimplemented!(); }

    // system.rs から必要とされているメソッド
    // TODO: ちゃんと実装する
    pub fn storage_mut<T: Component>(&mut self) -> Option<&mut dyn Any> { unimplemented!() }
    pub fn storage<T: Component>(&self) -> Option<&dyn Any> { unimplemented!() }
}

// (既存のテストコードは hecs 依存なので一旦コメントアウト or 削除)
/*
#[cfg(test)]
mod tests {
    // ...
}
*/ 