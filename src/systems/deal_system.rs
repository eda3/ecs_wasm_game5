// src/systems/deal_system.rs

// 必要なモジュールや型をインポートしていくよ！
use crate::{ // クレート（このプロジェクト）のルートから探す
    components::{ // components モジュールから
        card::{Card, Suit, Rank}, // カードのデータ
        position::Position,       // 位置情報
        stack::{StackInfo, StackType}, // スタック情報
        game_state::{GameState, GameStatus}, // ゲーム状態
    },
    entity::Entity,           // Entity 型
    system::System,           // System トレイト
    world::World,             // World
};
use rand::seq::SliceRandom; // 配列のシャッフルに使う！
use rand::thread_rng;       // 乱数生成器を使う！

/// DealSystem（ディールシステム）だよ！
///
/// ゲーム開始時にカードをシャッフルして、
/// 初期配置（山札、場札）にカードエンティティを生成・配置する役割を持つよ！
/// トランプゲームの「カードを配る人」みたいな感じだね！🃏
pub struct DealSystem {
    has_dealt: bool, // すでに配ったかどうかを記録するフラグだよ🚩
}

impl DealSystem {
    /// 新しい DealSystem を作るよ！
    /// 最初はまだ配ってないから `has_dealt` は `false` にしておくよ。
    pub fn new() -> Self {
        Self { has_dealt: false }
    }

    /// カードの山（デッキ）を作成するよ！ 52枚のカードデータを生成する！
    fn create_deck(&self) -> Vec<Card> {
        let mut deck = Vec::with_capacity(52); // 52枚分のメモリを確保しておくと効率的！
        // Suit (マーク) と Rank (数字) の全組み合わせをループで作る！
        for &suit in &[Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade] {
            for rank_value in 1..=13 { // 1 (Ace) から 13 (King) まで
                let rank = match rank_value {
                    1 => Rank::Ace, 2 => Rank::Two, 3 => Rank::Three, 4 => Rank::Four,
                    5 => Rank::Five, 6 => Rank::Six, 7 => Rank::Seven, 8 => Rank::Eight,
                    9 => Rank::Nine, 10 => Rank::Ten, 11 => Rank::Jack,
                    12 => Rank::Queen, 13 => Rank::King,
                    _ => unreachable!(), // 1..=13 以外はありえない！
                };
                // カードを作成してデッキに追加！最初は全部裏向きだよ！
                deck.push(Card { suit, rank, is_face_up: false });
            }
        }
        deck // 完成したデッキを返す！
    }

    /// デッキをシャッフルするよ！ `rand` クレートの力を借りる！<0xF0><0x9F><0xA7><0x84>
    fn shuffle_deck(&self, deck: &mut Vec<Card>) {
        let mut rng = thread_rng(); // 乱数生成器を取得
        deck.shuffle(&mut rng); // デッキをランダムに並び替える！
        println!("デッキをシャッフルしました！🃏");
    }

    /// シャッフルされたデッキからカードを配って、World にエンティティとコンポーネントを作成するよ！
    fn deal_cards(&mut self, world: &mut World, deck: Vec<Card>) {
        println!("カードを配ります...🎁");
        let mut current_card_index = 0; // デッキの何枚目を配るかを示すインデックス

        // --- GameState エンティティを作成・設定 --- (DealSystem が担当するのが自然かな？)
        // ID 0 は GameState 用に予約する想定 (create_entity_with_id を使うべきかも)
        let game_state_entity = Entity(0);
        // GameState::new() ではなく、直接構造体リテラルで作成する！
        world.add_component(game_state_entity, GameState { status: GameStatus::Playing }); // 初期状態は Playing
        println!("  GameState エンティティ ({:?}) を作成し、初期状態を設定しました。", game_state_entity);

        // --- 場札 (Tableau) に配る --- (7列あるよ)
        for i in 0..7 { // i は列のインデックス (0 から 6)
            for j in 0..=i { // j は各列に配るカードの枚数 (1枚目から i+1 枚目まで)
                // デッキからカードを取り出す (インデックスチェックは省略してるけど、本当は必要！)
                let card = deck[current_card_index].clone();
                // 新しいエンティティを作成 (カード1枚 = 1エンティティ)
                // create_entity の戻り値は Option<Entity> だったはず -> World の実装が変わったので Entity を返す
                let entity = world.create_entity();

                // カードコンポーネントを追加
                world.add_component(entity, card);
                // 位置コンポーネントを追加 (座標は仮だよ！後でちゃんと計算する)
                let pos = Position { x: 100.0 + i as f32 * 110.0, y: 250.0 + j as f32 * 30.0 };
                world.add_component(entity, pos);
                // スタック情報コンポーネントを追加
                let stack_info = StackInfo::new(StackType::Tableau(i as u8), j as u8);
                world.add_component(entity, stack_info);

                // 各列の一番上のカード (j == i) だけ表向きにする！
                if j == i {
                    if let Some(c) = world.get_component_mut::<Card>(entity) { // 可変参照を取得して変更！
                        c.is_face_up = true;
                    }
                    println!("  場札 {} の {} 枚目 ({:?}) を表向きで配置しました。", i, j + 1, entity);
                } else {
                    println!("  場札 {} の {} 枚目 ({:?}) を裏向きで配置しました。", i, j + 1, entity);
                }

                current_card_index += 1; // 次のカードへ！
            }
        }

        // --- 残りのカードを山札 (Stock) に配置 --- 
        println!("  残りのカードを山札に配置します...");
        for i in current_card_index..deck.len() {
            let card = deck[i].clone();
            let entity = world.create_entity();
            world.add_component(entity, card);
            // 山札の位置 (仮)
            let pos = Position { x: 100.0, y: 100.0 };
            world.add_component(entity, pos);
            // スタック情報: Stock, 位置は積む順 (0が一番下)
            let stack_info = StackInfo::new(StackType::Stock, (i - current_card_index) as u8);
            world.add_component(entity, stack_info);
            println!("    山札の {} 枚目 ({:?}) を配置しました。", i - current_card_index + 1, entity);
        }

        println!("カードの配布が完了しました！✨");
        self.has_dealt = true; // 配り終えたフラグを立てる！
    }
}

// System トレイトの実装！
// これで World が DealSystem を「システム」として認識できるようになるよ！
impl System for DealSystem {
    /// システムを実行するメソッドだよ！
    /// World の状態を受け取って、必要な処理（ここではカード配布）を行う。
    fn run(&mut self, world: &mut World) {
        // まだカードを配っていなければ...
        if !self.has_dealt {
            println!("DealSystem: 実行します！ (初回実行)");
            // 1. デッキを作る
            let mut deck = self.create_deck();
            // 2. デッキをシャッフルする
            self.shuffle_deck(&mut deck);
            // 3. カードを配る (World にエンティティとコンポーネントを作成する)
            self.deal_cards(world, deck);
        } else {
            // もう配り終わってる場合は何もしない
            // println!("DealSystem: 既に配布済みのためスキップします。");
        }
    }
}

// --- DealSystem のテスト --- 
#[cfg(test)]
mod tests {
    use super::*; // DealSystem, World などをインポート
    use crate::components::card::Card; // テスト確認用
    use crate::components::stack::{StackInfo, StackType}; // テスト確認用

    #[test]
    fn deal_system_deals_cards_correctly() {
        // 1. セットアップ
        let mut world = World::new();
        let mut deal_system = DealSystem::new();

        // 必要なコンポーネントを事前に登録！
        world.register_component::<Card>();
        world.register_component::<Position>();
        world.register_component::<StackInfo>();
        world.register_component::<GameState>(); // GameState も登録！

        // 2. 実行！
        deal_system.run(&mut world);

        // 3. 検証！
        // 正しく 52 枚のカード + 1 つの GameState エンティティが生成されたか？
        // (create_entity は 0 から ID を振るので、next_entity_id が 53 になっているはず)
        assert_eq!(world.next_entity_id, 52 + 1, "エンティティ数が正しくない！"); 

        // GameState が ID 0 に存在するか？
        assert!(world.get_component::<GameState>(Entity(0)).is_some(), "GameStateエンティティが見つからない！");

        // カードコンポーネントを持つエンティティが 52 個あるか？
        let card_entities = world.get_all_entities_with_component::<Card>();
        assert_eq!(card_entities.len(), 52, "カードエンティティ数が52ではない！");

        // 場札の各列の一番上のカードが表向きになっているか？ (例: 列0)
        let tableau0_entities: Vec<_> = card_entities.iter().filter(|&&e| 
            world.get_component::<StackInfo>(e).map_or(false, |si| si.stack_type == StackType::Tableau(0))
        ).collect();
        assert_eq!(tableau0_entities.len(), 1, "場札0のカード数が違う！");
        let top_card_entity_t0 = tableau0_entities[0];
        let top_card_t0 = world.get_component::<Card>(*top_card_entity_t0).unwrap();
        assert!(top_card_t0.is_face_up, "場札0の一番上のカードが裏向き！");
        let top_card_stack_t0 = world.get_component::<StackInfo>(*top_card_entity_t0).unwrap();
        assert_eq!(top_card_stack_t0.position_in_stack, 0, "場札0のカードのスタック位置が違う！");

         // 場札の列6の一番上のカードが表向きになっているか？
        let tableau6_entities: Vec<_> = card_entities.iter().filter(|&&e| 
            world.get_component::<StackInfo>(e).map_or(false, |si| si.stack_type == StackType::Tableau(6))
        ).collect();
        assert_eq!(tableau6_entities.len(), 7, "場札6のカード数が違う！"); // 列6には7枚
        // position_in_stack が最大のものが一番上
        let top_card_entity_t6 = tableau6_entities.iter().max_by_key(|&&e| 
            world.get_component::<StackInfo>(e).unwrap().position_in_stack
        ).unwrap();
        let top_card_t6 = world.get_component::<Card>(*top_card_entity_t6).unwrap();
        assert!(top_card_t6.is_face_up, "場札6の一番上のカードが裏向き！");
        let top_card_stack_t6 = world.get_component::<StackInfo>(*top_card_entity_t6).unwrap();
        assert_eq!(top_card_stack_t6.position_in_stack, 6, "場札6の一番上のカードのスタック位置が違う！");

        // 山札のカードが全て裏向きか？
        let stock_cards: Vec<_> = card_entities.iter().filter(|&&e| 
            world.get_component::<StackInfo>(e).map_or(false, |si| si.stack_type == StackType::Stock)
        ).collect();
        // 52 - (1+2+3+4+5+6+7) = 52 - 28 = 24枚
        assert_eq!(stock_cards.len(), 24, "山札のカード数が違う！"); 
        for entity in stock_cards {
            let card = world.get_component::<Card>(*entity).unwrap();
            assert!(!card.is_face_up, "山札のカード {:?} が表向き！", entity);
        }

        // DealSystem が再度実行されてもカードが増えないか？
        let entity_count_before = world.next_entity_id;
        deal_system.run(&mut world);
        let entity_count_after = world.next_entity_id;
        assert_eq!(entity_count_before, entity_count_after, "DealSystem が2回実行されてエンティティが増えた！");

        println!("DealSystem のカード配布テスト、成功！🎉");
    }
} 