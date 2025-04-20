// src/ecs/world.rs

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
use crate::ecs::entity::Entity;
// Component: 全てのコンポーネントが実装すべきマーカートレイト (中身は空でもOK)。ジェネリクスでコンポーネント型を制約するのに使う。
use crate::ecs::component::Component;
use crate::components::stack::{StackInfo, StackType};

/// コンポーネントストレージとその操作をまとめた内部的な構造体だよ！✨
/// これを使うことで、`World` の `component_stores` で型情報を隠蔽しつつも、
/// 型ごとの操作 (特に削除！) を安全に行えるようにするんだ！賢いっしょ？😎
struct ComponentStoreEntry {
    /// 実際のコンポーネントデータ (`HashMap<Entity, T>`) を保持するストレージ。
    /// `Box<dyn Any>` で型情報を隠蔽 (型消去) してるんだ。これにより、
    /// いろんな型の `HashMap<Entity, T>` を一つの `HashMap` (`component_stores`) で
    /// まとめて管理できる！マジ便利！💖
    storage: Box<dyn Any>,

    /// 指定されたエンティティに対応するコンポーネントを `storage` から削除するための関数ポインタ。🧹
    /// `storage` (Box<dyn Any>) と削除対象の `entity` を引数に取るよ。
    /// この関数ポインタがあるおかげで、`destroy_entity` の中で `storage` の具体的な型 (`T`) を
    /// 知らなくても、型ごとに最適化された削除処理を呼び出せるんだ！天才的アイディア！💡
    /// `fn(&mut Box<dyn Any>, Entity)` っていう型は、「`Box<dyn Any>` の可変参照と `Entity` を受け取って、何も返さない関数」って意味だよ！
    remover: fn(&mut Box<dyn Any>, Entity),
    // TODO: 将来的には、コンポーネントのシリアライズ/デシリアライズ関数とか、
    //       他の型ごとの操作関数もここに追加できるかもね！🤔
}

/// ゲーム世界の全てのエンティティとコンポーネントを管理する中心的な構造体 (自作ECSのコア！)。
/// エンティティの生存管理、コンポーネントの型ごとの保存とアクセス機能を提供するよ。
pub struct World {
    /// 現在生存しているエンティティIDのセット。エンティティが存在するかどうかを高速にチェックできる。
    entities: HashSet<Entity>,
    /// 次に生成するエンティティに割り当てるID。エンティティが作成されるたびにインクリメントされる。
    next_entity_id: usize,
    /// コンポーネントの種類 (TypeId) ごとに、その型のコンポーネントデータを格納するストレージと操作をまとめたもの。
    /// `TypeId` をキーとし、`ComponentStoreEntry` を値として持つ HashMap。
    /// これにより、型安全なコンポーネント削除とかが可能になる！✨
    component_stores: HashMap<TypeId, ComponentStoreEntry>,
    // 削除済みエンティティIDを再利用するためのリスト (今は使わないけど、将来的にメモリ効率↑のために使えるかも)
    // free_list: Vec<usize>,
}

impl World {
    /// 新しい空の World を作成するコンストラクタ。
    /// 各フィールドを初期状態 (空の HashSet, ID カウンタ 0, 空の HashMap) に設定する。
    pub fn new() -> Self {
        World {
            entities: HashSet::new(),
            next_entity_id: 0,
            component_stores: HashMap::new(),
            // free_list: Vec::new(),
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

    /// 指定されたエンティティを削除 (破棄) する。 ✨超重要メソッド！✨
    /// このエンティティに紐づけられている全てのコンポーネントも **自動的に削除される** よ！🧹 これでゴミデータが残らない！安心！💖
    ///
    /// # 引数
    /// * `entity` - 削除したいエンティティ。
    ///
    /// # 戻り値
    /// エンティティが存在し、正常に削除された場合は `true`。
    /// エンティティが存在しなかった場合は `false`。
    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        // まず、エンティティが生存リストにいるか確認。いなければ何もせず false を返す。
        if self.entities.remove(&entity) {
            println!("World: Destroying entity with ID {}", entity.0); // 標準出力

            // よっしゃ！エンティティは生存リストから消した！👍
            // 次は、このエンティティにくっついてたコンポーネントたちを全種類お掃除する番だ！🧹💨

            // `component_stores` (型ごとの倉庫&お掃除係のマップ) の中身を全部見て回るよ！
            // `values_mut()` を使うと、各倉庫 (`ComponentStoreEntry`) の中身を書き換えられる可変参照が手に入る！🔥
            for entry in self.component_stores.values_mut() {
                // 各 `ComponentStoreEntry` には、お掃除専用の関数 `remover` が登録されてる！✨
                // この `remover` 関数に、実際のデータ倉庫 (`entry.storage` の可変参照) と
                // 削除したいエンティティ (`entity`) を渡して実行してもらう！🙏
                // これで、`destroy_entity` 関数自体は `storage` の中身の具体的な型を知らなくても、
                // 型ごとに最適化された削除処理を安全に呼び出せるんだ！マジ天才！😎💖
                (entry.remover)(&mut entry.storage, entity);
            }

            // TODO: 将来的には、ここで free_list に entity.0 を追加してID再利用を実装できるかも
            // self.free_list.push(entity.0);

            true // 削除成功！✨
        } else {
            // 指定されたエンティティは元々存在しなかったみたい…🤔
            println!("World: Attempted to destroy non-existent entity with ID {}", entity.0);
            false // 削除失敗 (というか対象がいなかった)
        }
    }

    /// 新しい型のコンポーネントを World に登録する。
    /// これにより、その型のコンポーネントをエンティティに追加できるようになる。
    /// 内部的には、そのコンポーネント型用のストレージ (`HashMap<Entity, T>`) と、
    /// その型のコンポーネントを削除するための **お掃除関数🧹** を初期化して登録する！
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
        println!("World: Registering component type {:?} ({})", type_id, std::any::type_name::<T>()); // 型名もログに出す！

        // 型ごとの削除処理を行うための関数を定義するよ！✨
        // これはジェネリック関数じゃない、具体的な型 `T` のための関数ポインタになる！
        // 引数として `Box<dyn Any>` の可変参照と `Entity` を取る。
        // 関数の中では、`downcast_mut` を使って `Box<dyn Any>` を安全に `HashMap<Entity, T>` に変換して、
        // `remove` メソッドを呼び出す！👍
        let remover_fn: fn(&mut Box<dyn Any>, Entity) = |storage_any, entity| {
            // storage_any (Box<dyn Any>) を HashMap<Entity, T> にダウンキャスト試行！
            if let Some(storage) = storage_any.downcast_mut::<HashMap<Entity, T>>() {
                // 成功したら、HashMap から entity をキーにしてコンポーネントを削除！🧹
                // remove は削除された値 (Some(T)) か None を返すけど、ここでは使わないから捨てる！
                let _removed_component = storage.remove(&entity);
                // println!("Removed component for entity {} from storage {:?}", entity.0, TypeId::of::<T>()); // デバッグ用ログ
            } else {
                // ダウンキャスト失敗！？！？！？！？！？！？！？！？
                // `register_component` で正しい型の remover を登録してるはずだから、
                // ここに来ることは通常ありえないはず…もし来たら、プログラムのどこかがおかしい！😱
                eprintln!(
                    "FATAL ERROR in remover for type {}: Failed to downcast storage for TypeId {:?}. This indicates a critical bug!",
                    std::any::type_name::<T>(),
                    TypeId::of::<T>()
                );
                // ここでパニックしてもいいかも？🤔 でもとりあえずエラーメッセージだけにしとくか…
                // panic!("Critical error: Component storage type mismatch during removal!");
            }
        };

        // 新しい空の HashMap<Entity, T> を作成。これがコンポーネントの実データを保持する場所になる。
        let new_storage: HashMap<Entity, T> = HashMap::new();

        // `ComponentStoreEntry` を作成して、データ倉庫 (Box化されたHashMap) とお掃除関数をセットにする！✨
        let entry = ComponentStoreEntry {
            storage: Box::new(new_storage), // HashMap を Box に入れて Any で型消去！
            remover: remover_fn,           // 型 T 専用のお掃除関数ポインタ！🧹
        };

        // `component_stores` に、この型の `TypeId` をキーとして、作成した `ComponentStoreEntry` を挿入！
        // これで、この型のコンポーネントが使えるようになって、削除もできるようになった！🎉
        if self.component_stores.insert(type_id, entry).is_some() {
            // もし insert が Some を返したら、それは既に同じ TypeId が存在してたってこと！
            // これは普通、初期化ロジックのミス！🙅‍♀️ パニックさせてもいいレベル！
            eprintln!(
                "WARNING: Component type {:?} ({}) was registered more than once! Overwriting previous registration.",
                type_id,
                std::any::type_name::<T>()
            );
            // panic!("Component type registered twice!"); // 厳しくするならパニック！
        }
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
            // println!("World: Attempted to add component to non-existent entity {}", entity.0);
            // 存在しないエンティティへの追加はよくあることなので、ログレベルを下げるかコメントアウト
            return; // 何もせずに関数を抜ける
        }

        let type_id = TypeId::of::<T>();
        // println!("World: Adding component {:?} to entity {}", type_id, entity.0); // デバッグ用ログ

        // 1. `component_stores` から `TypeId` に対応する `ComponentStoreEntry` を可変参照で取得する。
        //    `get_mut` は `Option<&mut ComponentStoreEntry>` を返す。
        if let Some(entry) = self.component_stores.get_mut(&type_id) {
            // 2. `entry.storage` (Box<dyn Any>) から、目的の型 `HashMap<Entity, T>` への可変参照を取得する。
            //    `downcast_mut::<HashMap<Entity, T>>()` を使う。これは `Option<&mut HashMap<Entity, T>>` を返す。
            if let Some(storage) = entry.storage.downcast_mut::<HashMap<Entity, T>>() {
                // 3. ダウンキャスト成功！ ストレージ (HashMap) にエンティティとコンポーネントを挿入する。
                //    `insert` は、もしキーが既に存在していたら古い値 (Some(T)) を返す。
                let _old_component = storage.insert(entity, component);
                // if old_component.is_some() {
                //     println!("World: Replaced existing component {:?} for entity {}", type_id, entity.0);
                // }
            } else {
                // ダウンキャスト失敗。これは register_component で登録した型と違う型で add_component を呼んでるなど、
                // プログラムのロジックエラーの可能性が高い。register_component の実装ミスかも？
                panic!(
                    "World: Component storage downcast failed when adding component for TypeId {:?} ({}). This should not happen!",
                    type_id, std::any::type_name::<T>()
                );
            }
        } else {
            // `component_stores` に `TypeId` が存在しない場合。`register_component<T>()` を呼び忘れている。
            panic!(
                "World: Component type {:?} ({}) not registered! Call register_component::<{}>() first.",
                type_id, std::any::type_name::<T>(), std::any::type_name::<T>()
            );
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
        // ここでチェックしない場合、下の storage.get で結局 None が返るだけなので、なくても動作はする。
        // if !self.is_entity_alive(entity) {
        //     return None;
        // }

        let type_id = TypeId::of::<T>();
        // 1. `component_stores` から `TypeId` に対応する `ComponentStoreEntry` を取得。
        self.component_stores.get(&type_id)
            // 2. `and_then` を使って、`ComponentStoreEntry` があればその中の `storage` (Box<dyn Any>) のダウンキャストを試みる。
            .and_then(|entry| entry.storage.downcast_ref::<HashMap<Entity, T>>())
            // 3. `and_then` をさらに使って、ダウンキャスト成功 (ストレージが得られた) なら `HashMap::get` を試みる。
            .and_then(|storage| storage.get(&entity))
            // これで、途中で失敗 (型が登録されてない、ダウンキャスト失敗、エンティティにコンポーネントがない) したら None が返る！美しい！✨
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
        // 可変参照を返すので、エンティティ生存チェックはここでやった方が安全かも？🤔
        // (死んだエンティティのコンポーネントを書き換えようとするのを防げる)
        if !self.is_entity_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        // 1. `component_stores` から可変参照で `ComponentStoreEntry` を取得。
        self.component_stores.get_mut(&type_id)
            // 2. `and_then` で `entry.storage` のダウンキャスト (可変参照版 `downcast_mut`)。
            .and_then(|entry| entry.storage.downcast_mut::<HashMap<Entity, T>>())
            // 3. `and_then` で `HashMap` から可変参照を取得 (`get_mut`)。
            .and_then(|storage| storage.get_mut(&entity))
            // これも None 安全！👍
    }

    /// 指定されたエンティティから、指定された型のコンポーネントを **削除** する。
    /// 削除されたコンポーネントの値そのものを返すよ！(もし存在すればね！)
    ///
    /// # 型パラメータ
    /// * `T` - 削除するコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 引数
    /// * `entity` - コンポーネントを削除する対象のエンティティ。
    ///
    /// # 戻り値
    /// コンポーネントが存在し、削除された場合は `Some(T)` (削除されたコンポーネントの値)。
    /// コンポーネントが存在しなかった場合 (エンティティが存在しない、型が登録されていない、
    /// エンティティがそのコンポーネントを持っていない場合など) は `None`。
    pub fn remove_component<T: Component + Any + 'static>(&mut self, entity: Entity) -> Option<T> {
        // エンティティ生存チェックは必須ではない (get_mut で None が返るため) が、
        // パフォーマンスのために先にするのもアリ。どっちがいいかな？🤔 うーん、今回はシンプルに省略！
        // if !self.is_entity_alive(entity) {
        //     return None;
        // }

        let type_id = TypeId::of::<T>();
        // 1. `component_stores` から可変参照で `ComponentStoreEntry` を取得。
        self.component_stores.get_mut(&type_id)
            // 2. `and_then` で `entry.storage` を `HashMap<Entity, T>` にダウンキャスト (可変参照)。
            .and_then(|entry| entry.storage.downcast_mut::<HashMap<Entity, T>>())
            // 3. `and_then` で `HashMap` から `remove` を呼び出す！
            //    `remove(&entity)` は `Option<T>` を返す。これがまさに欲しい戻り値！✨
            .and_then(|storage| storage.remove(&entity))
            // これで完了！シンプル！👍
    }

    /// 指定された型のコンポーネントを持つ **全ての生存しているエンティティ** のリストを取得する。
    ///
    /// # 型パラメータ
    /// * `T` - 検索対象のコンポーネントの型。`Component` トレイトと `Any` トレイトを実装し、`'static` ライフタイムを持つ。
    ///
    /// # 戻り値
    /// 指定された型のコンポーネントを持つエンティティの `Vec<Entity>`。
    /// その型のコンポーネントが登録されていない場合や、誰も持っていない場合は空の `Vec` を返す。
    pub fn get_all_entities_with_component<T: Component + Any + 'static>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        // 1. `component_stores` から `ComponentStoreEntry` を取得。
        if let Some(entry) = self.component_stores.get(&type_id) {
            // 2. `entry.storage` を `HashMap<Entity, T>` にダウンキャスト。
            if let Some(storage) = entry.storage.downcast_ref::<HashMap<Entity, T>>() {
                // 3. ダウンキャスト成功！ ストレージ (HashMap) のキー (つまり Entity) を全て取得する。
                //    `keys()` はイテレータ (&Entity のイテレータ) を返す。
                // 4. `copied()` で &Entity から Entity に変換 (Entity は Copy トレイトを実装してるはず)。
                // 5. `filter()` で、生存しているエンティティだけを残す！ (重要！ dead entity を返さないように！)
                // 6. `collect()` でイテレータの結果を `Vec<Entity>` に集める。
                storage.keys().copied().filter(|e| self.is_entity_alive(*e)).collect()
            } else {
                // ダウンキャスト失敗！プログラムのエラー。空の Vec を返す。
                eprintln!(
                    "World: Component storage downcast failed when getting all entities for TypeId {:?} ({}). Returning empty Vec.",
                    type_id, std::any::type_name::<T>()
                );
                Vec::new()
            }
        } else {
            // 型が登録されていない場合。空の Vec を返す。
            // eprintln!("World: Component type {:?} not registered when getting all entities. Returning empty Vec.", type_id); // これはエラーじゃないのでコメントアウト
            Vec::new()
        }
        // .map_or(Vec::new(), |entry| { // map_or を使って書くこともできるけど、ちょっと読みにくい？🤔
        //     entry.storage.downcast_ref::<HashMap<Entity, T>>()
        //         .map_or(Vec::new(), |storage| {
        //             storage.keys().copied().filter(|e| self.is_entity_alive(*e)).collect()
        //         })
        // })
    }

    /// 指定された StackType を持つ最初のエンティティを探す。
    /// StackInfo コンポーネントを持つ全エンティティを検索し、
    /// stack_type が一致するものが見つかったらその Entity を返す。
    ///
    /// # 引数
    /// * `stack_type`: 検索したいスタックの種類 (`StackType`)。
    ///
    /// # 戻り値
    /// * `Some(Entity)`: 指定された `stack_type` を持つ最初のエンティティが見つかった場合。
    /// * `None`: 見つからなかった場合。
    pub fn find_entity_by_stack_type(&self, stack_type: StackType) -> Option<Entity> {
        // 1. StackInfo コンポーネントを持つ全てのエンティティのリストを取得する。
        let entities_with_stack_info = self.get_all_entities_with_component::<StackInfo>();

        // 2. リスト内の各エンティティについてループ処理を行う。
        for entity in entities_with_stack_info {
            // 3. 各エンティティから StackInfo コンポーネントへの参照を取得する。
            //    get_component は Option<&StackInfo> を返す。
            if let Some(stack_info) = self.get_component::<StackInfo>(entity) {
                // 4. StackInfo の stack_type フィールドが、引数で受け取った stack_type と一致するか比較する。
                if stack_info.stack_type == stack_type {
                    // 5. 一致したら、そのエンティティを Some でラップして返し、関数を終了する。
                    println!("World: Found entity {:?} for stack type {:?}", entity, stack_type); // デバッグ用ログ
                    return Some(entity);
                }
            }
            // get_component が None を返した場合や、stack_type が一致しなかった場合は、
            // ループの次のエンティティに進む。
        }

        // 6. ループが最後まで実行されても見つからなかった場合は、None を返す。
        println!("World: No entity found for stack type {:?}", stack_type); // デバッグ用ログ
        None
    }

    // --- 以下、テストコード用のヘルパーメソッド (外部公開はしない想定) ---

    /// 特定の型のコンポーネントストレージ (`HashMap<Entity, T>` が入った `Box<dyn Any>`) への
    /// **読み取り専用** 参照を取得する。（テストやデバッグ用）
    #[allow(dead_code)] // テスト以外で使わないので警告抑制
    pub(crate) fn storage<T: Component + Any + 'static>(&self) -> Option<&dyn Any> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get(&type_id)
            .map(|entry| &*entry.storage) // ComponentStoreEntry から中の Box<dyn Any> をデリファレンスして &dyn Any を返す！
    }

    /// 特定の型のコンポーネントストレージ (`HashMap<Entity, T>` が入った `Box<dyn Any>`) への
    /// **書き込み可能** 参照を取得する。（テストやデバッグ用）
    #[allow(dead_code)] // テスト以外で使わないので警告抑制
    pub(crate) fn storage_mut<T: Component + Any + 'static>(&mut self) -> Option<&mut dyn Any> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get_mut(&type_id)
            .map(|entry| &mut *entry.storage) // ComponentStoreEntry から中の Box<dyn Any> をデリファレンスして &mut dyn Any を返す！
    }

} // impl World の終わり

// テストコードは world_tests.rs に移動
#[cfg(test)]
mod world_tests; 