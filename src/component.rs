// src/component.rs

// Rust の Any 型を使うためにインポートするよ。
// これを使うと、具体的な型が分からなくても、型情報を扱えるようになるんだ！
// コンポーネントストレージを管理する時にちょっと役立つテクニックだよ。(後で使うかも？🤔)
use std::any::Any;
// HashMap を使うためにインポート！キーと値のペアを効率的に格納できるデータ構造だよ。
// Entity ID をキーにして、コンポーネントのデータを値として保存するのにピッタリ！👍
use std::collections::HashMap;

// さっき作った Entity 型をこのファイルでも使うからインポートするよ。
use crate::entity::Entity; // `crate::` は、このプロジェクト（クレート）のルートから見たパスって意味だよ。

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
pub trait Component: Send + Sync + 'static {
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

// --- ComponentStorage のテスト ---
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの要素を使う宣言
    use crate::entity::EntityManager; // Entity を作るために EntityManager も使う

    // テストで使うためのダミーコンポーネントを定義するよ！
    // 位置情報を表す Position コンポーネント
    #[derive(Debug, PartialEq, Clone)] // テストで比較したりクローンしたりできるようにする
    struct Position {
        x: f32,
        y: f32,
    }
    // Component トレイトを実装！ これで Position はコンポーネントとして認められる！🎉
    impl Component for Position {}

    // テストで使うためのダミーコンポーネント その２！
    // 速度情報を表す Velocity コンポーネント
    #[derive(Debug, PartialEq, Clone)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }
    // Component トレイトを実装！
    impl Component for Velocity {}

    #[test]
    fn insert_and_get_component() {
        // EntityManager と Position 用のストレージを作る
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<Position>::default(); // 型を指定するのを忘れずに！

        // エンティティをいくつか作る
        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();

        // コンポーネントのデータを作る
        let pos1 = Position { x: 10.0, y: 20.0 };
        let pos2 = Position { x: 30.0, y: 40.0 };

        // ストレージにコンポーネントを追加！
        storage.insert(entity1, pos1.clone()); // clone() でコピーして渡す
        storage.insert(entity2, pos2.clone());

        // ちゃんと取得できるか確認！
        // assert_eq! で中身が期待通りか比較するよ
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
        let mut storage = ComponentStorage::<Position>::default();
        let entity1 = manager.create_entity();
        let pos1 = Position { x: 10.0, y: 20.0 };
        storage.insert(entity1, pos1);

        // get_mut で可変参照を取得して、中身を変更してみる！✏️
        if let Some(pos_mut) = storage.get_mut(entity1) {
            pos_mut.x = 15.0; // x 座標を変更！
        } else {
            // ここに来たらテスト失敗！
            panic!("get_mut でコンポーネントを取得できなかった！😭");
        }

        // 変更が反映されてるか確認！
        assert_eq!(storage.get(entity1), Some(&Position { x: 15.0, y: 20.0 }), "コンポーネントの変更が反映されてない！🤔");

        println!("コンポーネントの変更テスト、成功！🎉");
    }

    #[test]
    fn remove_component() {
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<Position>::default();
        let entity1 = manager.create_entity();
        let pos1 = Position { x: 10.0, y: 20.0 };
        storage.insert(entity1, pos1.clone());

        // ちゃんと入ってることを確認
        assert!(storage.get(entity1).is_some(), "削除前にコンポーネントが存在しない！🥺");

        // コンポーネントを削除！🗑️
        let removed_component = storage.remove(entity1);

        // 削除されたコンポーネントが正しいか確認
        assert_eq!(removed_component, Some(pos1), "削除されたコンポーネントが違う！🤔");

        // 削除後に get したら None になるか確認
        assert!(storage.get(entity1).is_none(), "削除したはずのコンポーネントがまだ残ってる！😱");

        println!("コンポーネントの削除テスト、成功！🎉");
    }

     #[test]
    fn iter_components() {
        let manager = EntityManager::default();
        let mut storage = ComponentStorage::<Position>::default();

        let entity1 = manager.create_entity();
        let pos1 = Position { x: 1.0, y: 2.0 };
        storage.insert(entity1, pos1.clone());

        let entity2 = manager.create_entity();
        let pos2 = Position { x: 3.0, y: 4.0 };
        storage.insert(entity2, pos2.clone());

        // iter() を使ってループ処理！
        let mut count = 0;
        for (entity, pos) in storage.iter() {
            // ちゃんとエンティティとコンポーネントのペアが取れるか確認
            if *entity == entity1 {
                assert_eq!(pos, &pos1);
            } else if *entity == entity2 {
                assert_eq!(pos, &pos2);
            } else {
                panic!("知らないエンティティが出てきた！🤯");
            }
            count += 1;
        }
        // ちゃんと2つの要素が処理されたか確認
        assert_eq!(count, 2, "イテレーターの要素数が違う！🤔");

        println!("コンポーネントのイテレーターテスト、成功！🎉");
    }

    #[test]
    fn different_component_types() {
        // 違う種類のコンポーネントストレージもちゃんと動くか確認！
        let manager = EntityManager::default();
        let mut pos_storage = ComponentStorage::<Position>::default();
        let mut vel_storage = ComponentStorage::<Velocity>::default(); // Velocity 用のストレージ！

        let entity1 = manager.create_entity();
        let pos1 = Position { x: 1.0, y: 1.0 };
        let vel1 = Velocity { dx: 0.1, dy: 0.0 };

        pos_storage.insert(entity1, pos1.clone());
        vel_storage.insert(entity1, vel1.clone());

        let entity2 = manager.create_entity();
        let pos2 = Position { x: 5.0, y: 5.0 };
        // entity2 には Velocity は追加しない

        pos_storage.insert(entity2, pos2.clone());

        // それぞれのストレージから正しく取得できるか？
        assert_eq!(pos_storage.get(entity1), Some(&pos1));
        assert_eq!(vel_storage.get(entity1), Some(&vel1));
        assert_eq!(pos_storage.get(entity2), Some(&pos2));
        assert_eq!(vel_storage.get(entity2), None); // entity2 に Velocity はないはず！

        println!("複数コンポーネントタイプのテスト、成功！🎉");
    }
} 