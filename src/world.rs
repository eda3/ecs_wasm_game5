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


// === World のユニットテスト ===
// `#[cfg(test)]` は、`cargo test` を実行した時だけコンパイルされるコードブロックを示すよ！
#[cfg(test)]
mod tests {
    // 親モジュール (World の定義がある場所) のアイテムを全部インポート！ `*` はワイルドカードだよ。
    use super::*;
    // テストで使う標準ライブラリもインポート！
    use std::any::TypeId;

    // --- テスト用のダミーコンポーネントを定義 ---

    // 位置情報を表すシンプルなコンポーネント
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] // テストで比較したり表示したりコピーしたりするので必要なトレイトを derive！
    struct Position {
        x: i32,
        y: i32,
    }
    // Position がコンポーネントであることを示すマーカー実装！
    impl Component for Position {}

    // 速度情報を表すシンプルなコンポーネント
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Velocity {
        dx: i32,
        dy: i32,
    }
    // Velocity がコンポーネントであることを示すマーカー実装！
    impl Component for Velocity {}

    // --- テスト関数たち ---
    // 各テスト関数には `#[test]` アトリビュートを付けるよ！

    #[test]
    fn test_new_world_is_empty() {
        let world = World::new();
        assert!(world.entities.is_empty(), "New world should have no entities");
        assert_eq!(world.next_entity_id, 0, "Next entity ID should start at 0");
        assert!(world.component_stores.is_empty(), "New world should have no component stores");
        // assert!(world.free_list.is_empty(), "New world should have an empty free list"); // free_list を使う場合はこれも
        println!("test_new_world_is_empty: PASSED ✅");
    }

    #[test]
    fn test_create_entity() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        assert_eq!(entity1, Entity(0), "First entity ID should be 0");
        assert_eq!(entity2, Entity(1), "Second entity ID should be 1");
        assert_eq!(world.next_entity_id, 2, "Next entity ID should be 2");
        assert_eq!(world.entities.len(), 2, "World should contain 2 entities");
        assert!(world.entities.contains(&entity1), "World should contain entity1");
        assert!(world.entities.contains(&entity2), "World should contain entity2");
        println!("test_create_entity: PASSED ✅");
    }

    #[test]
    fn test_create_entity_with_id() {
        let mut world = World::new();
        let entity5 = Entity(5);
        let entity2 = Entity(2);

        world.create_entity_with_id(entity5);
        assert!(world.is_entity_alive(entity5), "Entity 5 should be alive");
        assert_eq!(world.next_entity_id, 6, "Next ID should be 6 after adding entity 5");
        assert_eq!(world.entities.len(), 1, "World should have 1 entity");

        world.create_entity_with_id(entity2);
        assert!(world.is_entity_alive(entity2), "Entity 2 should be alive");
        assert_eq!(world.next_entity_id, 6, "Next ID should still be 6 after adding entity 2");
        assert_eq!(world.entities.len(), 2, "World should have 2 entities");

        // 通常の create_entity を呼ぶと、next_entity_id から新しい ID が使われる
        let entity6 = world.create_entity();
        assert_eq!(entity6, Entity(6), "Next created entity should have ID 6");
        assert_eq!(world.next_entity_id, 7, "Next ID should become 7");
        assert_eq!(world.entities.len(), 3, "World should have 3 entities");

        println!("test_create_entity_with_id: PASSED ✅");
    }

    #[test]
    fn test_is_entity_alive() {
        let mut world = World::new();
        let entity0 = world.create_entity();
        let entity1 = Entity(1); // まだ作ってない

        assert!(world.is_entity_alive(entity0), "Entity 0 should be alive");
        assert!(!world.is_entity_alive(entity1), "Entity 1 should not be alive yet");

        world.create_entity_with_id(entity1);
        assert!(world.is_entity_alive(entity1), "Entity 1 should be alive now");

        println!("test_is_entity_alive: PASSED ✅");
    }

    #[test]
    fn test_register_and_add_component() {
        let mut world = World::new();
        world.register_component::<Position>(); // Position 型のコンポーネントを使えるように登録！

        let entity1 = world.create_entity();
        let pos1 = Position { x: 10, y: 20 };
        world.add_component(entity1, pos1); // entity1 に Position コンポーネントを追加！

        // ComponentStoreEntry と remover の存在を確認 (内部的なテスト)
        let type_id_pos = TypeId::of::<Position>();
        assert!(world.component_stores.contains_key(&type_id_pos), "Position store should exist");
        let entry = world.component_stores.get(&type_id_pos).unwrap();
        assert!(entry.storage.is::<HashMap<Entity, Position>>(), "Storage should be HashMap<Entity, Position>");
        // entry.remover のテストは難しいので、destroy_entity のテストで間接的に確認する

        // ストレージから直接値を確認 (テスト用の storage メソッドを使う)
        let storage_any = world.storage::<Position>().expect("Position storage should exist");
        let storage_map = storage_any.downcast_ref::<HashMap<Entity, Position>>().expect("Should downcast to HashMap<Entity, Position>");

        assert_eq!(storage_map.len(), 1, "Position storage should have 1 entry");
        assert_eq!(storage_map.get(&entity1), Some(&pos1), "Stored position should match");
        assert_eq!(storage_map.len(), 1, "Storage size should remain 1 BEFORE adding to non-existent");

        // get_component で取得できるか確認
        assert_eq!(world.get_component::<Position>(entity1), Some(&pos1));

        // 存在しないエンティティに追加しようとしても何も起こらないはず
        let non_existent_entity = Entity(99);
        world.add_component(non_existent_entity, Position { x: 0, y: 0 });
        assert_eq!(world.get_component::<Position>(non_existent_entity), None);

        println!("test_register_and_add_component: PASSED ✅");
    }


    #[test]
    #[should_panic] // このテストはパニックすることを期待してる！
    fn test_add_component_unregistered() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        // Position を register せずに add しようとするとパニックするはず！
        world.add_component(entity1, Position { x: 0, y: 0 });
        // ここに到達したらテスト失敗！
    }

    #[test]
    fn test_get_component() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let pos1 = Position { x: 1, y: 2 };
        let vel1 = Velocity { dx: 3, dy: 4 };
        let pos2 = Position { x: 5, y: 6 };

        world.add_component(entity1, pos1);
        world.add_component(entity1, vel1); // 同じエンティティに複数のコンポーネントを追加
        world.add_component(entity2, pos2);

        // 正しく取得できるか
        assert_eq!(world.get_component::<Position>(entity1), Some(&pos1));
        assert_eq!(world.get_component::<Velocity>(entity1), Some(&vel1));
        assert_eq!(world.get_component::<Position>(entity2), Some(&pos2));

        // 持っていないコンポーネントは None
        assert_eq!(world.get_component::<Velocity>(entity2), None);

        // 存在しないエンティティは None
        assert_eq!(world.get_component::<Position>(Entity(99)), None);

        // 登録されていないコンポーネント型は None (パニックしない！)
        #[derive(Debug, Clone, Copy, PartialEq, Eq)] struct UnregisteredComponent;
        impl Component for UnregisteredComponent {}
        assert_eq!(world.get_component::<UnregisteredComponent>(entity1), None);

        println!("test_get_component: PASSED ✅");
    }

    #[test]
    fn test_get_component_mut() {
        let mut world = World::new();
        world.register_component::<Position>();

        let entity1 = world.create_entity();
        let pos1 = Position { x: 1, y: 2 };
        world.add_component(entity1, pos1);

        // 可変参照を取得して値を変更
        { // スコープを作って可変参照の寿命を制限する (Rust警察👮‍♀️対策！)
            let pos_mut = world.get_component_mut::<Position>(entity1).expect("Should get mutable position");
            pos_mut.x += 10;
            pos_mut.y += 20;
        } // ここで pos_mut の可変借用が終わる

        // 変更が反映されているか確認
        let expected_pos = Position { x: 11, y: 22 };
        assert_eq!(world.get_component::<Position>(entity1), Some(&expected_pos));

        // 持っていない、存在しない、登録されていない場合は None
        assert!(world.get_component_mut::<Velocity>(entity1).is_none());
        assert!(world.get_component_mut::<Position>(Entity(99)).is_none());
        #[derive(Debug)] struct Unregistered; impl Component for Unregistered {}
        assert!(world.get_component_mut::<Unregistered>(entity1).is_none());

        println!("test_get_component_mut: PASSED ✅");
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        world.register_component::<Position>();

        let entity1 = world.create_entity();
        let pos1 = Position { x: 1, y: 2 };
        world.add_component(entity1, pos1);

        // 存在するコンポーネントを削除
        let removed = world.remove_component::<Position>(entity1);
        assert_eq!(removed, Some(pos1), "Should return the removed component");
        // 削除後は取得できないはず
        assert_eq!(world.get_component::<Position>(entity1), None);

        // ストレージからも消えているはず (内部的な確認)
        let storage_map = world.storage::<Position>().unwrap().downcast_ref::<HashMap<Entity, Position>>().unwrap();
        assert!(storage_map.get(&entity1).is_none(), "Component should be gone from storage");
        // ストレージ自体は残っている
        assert!(world.storage::<Position>().is_some());


        // 存在しないコンポーネントを削除しようとしても None が返る
        let removed_again = world.remove_component::<Position>(entity1);
        assert_eq!(removed_again, None, "Removing again should return None");

        // 存在しないエンティティから削除しようとしても None
        assert_eq!(world.remove_component::<Position>(Entity(99)), None);

        // 登録されていない型を削除しようとしても None (パニックしない！)
        #[derive(Debug, PartialEq)] struct Unregistered; impl Component for Unregistered {}
        assert_eq!(world.remove_component::<Unregistered>(entity1), None);

        println!("test_remove_component: PASSED ✅");
    }


    #[test]
    fn test_get_all_entities_with_component() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let e1 = world.create_entity(); // Pos, Vel
        let e2 = world.create_entity(); // Pos
        let e3 = world.create_entity(); // Vel
        let e4 = world.create_entity(); // なし
        let e5 = world.create_entity(); // Pos (後で消す)

        world.add_component(e1, Position { x: 0, y: 0 });
        world.add_component(e1, Velocity { dx: 1, dy: 1 });
        world.add_component(e2, Position { x: 1, y: 1 });
        world.add_component(e3, Velocity { dx: 2, dy: 2 });
        world.add_component(e5, Position { x: 0, y: 0 });

        // Position を持つエンティティを取得
        let mut pos_entities = world.get_all_entities_with_component::<Position>();
        pos_entities.sort_by_key(|e| e.0); // 順番を保証するためにソート
        assert_eq!(pos_entities, vec![e1, e2, e5]);

        // Velocity を持つエンティティを取得
        let mut vel_entities = world.get_all_entities_with_component::<Velocity>();
        vel_entities.sort_by_key(|e| e.0);
        assert_eq!(vel_entities, vec![e1, e3]);

        // 登録されていない型は空リスト
        #[derive(Debug)] struct Unregistered; impl Component for Unregistered {}
        let unregistered_entities = world.get_all_entities_with_component::<Unregistered>();
        assert!(unregistered_entities.is_empty());

        // e5 を削除してみる
        world.destroy_entity(e5); // e5 を削除
        let mut pos_entities_after_destroy = world.get_all_entities_with_component::<Position>();
        pos_entities_after_destroy.sort_by_key(|e| e.0);
        assert_eq!(pos_entities_after_destroy, vec![e1, e2], "Destroyed entity e5 should not be included");

        // コンポーネントを削除した場合
        world.remove_component::<Position>(e1);
        let mut pos_entities_after_remove = world.get_all_entities_with_component::<Position>();
        pos_entities_after_remove.sort_by_key(|e| e.0);
        assert_eq!(pos_entities_after_remove, vec![e2], "Entity e1 should not be included after removing Position");

        println!("test_get_all_entities_with_component: PASSED ✅");
    }

    /// これが今回のメインディッシュ！ destroy_entity がちゃんとコンポーネントを消すかテスト！🍽️
    #[test]
    fn test_destroy_entity_removes_components() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Velocity>();

        let entity_to_destroy = world.create_entity(); // ID 0
        let other_entity = world.create_entity();    // ID 1

        // 削除対象のエンティティにコンポーネントを追加
        world.add_component(entity_to_destroy, Position { x: 1, y: 1 });
        world.add_component(entity_to_destroy, Velocity { dx: 1, dy: 1 });

        // 別のエンティティにもコンポーネントを追加 (こっちは消えないはず！)
        world.add_component(other_entity, Position { x: 2, y: 2 });

        // --- いざ、削除！ ---
        let destroyed = world.destroy_entity(entity_to_destroy);
        assert!(destroyed, "destroy_entity should return true for existing entity");

        // --- 検証！ ---
        // 1. エンティティ自体が消えているか？
        assert!(!world.is_entity_alive(entity_to_destroy), "Destroyed entity should not be alive");
        assert!(world.is_entity_alive(other_entity), "Other entity should still be alive");

        // 2. 削除されたエンティティのコンポーネントが消えているか？ (get_component で確認)
        assert!(world.get_component::<Position>(entity_to_destroy).is_none(), "Position for destroyed entity should be None");
        assert!(world.get_component::<Velocity>(entity_to_destroy).is_none(), "Velocity for destroyed entity should be None");

        // 3. 他のエンティティのコンポーネントは残っているか？
        assert!(world.get_component::<Position>(other_entity).is_some(), "Position for other entity should remain");
        assert_eq!(world.get_component::<Position>(other_entity).unwrap(), &Position{ x: 2, y: 2 });

        // 4. 内部ストレージからも消えているか？ (テスト用ヘルパーで確認)
        let pos_storage_map = world.storage::<Position>().unwrap().downcast_ref::<HashMap<Entity, Position>>().unwrap();
        assert!(pos_storage_map.get(&entity_to_destroy).is_none(), "Position should be removed from storage map");
        assert!(pos_storage_map.get(&other_entity).is_some(), "Other entity's position should remain in storage map");
        assert_eq!(pos_storage_map.len(), 1, "Position storage should contain only other_entity's component");

        let vel_storage_map = world.storage::<Velocity>().unwrap().downcast_ref::<HashMap<Entity, Velocity>>().unwrap();
        assert!(vel_storage_map.get(&entity_to_destroy).is_none(), "Velocity should be removed from storage map");
        assert!(vel_storage_map.is_empty(), "Velocity storage should be empty as only destroyed entity had it");

        // 存在しないエンティティを削除しようとしても false が返る
        let destroyed_again = world.destroy_entity(entity_to_destroy);
        assert!(!destroyed_again, "Destroying already destroyed entity should return false");

        let destroyed_non_existent = world.destroy_entity(Entity(99));
        assert!(!destroyed_non_existent, "Destroying non-existent entity should return false");


        println!("test_destroy_entity_removes_components: PASSED! Component removal works! 🎉🧹");
    }

    // TODO: free_list を使うようになったら、そのテストも追加する
    // #[test]
    // fn test_entity_id_reuse() { ... }
} 