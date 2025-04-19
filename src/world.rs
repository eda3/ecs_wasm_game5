// src/world.rs

// === Rust 標準ライブラリからのインポート ===
// Any: 実行時に型情報を扱うためのトレイト。コンポーネントストレージを型に関係なく保持するために使う。
// TypeId: プログラム実行中に、それぞれの型にユニークなIDを割り当てるためのもの。コンポーネントの種類を区別するキーとして使う。
use std::any::{Any, TypeId};
// HashMap: キーと値のペアを高速に格納・検索できるデータ構造。TypeId をキーにして、その型のコンポーネントストレージ (Box<dyn Any> でラップ) を値として保持するのに使う。
use std::collections::HashMap;
// HashSet: 重複しない要素を格納するデータ構造。現在生存しているエンティティIDを管理するのに使う。
use std::collections::HashSet;

// === このクレート (プロジェクト) 内の他のモジュールからのインポート ===
// Entity: エンティティを表す単純な構造体 (通常はIDをラップしたもの)。
use crate::entity::Entity;
// Component: 全てのコンポーネントが実装すべきマーカートレイト (中身は空でもOK)。ジェネリクスでコンポーネント型を制約するのに使う。
use crate::component::Component;
// ComponentStorage: 特定の型のコンポーネントを Entity をキーとして格納するトレイト (今回は使わないかも？ HashMap<Entity, T> を直接使う方針)
// use crate::component::{Component, ComponentStorage}; // ★削除: ComponentStorage はもう使わない！

/// ゲーム世界の全てのエンティティとコンポーネントを管理する中心的な構造体 (自作ECSのコア！)。
/// エンティティの生存管理、コンポーネントの型ごとの保存とアクセス機能を提供するよ。
// #[derive(Default)] // Defaultトレイトは使うフィールドが増えると手動実装の方が良くなるので削除。new() をちゃんと書く。
pub struct World {
    /// 現在生存しているエンティティIDのセット。エンティティが存在するかどうかを高速にチェックできる。
    entities: HashSet<Entity>,
    /// 次に生成するエンティティに割り当てるID。エンティティが作成されるたびにインクリメントされる。
    next_entity_id: usize,
    /// コンポーネントの種類 (TypeId) ごとに、その型のコンポーネントデータを格納するストレージ。
    /// `TypeId` をキーとし、`Box<dyn Any>` を値として持つ HashMap。
    /// `Box<dyn Any>` の中身は、実際には `HashMap<Entity, T>` (T は具体的なコンポーネント型) が入ってる。
    /// `dyn Any` を使うことで、様々な型の `HashMap<Entity, T>` を一つの HashMap で管理できる！ (型消去というテクニック)
    component_stores: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    /// 新しい空の World を作成するコンストラクタ。
    /// 各フィールドを初期状態 (空の HashSet, ID カウンタ 0, 空の HashMap) に設定する。
    pub fn new() -> Self {
        World {
            entities: HashSet::new(),
            next_entity_id: 0,
            component_stores: HashMap::new(),
        }
    }

    /// 新しいエンティティを生成し、その Entity を返す。
    /// `next_entity_id` をインクリメントして、ユニークなIDを保証する。
    /// 生成されたエンティティIDは `entities` セットにも追加される。
    ///
    /// # 戻り値
    /// 新しく作成された `Entity`。
    pub fn create_entity(&mut self) -> Entity {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        let entity = Entity(entity_id);
        // 新しく作った Entity を生存リストに追加
        self.entities.insert(entity);
        // log(&format!("World: Created entity with ID {}", entity_id)); // logマクロは wasm_bindgen 経由じゃないと使えないのでコメントアウト
        println!("World: Created entity with ID {}", entity_id); // 標準出力で代替 (デバッグ用)
        entity
    }

    /// 指定されたIDで新しいエンティティを作成する。
    /// サーバーから受け取った状態を再現する場合などに使うことを想定。
    /// **注意:** 既に存在するIDを指定した場合、既存のエンティティは上書きされず、
    ///       単に `entities` セットに追加されるだけ (セットなので重複はしない)。
    ///       ID の衝突管理はこのメソッドの責務外。呼び出し側で注意が必要。
    ///
    /// # 引数
    /// * `entity` - 作成したいエンティティの `Entity` (IDを含む)。
    pub fn create_entity_with_id(&mut self, entity: Entity) {
        // 指定された Entity を生存リストに追加
        self.entities.insert(entity);
        // next_entity_id を必要なら更新 (指定IDが現在値以上なら、次のIDが重複しないように)
        // entity.0 は Entity 構造体のタプル要素 (pub usize) にアクセスする方法
        self.next_entity_id = self.next_entity_id.max(entity.0 + 1);
        println!("World: Created entity with specific ID {}", entity.0); // 標準出力
    }

    /// 指定されたエンティティが存在するかどうかを確認する。
    ///
    /// # 引数
    /// * `entity` - 存在を確認したいエンティティ。
    ///
    /// # 戻り値
    /// エンティティが存在すれば `true`、しなければ `false`。
    pub fn is_entity_alive(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    /// 指定されたエンティティを削除 (破棄) する。
    /// このエンティティに紐づけられている全てのコンポーネントも削除される **(TODO: 現状未実装！)**。
    ///
    /// # 引数
    /// * `entity` - 削除したいエンティティ。
    ///
    /// # 戻り値
    /// エンティティが存在し、正常に削除された場合は `true`。
    /// エンティティが存在しなかった場合は `false`。
    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        // まず、生存リストからエンティティを削除
        if self.entities.remove(&entity) {
            println!("World: Destroying entity with ID {}", entity.0); // 標準出力
            // 次に、全てのコンポーネントストレージをイテレートして、
            // このエンティティに関連するコンポーネントを削除する。
            // `component_stores` の値 (Box<dyn Any>) に対して操作を行う必要がある。
            // ここで `Any` トレイトのメソッド (`downcast_mut` など) を使うことになる。
            // 各ストレージは `HashMap<Entity, _>` である想定。
            for (_type_id, _storage_any) in self.component_stores.iter_mut() { // ★警告修正: storage_any を _storage_any に
                // Box<dyn Any> から中のデータへの可変参照を取得しようとする。
                // ただし、具体的な型がわからないと HashMap の remove は呼べない。
                // ここでちょっと困る。どうやって型ごとに remove を呼ぶか？
                // -> 解決策: Component トレイトに「指定エンティティのデータを削除する」メソッドを追加するか、
                //    あるいは、World が各ストレージの具体的な型を知っている Map (TypeId -> remover function?) を持つ必要がある。
                //    より簡単なのは、登録時に型ごとの削除関数も登録しておくことかも。
                //
                // *** 一旦、削除処理の詳細は保留 (この部分は結構難しい！) ***
                // *** 理想的な実装には、トレイトオブジェクトやマクロなどが関係してくるかも。 ***
                // *** 今はまず、コンポーネントの追加・取得を優先して実装しよう！ ***
                // TODO: エンティティ削除時にコンポーネントも削除するロジックを実装する。
            }
            true // 生存リストから削除できたので true を返す
        } else {
            println!("World: Attempted to destroy non-existent entity with ID {}", entity.0);
            false // エンティティが存在しなかったので false
        }
    }

    /// 新しい型のコンポーネントを World に登録する。
    /// これにより、その型のコンポーネントをエンティティに追加できるようになる。
    /// 内部的には、そのコンポーネント型用のストレージ (HashMap<Entity, T>) を初期化してる。
    ///
    /// # 型パラメータ
    /// * `T` - 登録したいコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、
    ///         `'static` ライフタイムを持つ必要がある (`'static` はデータがプログラム終了まで生存可能という意味)。
    ///
    /// # パニック
    /// すでに同じ型のコンポーネントが登録されている場合にパニックする可能性がある (HashMap::insert の仕様による)。
    /// 通常はゲーム初期化時に一度だけ呼ぶ。
    pub fn register_component<T: Component + Any + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        println!("World: Registering component type with ID {:?}", type_id); // 標準出力
        // 新しい空の HashMap<Entity, T> を作成し、それを Box<dyn Any> で包んで component_stores に挿入する。
        let new_storage: HashMap<Entity, T> = HashMap::new();
        self.component_stores.insert(type_id, Box::new(new_storage));
    }

    /// 指定されたエンティティにコンポーネントを追加する。
    /// もしエンティティが生存していなければ、コンポーネントは追加されない (エラーにはならず、単に無視)。
    /// もし指定された型のコンポーネントストレージが存在しなければ (register_component 忘れ)、パニックする。
    /// もしエンティティに既に同じ型のコンポーネントが存在する場合、上書きされる。
    ///
    /// # 型パラメータ
    /// * `T` - 追加するコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 引数
    /// * `entity` - コンポーネントを追加する対象のエンティティ。
    /// * `component` - 追加するコンポーネントのインスタンス。
    pub fn add_component<T: Component + Any + 'static>(&mut self, entity: Entity, component: T) {
        // エンティティが生きてるかチェック (死んでるエンティティには追加しない)
        if !self.is_entity_alive(entity) {
            println!("World: Attempted to add component to non-existent entity {}", entity.0);
            return; // 何もせずに関数を抜ける
        }

        let type_id = TypeId::of::<T>();
        println!("World: Adding component {:?} to entity {}", type_id, entity.0); // 標準出力

        // 1. `component_stores` から `TypeId` に対応する `Box<dyn Any>` を可変参照で取得する。
        //    `get_mut` は `Option<&mut Box<dyn Any>>` を返す。
        if let Some(storage_any) = self.component_stores.get_mut(&type_id) {
            // 2. `Box<dyn Any>` から、目的の型 `HashMap<Entity, T>` への可変参照を取得する。
            //    `downcast_mut::<HashMap<Entity, T>>()` を使う。これは `Option<&mut HashMap<Entity, T>>` を返す。
            //    ダウンキャストが成功すれば (つまり、Box の中身が本当に HashMap<Entity, T> なら) Some が返る。
            if let Some(storage) = storage_any.downcast_mut::<HashMap<Entity, T>>() {
                // 3. ダウンキャスト成功！ ストレージ (HashMap) にエンティティとコンポーネントを挿入する。
                //    `insert` は、もしキーが既に存在していたら古い値を返す (今回は使わない)。
                storage.insert(entity, component);
            } else {
                // ダウンキャスト失敗。これは通常、プログラムのロジックエラー (型の不一致など)。
                // 例えば、TypeId::of::<T>() で取得した ID なのに、中身が HashMap<Entity, T> じゃなかった場合。ありえないはず。
                panic!("World: Component storage downcast failed for TypeId {:?}. This should not happen!", type_id);
            }
        } else {
            // `component_stores` に `TypeId` が存在しない場合。`register_component<T>()` を呼び忘れている可能性が高い。
            panic!("World: Component type {:?} not registered! Call register_component<{}>() first.", type_id, std::any::type_name::<T>());
        }
    }

    /// 指定されたエンティティから、指定された型のコンポーネントへの **読み取り専用** 参照を取得する。
    ///
    /// # 型パラメータ
    /// * `T` - 取得したいコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 引数
    /// * `entity` - コンポーネントを取得したいエンティティ。
    ///
    /// # 戻り値
    /// コンポーネントが見つかれば `Some(&T)`、見つからなければ (エンティティが存在しない、
    /// その型のコンポーネントが登録されていない、エンティティがそのコンポーネントを持っていない場合など) `None`。
    pub fn get_component<T: Component + Any + 'static>(&self, entity: Entity) -> Option<&T> {
        // エンティティが生きてるか軽くチェック (必須ではないが、無駄な検索を省けるかも)
        if !self.is_entity_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        // 1. `component_stores` から `TypeId` に対応する `Box<dyn Any>` を取得。
        self.component_stores.get(&type_id)
            // 2. Option 型のメソッド `and_then` を使って、`Box<dyn Any>` があればダウンキャストを試みる。
            //    `and_then` は Some(v) ならクロージャを実行し、その結果 (Option) を返し、None なら None を返す。
            .and_then(|storage_any| {
                // 3. `Box<dyn Any>` から `HashMap<Entity, T>` への参照を取得。
                storage_any.downcast_ref::<HashMap<Entity, T>>()
            })
            // 4. Option 型のメソッド `and_then` をさらに使って、ストレージがあれば `get` を試みる。
            .and_then(|storage| {
                // 5. `HashMap` から `entity` キーに対応するコンポーネントへの参照 (`&T`) を取得する。
                //    `storage.get(&entity)` は `Option<&T>` を返す。
                storage.get(&entity)
            })
            // `and_then` を連鎖させることで、途中で None になったら最終結果も None になる、綺麗なコードになる！
    }

    /// 指定されたエンティティから、指定された型のコンポーネントへの **書き込み可能** 参照を取得する。
    ///
    /// # 型パラメータ
    /// * `T` - 取得したいコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 引数
    /// * `entity` - コンポーネントを取得したいエンティティ。
    ///
    /// # 戻り値
    /// コンポーネントが見つかれば `Some(&mut T)`、見つからなければ `None`。
    pub fn get_component_mut<T: Component + Any + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        // 可変参照を返すので、エンティティ生存チェックはここでするのが適切かも
        if !self.is_entity_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        // 1. `component_stores` から可変参照を取得。
        self.component_stores.get_mut(&type_id)
            // 2. `and_then` でダウンキャスト (可変参照版 `downcast_mut`)。
            .and_then(|storage_any| {
                storage_any.downcast_mut::<HashMap<Entity, T>>()
            })
            // 3. `and_then` で `HashMap` から可変参照を取得 (`get_mut`)。
            .and_then(|storage| {
                storage.get_mut(&entity)
            })
    }

    /// 指定されたエンティティから、指定された型のコンポーネントを削除する。
    ///
    /// # 型パラメータ
    /// * `T` - 削除したいコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 引数
    /// * `entity` - コンポーネントを削除したいエンティティ。
    ///
    /// # 戻り値
    /// コンポーネントが存在し、正常に削除された場合は `Some(T)` (削除されたコンポーネントの値)。
    /// 見つからなかった場合 (エンティティが存在しない、型が登録されていない、
    /// エンティティがそのコンポーネントを持っていない場合など) は `None`。
    pub fn remove_component<T: Component + Any + 'static>(&mut self, entity: Entity) -> Option<T> {
        // エンティティ生存チェック
        if !self.is_entity_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        // 1. 可変参照でストレージを取得。
        self.component_stores.get_mut(&type_id)
            // 2. 可変参照でダウンキャスト。
            .and_then(|storage_any| {
                storage_any.downcast_mut::<HashMap<Entity, T>>()
            })
            // 3. `and_then` ではなく `and_then` を使う (HashMap::remove は Option<T> を返すので、ネストしない)。
            //    `HashMap` から `entity` をキーとしてコンポーネントを削除し、その値を取得する。
            .and_then(|storage| {
                // `remove(&entity)` は `Option<T>` を返す。
                storage.remove(&entity)
            })
    }

    /// 指定された型のコンポーネントを持つ全ての **生存している** エンティティのリスト (`Vec<Entity>`) を返す。
    /// パフォーマンス: この操作は、該当するコンポーネントストレージ内の全てのキーを調べるため、
    ///              コンポーネントを持つエンティティ数に比例したコストがかかる。
    ///
    /// # 型パラメータ
    /// * `T` - 検索対象のコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 戻り値
    /// 指定されたコンポーネントを持つエンティティの `Vec<Entity>`。
    /// その型のコンポーネントが登録されていない場合は空の `Vec` を返す。
    pub fn get_all_entities_with_component<T: Component + Any + 'static>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();

        // 1. `component_stores` からストレージ (Box<dyn Any>) を取得。
        if let Some(storage_any) = self.component_stores.get(&type_id) {
            // 2. `HashMap<Entity, T>` にダウンキャスト。
            if let Some(storage) = storage_any.downcast_ref::<HashMap<Entity, T>>() {
                // 3. ストレージ (HashMap) のキー (Entity) をイテレートする。
                //    `keys()` はイテレータ `Keys<'_, Entity, T>` を返す。
                // 4. `filter` を使って、生存しているエンティティのみを抽出する。
                //    `self.is_entity_alive(*entity)` でチェック。`entity` は `&Entity` なので `*` でデリファレンス。
                // 5. `copied()` または `cloned()` を使って、参照 (`&Entity`) から値 (`Entity`) に変換する。
                //    `Entity` が `Copy` トレイトを実装していれば `copied()` が効率的。
                //    (entity.rs で `#[derive(Copy, Clone, ...)]` としている前提)
                // 6. `collect()` を使って、イテレータから `Vec<Entity>` を生成する。
                storage.keys()
                    .filter(|entity| self.is_entity_alive(**entity)) // 生存チェック
                    .copied() // Entity が Copy なら copied(), そうでなければ cloned()
                    .collect()
            } else {
                // ダウンキャスト失敗 (通常ありえない)
                Vec::new() // 空の Vec を返す
            }
        } else {
            // 型が登録されていない
            Vec::new() // 空の Vec を返す
        }
    }

    // === system.rs など、他のモジュールから必要とされる可能性のあるヘルパーメソッド ===
    // これらは直接コンポーネントを操作するのではなく、ストレージ自体へのアクセスを提供する。
    // より高度なクエリやシステムの実装で役立つかもしれない。

    /// 特定の型のコンポーネントストレージへの **読み取り専用** 参照 (`&dyn Any`) を返す。
    /// 呼び出し側でダウンキャストして使う必要がある。
    ///
    /// # 型パラメータ
    /// * `T` - アクセスしたいストレージのコンポーネント型。
    ///
    /// # 戻り値
    /// ストレージが見つかれば `Some(&dyn Any)`、なければ `None`。
    pub fn storage<T: Component + Any + 'static>(&self) -> Option<&dyn Any> {
        let type_id = TypeId::of::<T>();
        // `map` を使って `&Box<dyn Any>` を `&dyn Any` に変換する。
        // `as_ref()` は `Box<T>` から `&T` を得る標準的な方法。
        // `&*bx` のようにデリファレンスを使うこともできる。
        self.component_stores.get(&type_id).map(|bx| bx.as_ref())
        // self.component_stores.get(&type_id).map(|bx| &**bx) // これでも同じ
    }

    /// 特定の型のコンポーネントストレージへの **書き込み可能** 参照 (`&mut dyn Any`) を返す。
    /// 呼び出し側でダウンキャストして使う必要がある。
    ///
    /// # 型パラメータ
    /// * `T` - アクセスしたいストレージのコンポーネント型。
    ///
    /// # 戻り値
    /// ストレージが見つかれば `Some(&mut dyn Any)`、なければ `None`。
    pub fn storage_mut<T: Component + Any + 'static>(&mut self) -> Option<&mut dyn Any> {
        let type_id = TypeId::of::<T>();
        // `map` を使って `&mut Box<dyn Any>` を `&mut dyn Any` に変換する。
        // `as_mut()` は `Box<T>` から `&mut T` を得る標準的な方法。
        self.component_stores.get_mut(&type_id).map(|bx| bx.as_mut())
        // self.component_stores.get_mut(&type_id).map(|bx| &mut **bx) // これでも同じ
    }
}

// === テストモジュール ===
// `#[cfg(test)]` は、`cargo test` を実行した時だけコンパイルされるコードブロックを示す。
#[cfg(test)]
mod tests {
    // テストに必要なものを親モジュール (このファイルの上部) からインポート
    use super::*; // `World`, `Entity`, `Component` など
    use crate::component::Component; // Component トレイトを再度 use

    // --- テスト用のコンポーネントをいくつか定義 ---
    // derive(...) は、Rustコンパイラに特定のトレイトの実装を自動生成させる指示。
    // Debug: println! や assert_eq! で表示できるようにする。
    // Clone: 値をコピーできるようにする (.clone() メソッド)。テストで便利。
    // Copy: Clone より軽量なコピー (代入時にムーブでなくコピーされる)。単純な型なら可能。
    // PartialEq: `==` 演算子で比較できるようにする。アサーションで使う。
    // Eq: PartialEq とセットで、`a == a` が常に true であることを示すマーカー。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Position {
        x: i32,
        y: i32,
    }
    // Component トレイトを実装 (マーカーなので中身は空)
    impl Component for Position {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Velocity {
        dx: i32,
        dy: i32,
    }
    impl Component for Velocity {}

    // --- テストケース ---

    #[test]
    fn test_create_entity() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        assert_eq!(entity1.0, 0); // 最初の ID は 0
        assert_eq!(entity2.0, 1); // 次の ID は 1
        assert_eq!(world.next_entity_id, 2); // カウンタは 2 に進んでいる
        assert!(world.is_entity_alive(entity1)); // entity1 は生存している
        assert!(world.is_entity_alive(entity2)); // entity2 は生存している
        assert_eq!(world.entities.len(), 2); // 生存エンティティ数は 2
    }

    #[test]
    fn test_create_entity_with_id() {
        let mut world = World::new();
        let entity5 = Entity(5);
        world.create_entity_with_id(entity5);

        assert!(world.is_entity_alive(entity5));
        assert_eq!(world.entities.len(), 1);
        assert_eq!(world.next_entity_id, 6); // next_entity_id が更新されることを確認

        let entity3 = Entity(3);
        world.create_entity_with_id(entity3);
        assert!(world.is_entity_alive(entity3));
        assert_eq!(world.entities.len(), 2);
        assert_eq!(world.next_entity_id, 6); // next_entity_id は小さい ID を追加しても変わらない

        let entity8 = Entity(8);
        world.create_entity_with_id(entity8);
        assert!(world.is_entity_alive(entity8));
        assert_eq!(world.entities.len(), 3);
        assert_eq!(world.next_entity_id, 9); // 再度 next_entity_id が更新される
    }


    #[test]
    fn test_register_and_add_component() {
        let mut world = World::new();
        // コンポーネントを登録！ これを忘れると add_component でパニックする
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let pos1 = Position { x: 10, y: 20 };
        let vel1 = Velocity { dx: 1, dy: 0 };
        let pos2 = Position { x: -5, y: 15 };

        // entity1 に Position と Velocity を追加
        world.add_component(entity1, pos1); // pos1 はここでムーブされる (Copy じゃなければ)
        world.add_component(entity1, vel1); // vel1 もムーブ

        // entity2 に Position を追加
        world.add_component(entity2, pos2); // pos2 もムーブ

        // --- component_stores の中身を直接確認 (テスト目的) ---
        // Position 用のストレージがあるか確認
        let pos_type_id = TypeId::of::<Position>();
        assert!(world.component_stores.contains_key(&pos_type_id));
        // Velocity 用のストレージがあるか確認
        let vel_type_id = TypeId::of::<Velocity>();
        assert!(world.component_stores.contains_key(&vel_type_id));

        // Position ストレージの中身を確認 (ダウンキャストが必要)
        if let Some(pos_storage_any) = world.component_stores.get(&pos_type_id) {
            if let Some(pos_storage) = pos_storage_any.downcast_ref::<HashMap<Entity, Position>>() {
                assert_eq!(pos_storage.len(), 2); // 2つのエンティティが Position を持つ
                assert!(pos_storage.contains_key(&entity1));
                assert!(pos_storage.contains_key(&entity2));
            } else {
                panic!("Position storage downcast failed in test!");
            }
        } else {
            panic!("Position storage not found in test!");
        }
    }

    #[test]
    #[should_panic] // このテストはパニックすることを期待する
    fn test_add_component_unregistered() {
        let mut world = World::new();
        let entity = world.create_entity();
        let pos = Position { x: 0, y: 0 };
        // Position を register せずに add しようとするとパニックするはず！
        world.add_component(entity, pos);
    }

    #[test]
    fn test_get_component() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let entity1 = world.create_entity();
        let entity2 = world.create_entity(); // PositionもVelocityも持たないエンティティ

        let pos1_val = Position { x: 10, y: 20 };
        let vel1_val = Velocity { dx: 1, dy: 0 };
        world.add_component(entity1, pos1_val); // Position は Copy なので値はコピーされる
        world.add_component(entity1, vel1_val.clone()); // Velocity は Clone なので .clone() が必要

        // entity1 から Position を取得 (成功するはず)
        let retrieved_pos1 = world.get_component::<Position>(entity1);
        assert!(retrieved_pos1.is_some()); // Option が Some であることを確認
        assert_eq!(retrieved_pos1.unwrap(), &pos1_val); // 中身が正しいか確認 (unwrap は Some であることが確実な場合のみ使う)

        // entity1 から Velocity を取得 (成功するはず)
        let retrieved_vel1 = world.get_component::<Velocity>(entity1);
        assert!(retrieved_vel1.is_some());
        assert_eq!(retrieved_vel1.unwrap(), &vel1_val);

        // entity2 から Position を取得 (失敗するはず -> None)
        let retrieved_pos2 = world.get_component::<Position>(entity2);
        assert!(retrieved_pos2.is_none());

        // 存在しないエンティティから取得 (失敗するはず -> None)
        let non_existent_entity = Entity(999);
        let retrieved_pos_non_existent = world.get_component::<Position>(non_existent_entity);
        assert!(retrieved_pos_non_existent.is_none());

        // 登録されていないコンポーネント型を取得 (失敗するはず -> None)
        // テスト用にダミーの Component を定義
        #[derive(Debug, Clone, Copy, PartialEq, Eq)] struct UnregisteredComponent;
        impl Component for UnregisteredComponent {}
        let retrieved_unregistered = world.get_component::<UnregisteredComponent>(entity1);
        assert!(retrieved_unregistered.is_none());
    }

    #[test]
    fn test_get_component_mut() {
        let mut world = World::new();
        world.register_component::<Position>();

        let entity1 = world.create_entity();
        let pos1_initial = Position { x: 10, y: 20 };
        world.add_component(entity1, pos1_initial);

        // entity1 から Position の可変参照を取得
        let retrieved_pos1_mut = world.get_component_mut::<Position>(entity1);
        assert!(retrieved_pos1_mut.is_some());

        // 取得した可変参照を使って値を変更！
        if let Some(pos_mut) = retrieved_pos1_mut {
            pos_mut.x += 5;
            pos_mut.y = 0;
        }

        // 再度、読み取り専用で取得して、値が変更されたか確認
        let retrieved_pos1_after = world.get_component::<Position>(entity1);
        assert!(retrieved_pos1_after.is_some());
        assert_eq!(retrieved_pos1_after.unwrap(), &Position { x: 15, y: 0 });

        // コンポーネントを持たないエンティティから可変参照を取得 (None になるはず)
        let entity2 = world.create_entity();
        let retrieved_pos2_mut = world.get_component_mut::<Position>(entity2);
        assert!(retrieved_pos2_mut.is_none());
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let entity1 = world.create_entity();
        let pos1_val = Position { x: 10, y: 20 };
        let vel1_val = Velocity { dx: 1, dy: 0 };
        world.add_component(entity1, pos1_val);
        world.add_component(entity1, vel1_val.clone());

        // Position を削除
        let removed_pos = world.remove_component::<Position>(entity1);
        assert!(removed_pos.is_some()); // 削除成功 -> Some
        assert_eq!(removed_pos.unwrap(), pos1_val); // 削除された値が正しいか

        // 再度 Position を get してみる -> None になるはず
        let retrieved_pos_after_remove = world.get_component::<Position>(entity1);
        assert!(retrieved_pos_after_remove.is_none());

        // Velocity はまだ存在しているはず
        let retrieved_vel_after_remove = world.get_component::<Velocity>(entity1);
        assert!(retrieved_vel_after_remove.is_some());
        assert_eq!(retrieved_vel_after_remove.unwrap(), &vel1_val);

        // 存在しないコンポーネントを削除しようとする -> None
        let removed_pos_again = world.remove_component::<Position>(entity1);
        assert!(removed_pos_again.is_none());

        // 存在しないエンティティから削除しようとする -> None
        let non_existent_entity = Entity(999);
        let removed_pos_non_existent = world.remove_component::<Position>(non_existent_entity);
        assert!(removed_pos_non_existent.is_none());
    }


    #[test]
    fn test_get_all_entities_with_component() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let e1 = world.create_entity(); // Pos, Vel
        let e2 = world.create_entity(); // Pos only
        let e3 = world.create_entity(); // Vel only
        let e4 = world.create_entity(); // No components
        let e5 = world.create_entity(); // Pos, Vel (but will be destroyed)

        world.add_component(e1, Position { x: 0, y: 0 });
        world.add_component(e1, Velocity { dx: 1, dy: 1 });
        world.add_component(e2, Position { x: 1, y: 1 });
        world.add_component(e3, Velocity { dx: 2, dy: 2 });
        world.add_component(e5, Position { x: 5, y: 5 });
        world.add_component(e5, Velocity { dx: 5, dy: 5 });

        // Position を持つエンティティを取得 (e1, e2, e5 のはず)
        let mut pos_entities = world.get_all_entities_with_component::<Position>();
        pos_entities.sort_by_key(|e| e.0); // 順序を保証するためにソート
        assert_eq!(pos_entities, vec![e1, e2, e5]);

        // Velocity を持つエンティティを取得 (e1, e3, e5 のはず)
        let mut vel_entities = world.get_all_entities_with_component::<Velocity>();
        vel_entities.sort_by_key(|e| e.0); // ソート
        assert_eq!(vel_entities, vec![e1, e3, e5]);

        // 登録されていないコンポーネントで検索 -> 空のはず
        #[derive(Debug)] struct Unregistered; impl Component for Unregistered {}
        let unregistered_entities = world.get_all_entities_with_component::<Unregistered>();
        assert!(unregistered_entities.is_empty());

        // e5 を削除してみる
        // TODO: destroy_entity がコンポーネントも削除するようになったら、このテストを有効化する
        // world.destroy_entity(e5);

        // 再度 Position を持つエンティティを取得 (e1, e2 のはず)
        // TODO: 上記 destroy_entity が実装されたら、以下のアサーションを有効化する
        // let mut pos_entities_after_destroy = world.get_all_entities_with_component::<Position>();
        // pos_entities_after_destroy.sort_by_key(|e| e.0);
        // assert_eq!(pos_entities_after_destroy, vec![e1, e2]);
    }

    // TODO: destroy_entity のテストケースを追加する (コンポーネントがちゃんと消えるか)
    #[test]
    #[ignore] // destroy_entity のコンポーネント削除が未実装なので無視
    fn test_destroy_entity_removes_components() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let entity = world.create_entity();
        world.add_component(entity, Position { x: 0, y: 0 });
        world.add_component(entity, Velocity { dx: 1, dy: 1 });

        assert!(world.get_component::<Position>(entity).is_some());
        assert!(world.get_component::<Velocity>(entity).is_some());

        // エンティティを削除
        let destroyed = world.destroy_entity(entity);
        assert!(destroyed); // 削除が成功したか
        assert!(!world.is_entity_alive(entity)); // 生存リストから消えたか

        // 削除後、コンポーネントも取得できなくなっているはず！
        // *** ここが現状の実装では失敗する！ ***
        assert!(world.get_component::<Position>(entity).is_none());
        assert!(world.get_component::<Velocity>(entity).is_none());

        // ストレージ自体からも消えているか確認 (より詳細なチェック)
        // Position ストレージを取得
        let pos_type_id = TypeId::of::<Position>();
        if let Some(storage_any) = world.component_stores.get(&pos_type_id) {
            if let Some(storage) = storage_any.downcast_ref::<HashMap<Entity, Position>>() {
                assert!(!storage.contains_key(&entity)); // entity のキーが存在しないはず
            } else { panic!("Downcast failed"); }
        } else { panic!("Storage not found"); }
        // Velocity ストレージも同様にチェック...
    }
} 