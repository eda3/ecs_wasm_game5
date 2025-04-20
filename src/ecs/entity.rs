// src/entity.rs

// まずは serde っていうライブラリを使う宣言だよ！
// これはデータをJSON形式とかに変換するのに使うんだけど、
// EntityのIDをネットワークで送受信したり、セーブデータにしたりする時に役立つかも！
use serde::{Serialize, Deserialize};
// std::sync::atomic は、複数の処理が同時にIDを使おうとしても安全に番号を増やせるようにするためのものだよ！
// AtomicUsize は、一度に一つの処理しかアクセスできない特殊な整数型。これでIDがかぶらないようにするんだ！
// Ordering::Relaxed は、ちょっと難しいけど、パフォーマンスを良くするための設定だよ。今は気にしなくてOK！😉
use std::sync::atomic::{AtomicUsize, Ordering};

/// Entity（エンティティ）とは、ゲームに登場する「モノ」を表すただの識別子（ID）だよ！
/// 例えば、カード1枚1枚、プレイヤー、ゲームボード自体なんかもエンティティになる。
///
/// この ID は、単なる数字（ここでは usize 型、符号なし整数）で、
/// これだけだと意味はないんだけど、後で作る「コンポーネント」と組み合わせることで、
/// 「IDが 5 のエンティティは、スペードのAのカードで、座標 (10, 20) にある」
/// みたいに意味を持たせることができるんだ！便利でしょ？ ✨
///
/// #[derive(...)] っていうのは、Rustが自動的に便利な機能を追加してくれるおまじないみたいなものだよ！
/// - PartialEq, Eq: ID同士が同じかどうか比較できるようにする (`==` とか)
/// - PartialOrd, Ord: IDの大小を比較できるようにする (`<` とか `>`)
/// - Hash: IDを高速に検索できるデータ構造（HashMapとか）で使えるようにする
/// - Clone, Copy: IDを簡単に複製できるようにする
/// - Debug: IDをデバッグ出力 (`println!("{:?}", entity_id);` みたいに) できるようにする
/// - Serialize, Deserialize: serde でJSONなどに変換できるようにする
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Serialize, Deserialize,
)]
pub struct Entity(pub usize); // usize 型の数値を保持するタプル構造体。これがエンティティIDの実体！

/// EntityManager（エンティティマネージャー）は、エンティティIDを作成したり管理したりする役割を持つよ！
/// ゲーム中に新しいエンティティ（例えば新しいカードとか）が必要になったら、
/// このマネージャーに「新しいIDちょうだい！」ってお願いするんだ🙏
///
/// Default トレイトを実装すると、`EntityManager::default()` で簡単に初期化できるようになるよ！
#[derive(Default)]
pub struct EntityManager {
    // next_id は、次に割り振るエンティティIDを保持するよ。
    // AtomicUsize を使ってるのは、マルチスレッド環境（複数の処理が同時に動く状況）でも
    // 安全に一意なIDを発行できるようにするためだよ！💪
    // ゲームが複雑になってきても安心！😌
    next_id: AtomicUsize,
    // TODO: 将来的には、削除されたエンティティIDを再利用する仕組みも入れたいね！♻️
    // 今は単純にIDを増やし続けるだけだから、メモリ効率はちょっと悪いかも？🤔
    // でも、まずはシンプルに作るのが一番！👍
}

impl EntityManager {
    /// 新しい一意なエンティティIDを作成して返すよ！
    ///
    /// # 戻り値
    /// - 新しく作成された `Entity`
    ///
    /// この関数は、内部カウンター `next_id` の値を1増やして、その値を新しいIDとして使うよ。
    /// `fetch_add(1, Ordering::Relaxed)` は、アトミックに（安全に）カウンターを増やして、
    /// 増やす前の値を返すんだ。だから、IDは0から始まる連番になるよ！0, 1, 2, 3... って感じ！🔢
    pub fn create_entity(&self) -> Entity {
        // Relaxed はパフォーマンスのため。今は深く考えなくて大丈夫！😉
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        Entity(id) // 新しいIDを Entity 構造体で包んで返す！
    }

    // TODO: エンティティを削除する機能もあとで追加しよう！🗑️
    // pub fn destroy_entity(&mut self, entity: Entity) {
    //     // 削除されたIDを管理するリストに追加する処理とかを書く予定！
    // }
}

// --- EntityManager のテスト ---
// Rustにはテストを書く機能が組み込まれてるんだ！便利！🧪
// `cargo test` ってコマンドを打つと、この中のコードが実行されるよ。
#[cfg(test)]
mod tests {
    // `super::*` は、このモジュール（tests）の親モジュール（entity.rsの一番上の階層）にある
    // Entity とか EntityManager を使うよ、っていう意味だよ。
    use super::*;

    // `#[test]` って書くと、これがテスト関数だとRustに伝わるよ！
    #[test]
    fn create_entities_gives_unique_ids() {
        // EntityManager を作る
        let manager = EntityManager::default();

        // いくつかエンティティを作ってみる
        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();
        let entity3 = manager.create_entity();

        // IDがちゃんとユニーク（全部違う）になってるかチェック！
        // `assert_ne!` は、2つの値が等しくないことを確認するマクロだよ。
        // もし等しかったら、テストは失敗する！😱
        assert_ne!(entity1, entity2, "エンティティ1と2のIDが同じになっちゃった！😱");
        assert_ne!(entity1, entity3, "エンティティ1と3のIDが同じになっちゃった！😱");
        assert_ne!(entity2, entity3, "エンティティ2と3のIDが同じになっちゃった！😱");

        // IDがちゃんと0から連番になってるかも確認してみよう！
        // `assert_eq!` は、2つの値が等しいことを確認するマクロだよ。
        assert_eq!(entity1.0, 0, "最初のIDは0のはず！🤔");
        assert_eq!(entity2.0, 1, "2番目のIDは1のはず！🤔");
        assert_eq!(entity3.0, 2, "3番目のIDは2のはず！🤔");

        println!("エンティティIDのユニーク性テスト、成功！🎉");
    }
} 