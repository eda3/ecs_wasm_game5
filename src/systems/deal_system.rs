// src/systems/deal_system.rs

// 必要なものをインポート！
use crate::{
    component::Component, // Component トレイト (Card とか Position が実装してるやつ)
    components::{ // ゲーム固有のコンポーネントたち！
        card::{Card, Suit, Rank}, // カード情報
        position::Position,      // 位置情報
        game_state::{GameState, GameStatus}, // ゲーム状態
    },
    entity::Entity,   // エンティティID
    system::System,   // System トレイト (このファイルで作る DealSystem が実装する！)
    world::World,     // ECS の中心、World！
};
// rand クレートから、シャッフルに必要なものをインポート！
use rand::seq::SliceRandom; // 配列やベクターのスライスをシャッフルする機能！
use rand::thread_rng;      // OS が提供する安全な乱数生成器を取得する関数！

/// ゲーム開始時にカードを配るシステムだよ！🃏💨
///
/// このシステムは通常、ゲームの初期化時に一度だけ実行される想定だよ。
/// (もしリセット機能とか作るなら、また呼ばれるかも？🤔)
pub struct DealSystem {
    // システムの状態を持つ必要がある場合は、ここにフィールドを追加するよ。
    // 例えば、「カードを配り終えたか」みたいなフラグとか？
    // 今回はシンプルに、状態は持たない構造体にしてみよう！👍
    has_dealt: bool, // カードを配り終えたかどうかを示すフラグ
}

impl DealSystem {
    /// 新しい DealSystem を作るよ。
    pub fn new() -> Self {
        Self { has_dealt: false } // 最初はまだ配っていない
    }
}

// System トレイトを実装！ これで World から run メソッドを呼べるようになる！
impl System for DealSystem {
    /// カードを配るロジックを実行するよ！
    fn run(&mut self, world: &mut World) {
        // すでにカードを配り終えていたら、何もしないで終了！ (一度だけ実行するため)
        if self.has_dealt {
            return;
        }

        println!("DealSystem 実行中... 🃏 カードを配ります！");

        // --- 1. デッキの作成 ---
        // まずは、52枚全てのカードのデータを作るよ！ (スートとランクの組み合わせ)
        let mut deck: Vec<(Suit, Rank)> = Vec::with_capacity(52); // 52要素分のメモリを確保！効率的！
        // for ループを使って、全スートと全ランクの組み合わせを deck に追加！
        for &suit in [Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade].iter() {
            for rank_val in 1..=13 { // 1から13まで (Rank enum の Ace から King に対応)
                // u8 から Rank に変換 (ちょっと強引だけど、今はこれでOK！) 
                // 本来なら、もっと安全な変換方法を考えるべきかも？🤔 TryFrom とか！
                let rank = match rank_val {
                    1 => Rank::Ace,
                    2 => Rank::Two,
                    3 => Rank::Three,
                    4 => Rank::Four,
                    5 => Rank::Five,
                    6 => Rank::Six,
                    7 => Rank::Seven,
                    8 => Rank::Eight,
                    9 => Rank::Nine,
                    10 => Rank::Ten,
                    11 => Rank::Jack,
                    12 => Rank::Queen,
                    13 => Rank::King,
                    _ => unreachable!(), // 1から13以外はありえないはず！
                };
                deck.push((suit, rank)); // (スート, ランク) のタプルを deck に追加！
            }
        }

        // --- 2. デッキのシャッフル ---
        // 作ったデッキをシャッフル！これでランダムな順番になる！🎲
        let mut rng = thread_rng(); // 乱数生成器を取得
        deck.shuffle(&mut rng);   // deck の中身をランダムに並び替え！✨
        println!("  デッキをシャッフルしました！🌀 ({}枚)", deck.len());

        // --- 3. World にカードエンティティを作成＆コンポーネント追加 ---
        // シャッフルされたデッキの順番で、カードエンティティを作っていくよ！
        let card_entities: Vec<Entity> = deck.into_iter() // deck の所有権を奪ってイテレーターに変換
            .map(|(suit, rank)| { // 各 (suit, rank) タプルに対して処理を実行
                // 新しいエンティティを作成
                let entity = world.create_entity();

                // Card コンポーネントを作成 (最初は裏向き)
                let card_component = Card { suit, rank, is_face_up: false };
                // Position コンポーネントを作成 (初期位置は仮で (0,0) にしておく！) 
                // TODO: 後でちゃんと山札の位置とかに設定する！
                let position_component = Position { x: 0.0, y: 0.0 };

                // コンポーネント型を World に登録 (まだ登録されてなければ)
                // 本当はゲーム初期化時に一括で登録する方が効率的かも？🤔
                world.register_component::<Card>();
                world.register_component::<Position>();

                // エンティティにコンポーネントを追加！
                world.add_component(entity, card_component);
                world.add_component(entity, position_component);

                entity // 作成したエンティティIDを返す
            })
            .collect(); // イテレーターの結果を集めて Vec<Entity> にする！

        println!("  {} 枚のカードエンティティを作成し、World に追加しました！", card_entities.len());

        // --- 4. カードを場に配る ---
        // TODO: ここにソリティアのルールに従ってカードを配る処理を書く！
        // - 場札 (Tableau) に配る (1枚目は表、2列目は1枚裏1枚表...)
        // - 残りを山札 (Stock) に置く
        // - 組札 (Foundation) の場所を準備する (エンティティを作るだけかも？)
        // これらは Card や Position コンポーネントの値を更新することで表現するよ！
        // 例: world.get_component_mut::<Position>(card_entity).unwrap().x = ...;
        //     world.get_component_mut::<Card>(card_entity).unwrap().is_face_up = true;
        println!("  TODO: カードを場札と山札に配る処理を実装します！💪");

        // --- 5. ゲーム状態の設定 ---
        // ゲーム開始なので、GameState を Playing に設定する！
        // GameState は通常、特定の1つのエンティティが持つ想定だよ。
        // ここでは仮に entity 0 を GameState 用エンティティとして使ってみよう！
        // (本当は World にリソースとして直接格納する方がモダンなECS設計かも？今回はシンプルに！)
        let game_state_entity = Entity(0); // 仮のID！

        // GameState コンポーネントを登録＆追加
        world.register_component::<GameState>();
        world.add_component(game_state_entity, GameState { status: GameStatus::Playing });
        println!("  ゲーム状態を Playing に設定しました！🎮");

        // --- 処理完了 ---
        self.has_dealt = true; // 配り終えたフラグを立てる！
        println!("DealSystem 実行完了！✨");
    }
}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // DealSystem やインポートしたものをテストで使う
    use crate::world::World; // テスト用の World を作る

    #[test]
    fn deal_system_creates_52_cards_and_sets_state() {
        let mut world = World::new();
        let mut deal_system = DealSystem::new();

        // システムを実行！
        deal_system.run(&mut world);

        // カードが52枚作られたか確認！
        // Card ストレージを取得して長さをチェック！
        let card_storage = world.storage::<Card>().expect("Card storage should exist after dealing");
        assert_eq!(card_storage.len(), 52, "カードが52枚作られていません！😱");

        // Position ストレージも52個あるか確認！
        let pos_storage = world.storage::<Position>().expect("Position storage should exist after dealing");
        assert_eq!(pos_storage.len(), 52, "Position が52個作られていません！😱");

        // GameState が Playing になっているか確認！
        let game_state_entity = Entity(0); // システム内で使った仮のID
        let game_state = world.get_component::<GameState>(game_state_entity)
            .expect("GameState component should exist after dealing");
        assert_eq!(game_state.status, GameStatus::Playing, "ゲーム状態が Playing になっていません！🤔");

        // has_dealt フラグが true になったか確認
        assert_eq!(deal_system.has_dealt, true, "has_dealt フラグが true になっていません！");

        // もう一度実行しても何も起こらない（カードが増えたりしない）ことを確認
        let card_count_before = world.storage::<Card>().unwrap().len();
        deal_system.run(&mut world); // 2回目実行
        let card_count_after = world.storage::<Card>().unwrap().len();
        assert_eq!(card_count_before, card_count_after, "2回目の実行でカード数が増えました！😭");


        println!("DealSystem の基本的なテスト、成功！🎉");
        // TODO: 配られたカードの内容（重複がないかとか）や位置、表裏の状態なども
        //       本格的にテストしたいね！ (今は TODO の部分が多いから、後で！)
    }
} 