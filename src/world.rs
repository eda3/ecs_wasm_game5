// src/world.rs

// Rust の標準ライブラリから、Any と TypeId を使うよ。
// Any: さっき component.rs でも出てきたけど、具体的な型を隠蔽して扱えるようにするやつ。
// TypeId: 型ごとにユニークなIDを取得するためのもの。これで ComponentStorage を型安全に管理する！
use std::any::{Any, TypeId};
// HashMap: キーと値のペアを効率的に格納するデータ構造。ComponentStorage を管理するのに使う！
use std::collections::HashMap;
use std::rc::Rc;

// 自作の entity モジュールから Entity を使う。
use crate::entity::Entity;
// 自作の component モジュールから Component と ComponentStorage を使う。
use crate::component::{Component, ComponentStorage};

/// World（ワールド）は、ゲーム世界の全てのエンティティとコンポーネントを管理する中心的な存在だよ！
/// この World を通して、エンティティの作成、コンポーネントの追加・削除・取得などを行うことになるんだ。
/// まさにゲーム世界の司令塔！ 司令官気分だね！🫡
#[derive(Default)] // `World::default()` で簡単に初期化できるようにする。
pub struct World {
    // エンティティを管理する EntityManager を持つ。
    // entity_manager: EntityManager,

    // ComponentStorage を管理するための HashMap。
    // キー: コンポーネントの型を示す TypeId。これで「どの型のストレージか」を区別する。
    // 値: Box<dyn Any>。これは「どんな型の ComponentStorage でも入れられる魔法の箱」みたいなもの！ ✨
    //     - `Box`: ヒープ領域にデータを格納するためのポインタ。ストレージのサイズがコンパイル時に
    //              決まらなくても大丈夫になる。
    //     - `dyn Any`: 任意の型を格納できる「トレイトオブジェクト」。これで、`ComponentStorage<Position>` や
    //                  `ComponentStorage<Velocity>` など、色々な型のストレージを一つの HashMap に
    //                  まとめて格納できるんだ！すごいテクニックでしょ？😎
    pub(crate) components: HashMap<TypeId, Box<dyn Any>>,
    // next_entity_id を World が直接持つ
    pub(crate) next_entity_id: usize,
    // TODO: 将来的には、削除されたエンティティを追跡する仕組みもここに必要になるかも？🤔
    //       (例えば、エンティティが削除されたら、関連するコンポーネントも全ストレージから削除するとか)
}

impl World {
    /// 新しい空の World を作成するよ。
    pub fn new() -> Self {
        World {
            // entity_manager: EntityManager::default(), // EntityManager は使わない
            components: HashMap::new(),      // ComponentStorage を管理する HashMap を初期化
            next_entity_id: 0, // World が直接持つ ID を初期化
        }
    }

    // --- Entity Management ---

    /// 新しいエンティティを作成するよ。
    ///
    /// EntityManager に処理を委譲（お願い）するだけ！簡単！👍
    pub fn create_entity(&mut self) -> Entity {
        // World が持つ next_entity_id を使う
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        // ... (rest of create_entity as before, e.g., resizing storage implicitly in add_component)
        println!("World: Entity {} created.", entity_id);
        Entity(entity_id)
    }

    // TODO: エンティティを削除するメソッドも後で追加しよう！
    // pub fn destroy_entity(&mut self, entity: Entity) {
    //     // ここで EntityManager に削除を依頼し、
    //     // さらに、全ての ComponentStorage からも該当エンティティのコンポーネントを削除する必要があるね！💪
    //     // self.entity_manager.destroy_entity(entity);
    //     // for storage_any in self.component_storages.values_mut() {
    //     //     // ここで storage_any をダウンキャストして、remove を呼び出す処理が必要！ (ちょっと複雑！)
    //     // }
    // }

    // --- Component Management ---

    /// 特定の型の ComponentStorage を取得するヘルパー関数（内部処理用だよ）。
    ///
    /// この関数はジェネリクス `<T: Component>` を使ってるから、
    /// `get_storage::<Position>()` みたいに呼び出すと、Position 用のストレージを取得できるんだ！
    ///
    /// # 戻り値
    /// - `Some(&ComponentStorage<T>)`: ストレージが存在すれば、その参照を返す。
    /// - `None`: ストレージが存在しなければ、None を返す。
    ///
    /// `private` な関数（`pub` が付いてない）なので、World の外からは直接呼べないよ。
    fn get_storage<T: Component>(&self) -> Option<&ComponentStorage<T>> {
        // 1. 型 T の TypeId を取得する。
        let type_id = TypeId::of::<T>();
        // 2. HashMap から TypeId をキーにして Box<dyn Any> を取得する。
        self.components.get(&type_id)
            // 3. `and_then` で、取得できた場合にのみ次の処理に進む。
            .and_then(|storage_any| {
                // 4. `downcast_ref::<ComponentStorage<T>>()` を試みる！
                //    これが魔法のテクニック！🪄 `dyn Any` で隠蔽された元の型 (ComponentStorage<T>) に
                //    安全に変換（ダウンキャスト）しようとするんだ。
                //    もし型が一致すれば `Some(&ComponentStorage<T>)` が、違えば `None` が返る。
                storage_any.downcast_ref::<ComponentStorage<T>>()
            })
    }

    /// 特定の型の ComponentStorage を取得するヘルパー関数（内部処理用、書き込み可能版）。
    ///
    /// `get_storage` の `&mut` (可変参照) バージョンだよ。これでストレージの中身を変更できる！✏️
    ///
    /// # 戻り値
    /// - `Some(&mut ComponentStorage<T>)`: ストレージが存在すれば、その可変参照を返す。
    /// - `None`: ストレージが存在しなければ、None を返す。
    fn get_storage_mut<T: Component>(&mut self) -> Option<&mut ComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)
            .and_then(|storage_any| {
                // `downcast_mut` を使うところが `get_storage` との違いだよ！
                storage_any.downcast_mut::<ComponentStorage<T>>()
            })
    }

    /// コンポーネント型を World に登録するよ。
    ///
    /// 特定の型のコンポーネント (例: Position) を使う前に、この関数で
    /// 「Position コンポーネントを使います！」と宣言して、対応するストレージを
    /// World に作成しておく必要があるんだ。
    ///
    /// もし既に登録済みの型なら、何もしないよ。
    ///
    /// # 使用例
    /// ```
    /// world.register_component::<Position>();
    /// world.register_component::<Velocity>();
    /// ```
    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        // `entry(type_id)` は、HashMap に type_id が存在するかどうかチェックして、
        // 存在しなければ新しいエントリーを作るための便利なメソッドだよ。
        // `or_insert_with` は、存在しなかった場合にのみ、クロージャ（`|| { ... }` の部分）を実行して、
        // その結果を HashMap に挿入するんだ。
        self.components.entry(type_id).or_insert_with(|| {
            // 新しい ComponentStorage<T> を作る。
            let storage = ComponentStorage::<T>::new();
            // それを Box で包んで、型情報を隠蔽 (dyn Any に変換) して HashMap に格納！
            Box::new(storage)
        });
        // これで、この型の ComponentStorage が確実に World 内に存在するようになる！👍
    }

    /// エンティティにコンポーネントを追加するよ。
    ///
    /// # 引数
    /// - `entity`: コンポーネントを追加したいエンティティ
    /// - `component`: 追加するコンポーネントのデータ (例: `Position { x: 0.0, y: 0.0 }`)
    ///
    /// # パニック！😱
    /// - もし `T` 型のコンポーネントが `register_component::<T>()` で事前に登録されていなかった場合、
    ///   この関数はパニック（プログラムが強制終了）するよ！
    ///   だから、使う前に必ず登録するのを忘れないでね！🙏
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        // まず、書き込み可能なストレージを取得する。
        // `expect` は Option 型に対して使うメソッドで、
        // - Some(value) なら value を返す。
        // - None なら、指定されたエラーメッセージでパニックする。
        // これで、ストレージが存在しない (登録されていない) 場合に、分かりやすいエラーで落ちるようにしてるんだ。
        self.get_storage_mut::<T>()
            .expect("Component type not registered before adding!") // 登録忘れ防止！🚨
            .insert(entity, component); // ストレージが見つかれば、insert を呼び出す！
    }

    /// エンティティからコンポーネントを取得するよ（読み取り専用）。
    ///
    /// # 戻り値
    /// - `Some(&T)`: コンポーネントが見つかった場合。
    /// - `None`: エンティティが存在しないか、コンポーネントを持っていない、
    ///           またはコンポーネント型が登録されていない場合。
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        // 読み取り専用のストレージを取得して…
        self.get_storage::<T>()
            // ストレージが見つかれば、そのストレージの get メソッドを呼ぶ。
            .and_then(|storage| storage.get(entity))
    }

    /// エンティティからコンポーネントを取得するよ（書き込み可能）。
    ///
    /// # 戻り値
    /// - `Some(&mut T)`: コンポーネントが見つかった場合。
    /// - `None`: エンティティが存在しないか、コンポーネントを持っていない、
    ///           またはコンポーネント型が登録されていない場合。
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        // 書き込み可能なストレージを取得して…
        self.get_storage_mut::<T>()
            // ストレージが見つかれば、そのストレージの get_mut メソッドを呼ぶ。
            .and_then(|storage| storage.get_mut(entity))
    }

    /// エンティティからコンポーネントを削除するよ。
    ///
    /// # 戻り値
    /// - `Some(T)`: 削除されたコンポーネントのデータ。
    /// - `None`: エンティティが存在しないか、コンポーネントを持っていない、
    ///           またはコンポーネント型が登録されていない場合。
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        // 書き込み可能なストレージを取得して…
        self.get_storage_mut::<T>()
            // ストレージが見つかれば、そのストレージの remove メソッドを呼ぶ。
            .and_then(|storage| storage.remove(entity))
    }

    /// 指定された型の ComponentStorage への参照を直接取得するよ（読み取り専用）。
    ///
    /// システム (後で作るやつ！) が特定のコンポーネント群をまとめて処理したい場合に便利だよ！
    ///
    /// # 戻り値
    /// - `Some(&ComponentStorage<T>)`: ストレージが見つかった場合。
    /// - `None`: コンポーネント型が登録されていない場合。
    pub fn storage<T: Component>(&self) -> Option<&ComponentStorage<T>> {
        self.get_storage::<T>()
    }

    /// 指定された型の ComponentStorage への可変参照を直接取得するよ（書き込み可能）。
    ///
    /// システムが特定のコンポーネント群をまとめて変更したい場合に使う！
    ///
    /// # 戻り値
    /// - `Some(&mut ComponentStorage<T>)`: ストレージが見つかった場合。
    /// - `None`: コンポーネント型が登録されていない場合。
    pub fn storage_mut<T: Component>(&mut self) -> Option<&mut ComponentStorage<T>> {
        self.get_storage_mut::<T>()
    }

    /// 指定されたエンティティIDが存在するかどうかを確認するよ。
    ///
    /// # 引数
    /// * `entity` - 確認したいエンティティID。
    ///
    /// # 戻り値
    /// * エンティティが作成された範囲内であれば `true`、そうでなければ `false`。
    ///   (削除はまだ考慮していないよ！)
    ///
    /// # 実装について
    /// 今は単純に、エンティティIDが次に割り振られるID (`next_entity_id`) より小さいかで判断してるよ。
    pub fn entity_exists(&self, entity: Entity) -> bool {
        // self.entity_manager.next_entity_id ではなく、
        // World 自身の next_entity_id を参照する！
        entity.0 < self.next_entity_id
    }

    /// 指定されたIDでエンティティを作成（または予約）するよ。
    /// 主にテストや特定のエンティティ（GameState用など）を固定IDで扱うために使う想定。
    ///
    /// # 引数
    /// * `entity` - 作成したいエンティティのID。
    ///
    /// # 注意点
    /// - もし指定された `entity.0` が現在の `next_entity_id` より大きい場合、
    ///   `next_entity_id` が更新され、間のIDがスキップされることになるよ。
    /// - このメソッドはコンポーネントストレージのリサイズは行わないので、
    ///   実際にコンポーネントを追加する際に `add_component` でリサイズされるよ。
    pub fn create_entity_with_id(&mut self, entity: Entity) {
        let id = entity.0;
        // 指定されたIDが現在の次のID以上なら、次のIDを指定IDの次まで進める
        if id >= self.next_entity_id {
            self.next_entity_id = id + 1;
        }
        // TODO: 将来的には、指定IDが既に存在するかどうかのチェックや、
        //       より厳密なエンティティ管理が必要になるかも。
        println!("World: Entity {:?} created/reserved with specific ID.", entity);
    }

    /// 指定された型のコンポーネントを持つ全てのエンティティのリストを取得するよ。
    ///
    /// # 戻り値
    /// - `Vec<Entity>`: 指定された型のコンポーネントを持つ全てのエンティティのリスト。
    /// - `Vec::new()`: 指定された型のコンポーネントを持つエンティティが存在しない場合。
    pub fn get_all_entities_with_component<T: Component + 'static>(&self) -> Vec<Entity> {
        // storage メソッドは ComponentStorage<T> を返す想定
        if let Some(storage) = self.storage::<T>() {
            // ComponentStorage の iter() (またはそれに類するメソッド) を使う
             storage.iter()
                 // ここを修正！ `entity` は `&Entity` だけど、Copy トレイトがあるから `*entity` で値を取得できる！
                 .map(|(entity, _component)| *entity)
                 .collect()
        } else {
            Vec::new()
        }
    }
}


// --- World のテスト ---
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの World とか Component とか Entity を使う

    // テスト用のダミーコンポーネント (component.rs のテストからコピペ！)
    #[derive(Debug, PartialEq, Clone)]
    struct Position { x: f32, y: f32 }
    impl Component for Position {}

    #[derive(Debug, PartialEq, Clone)]
    struct Velocity { dx: f32, dy: f32 }
    impl Component for Velocity {}

    #[test]
    fn create_entity_registers_and_adds_components() {
        // World を作る
        let mut world = World::new();

        // コンポーネント型を登録！ これを忘れるとパニックする！😱
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        // エンティティを作る
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        // コンポーネントを作る
        let pos1 = Position { x: 1.0, y: 2.0 };
        let vel1 = Velocity { dx: 0.1, dy: 0.0 };
        let pos2 = Position { x: 3.0, y: 4.0 };

        // エンティティにコンポーネントを追加！
        world.add_component(entity1, pos1.clone());
        world.add_component(entity1, vel1.clone()); // entity1 には Position と Velocity 両方！
        world.add_component(entity2, pos2.clone()); // entity2 には Position のみ！

        // ちゃんと取得できるか確認！
        assert_eq!(world.get_component::<Position>(entity1), Some(&pos1));
        assert_eq!(world.get_component::<Velocity>(entity1), Some(&vel1));
        assert_eq!(world.get_component::<Position>(entity2), Some(&pos2));
        // entity2 には Velocity を追加してないので None になるはず！
        assert_eq!(world.get_component::<Velocity>(entity2), None);

        // 存在しないエンティティや未登録のコンポーネントはどうなる？
        let entity3 = world.create_entity();
        #[derive(Debug, PartialEq, Clone)] struct Unregistered; impl Component for Unregistered {}
        assert_eq!(world.get_component::<Position>(entity3), None); // entity3 は Position を持たない
        assert_eq!(world.get_component::<Unregistered>(entity1), None); // Unregistered は登録してない

        println!("World でのエンティティ作成、コンポーネント登録・追加・取得テスト、成功！🎉");
    }

    #[test]
    fn get_and_modify_component_mut() {
        let mut world = World::new();
        world.register_component::<Position>();
        let entity = world.create_entity();
        let initial_pos = Position { x: 10.0, y: 10.0 };
        world.add_component(entity, initial_pos.clone());

        // get_component_mut で取得して変更！
        if let Some(pos_mut) = world.get_component_mut::<Position>(entity) {
            pos_mut.x += 5.0;
        } else {
            panic!("get_component_mut で Position を取得できなかった！😭");
        }

        // 変更が反映されたか確認！
        let expected_pos = Position { x: 15.0, y: 10.0 };
        assert_eq!(world.get_component::<Position>(entity), Some(&expected_pos));

        println!("World でのコンポーネント変更テスト、成功！🎉");
    }

    #[test]
    fn remove_component_from_world() {
        let mut world = World::new();
        world.register_component::<Position>();
        let entity = world.create_entity();
        let pos = Position { x: 0.0, y: 0.0 };
        world.add_component(entity, pos.clone());

        // ちゃんと存在することを確認
        assert!(world.get_component::<Position>(entity).is_some());

        // コンポーネントを削除！
        let removed = world.remove_component::<Position>(entity);

        // 削除されたデータが正しいか確認
        assert_eq!(removed, Some(pos));
        // 削除後は取得できないことを確認
        assert!(world.get_component::<Position>(entity).is_none());

        println!("World でのコンポーネント削除テスト、成功！🎉");
    }

    #[test]
    fn access_storage_directly() {
        let mut world = World::new();
        world.register_component::<Position>();

        let entity1 = world.create_entity();
        world.add_component(entity1, Position { x: 1.0, y: 1.0 });
        let entity2 = world.create_entity();
        world.add_component(entity2, Position { x: 2.0, y: 2.0 });

        // 読み取り専用ストレージを取得
        let pos_storage = world.storage::<Position>().expect("Position storage should exist");
        assert_eq!(pos_storage.len(), 2);
        assert!(pos_storage.get(entity1).is_some());
        assert!(pos_storage.get(entity2).is_some());

        // 書き込み可能ストレージを取得して、全要素をループで変更！
        let mut total_x = 0.0;
        if let Some(pos_storage_mut) = world.storage_mut::<Position>() {
            for (_entity, pos) in pos_storage_mut.iter_mut() {
                pos.x *= 10.0; // x座標を10倍に！
                total_x += pos.x;
            }
        } else {
            panic!("Failed to get mutable Position storage");
        }

        // 変更が反映され、合計値が正しいか確認
        assert_eq!(total_x, 30.0); // (1.0*10 + 2.0*10)
        assert_eq!(world.get_component::<Position>(entity1).unwrap().x, 10.0);
        assert_eq!(world.get_component::<Position>(entity2).unwrap().x, 20.0);

        println!("World から ComponentStorage を直接取得するテスト、成功！🎉");
    }

    #[test]
    #[should_panic] // このテストはパニックすることを期待してる！
    fn add_component_panics_if_not_registered() {
        let mut world = World::new();
        // Position を登録せずに add_component を呼ぶ！
        let entity = world.create_entity();
        world.add_component(entity, Position { x: 0.0, y: 0.0 }); // ここでパニックするはず！
    }
} 