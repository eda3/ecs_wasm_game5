// src/systems/deal_system.rs

// 必要なものをインポート！
use crate::{
    component::Component, // Component トレイト (Card とか Position が実装してるやつ)
    components::{ // ゲーム固有のコンポーネントたち！
        card::{Card, Suit, Rank}, // カード情報
        position::Position,      // 位置情報
        game_state::{GameState, GameStatus}, // ゲーム状態
        // StackInfo と StackType を追加！
        stack::{StackInfo, StackType},
    },
    entity::Entity,   // エンティティID
    system::System,   // System トレイト (このファイルで作る DealSystem が実装する！)
    world::World,     // ECS の中心、World！
};
// rand クレートから、シャッフルに必要なものをインポート！
use rand::seq::SliceRandom; // 配列やベクターのスライスをシャッフルする機能！
use rand::thread_rng;      // OS が提供する安全な乱数生成器を取得する関数！
use rand::Rng; // thread_rng を使うために必要

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

    // --- リファクタリングで抽出された関数群 ---

    /// 52枚のカードデッキを作成し、シャッフルして返す関数。
    fn create_shuffled_deck<R: Rng>(rng: &mut R) -> Vec<(Suit, Rank)> {
        println!("  デッキを作成し、シャッフルします...");
        let suits = [Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade];
        let ranks = [
            Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King,
        ];

        let mut deck: Vec<(Suit, Rank)> = suits
            .iter()
            .flat_map(|&suit| ranks.iter().map(move |&rank| (suit, rank)))
            .collect();

        deck.shuffle(rng);
        println!("  デッキシャッフル完了！🌀 ({}枚)", deck.len());
        deck
    }

    /// シャッフルされたデッキから、カードエンティティと Card コンポーネントを作成する関数。
    /// 作成された Entity のリストを返す。
    fn create_card_entities(world: &mut World, deck: &[(Suit, Rank)]) -> Vec<Entity> {
         println!("  カードエンティティを作成します...");
        world.register_component::<Card>();
        world.register_component::<Position>();
        world.register_component::<StackInfo>();

        let entities: Vec<Entity> = deck
            .iter()
            .map(|(suit, rank)| {
                let entity = world.create_entity();
                let card_component = Card { suit: *suit, rank: *rank, is_face_up: false }; // 最初は全部裏向き
                world.add_component(entity, card_component);
                entity
            })
            .collect();
         println!("  {} 枚のカードエンティティと Card コンポーネントを作成しました！", entities.len());
        entities
    }

    /// カードエンティティリストを受け取り、場札と山札に配る関数。
    /// Position と StackInfo コンポーネントを追加し、必要なら Card を表向きにする。
    fn deal_cards(world: &mut World, card_entities: &[Entity]) {
        println!("  カードを場札と山札に配ります...");
        let mut card_iter = card_entities.iter().copied(); // イテレータをコピーして使う

        // 4.1 場札 (Tableau) に配る
        println!("    場札に配っています...");
        for tableau_index in 0..7u8 {
            for card_index_in_stack in 0..=tableau_index {
                if let Some(entity) = card_iter.next() {
                    Self::deal_to_tableau_stack(world, entity, tableau_index, card_index_in_stack);
                } else {
                    eprintln!("エラー: 場札への配布中にカードが足りなくなりました！ (予期せぬエラー)");
                    return; // ここで処理中断
                }
            }
        }
         println!("    場札への配布完了。");

        // 4.2 残りを山札 (Stock) に置く
         println!("    山札に配っています...");
        let mut stock_count = 0;
        for (stock_position_index, entity) in card_iter.enumerate() {
            Self::deal_to_stock_stack(world, entity, stock_position_index as u8);
            stock_count += 1;
        }
         println!("    山札への配布完了 ({}枚)。", stock_count);
    }

    /// 特定のエンティティを場札の指定位置に配るヘルパー関数。
    fn deal_to_tableau_stack(world: &mut World, entity: Entity, tableau_index: u8, card_index_in_stack: u8) {
        // StackInfo を設定
        let stack_type = StackType::Tableau(tableau_index);
        let stack_info = StackInfo::new(stack_type, card_index_in_stack);
        world.add_component(entity, stack_info);

        // Position を設定 (仮)
        let position = Position {
            x: 100.0 + (tableau_index as f32 * 110.0),
            y: 250.0 + (card_index_in_stack as f32 * 30.0),
        };
        world.add_component(entity, position);

        // 一番上のカードだけ表向きにする
        let is_top_card = card_index_in_stack == tableau_index;
        if is_top_card {
            if let Some(card) = world.get_component_mut::<Card>(entity) {
                card.is_face_up = true;
            }
        }
        // println!("      エンティティ {:?} を場札 {} の {} 番目に配置 (表向き: {})", entity, tableau_index, card_index_in_stack, is_top_card);
    }

    /// 特定のエンティティを山札の指定位置に配るヘルパー関数。
    fn deal_to_stock_stack(world: &mut World, entity: Entity, stock_position_index: u8) {
        // StackInfo を設定
        let stack_info = StackInfo::new(StackType::Stock, stock_position_index);
        world.add_component(entity, stack_info);

        // Position を設定 (仮)
        let position = Position { x: 100.0, y: 100.0 };
        world.add_component(entity, position);
        // Card は裏向きのまま
        // println!("      エンティティ {:?} を山札の {} 番目に配置", entity, stock_position_index);
    }

    /// ゲーム状態を Playing に初期化する関数。
    fn initialize_game_state(world: &mut World) {
         println!("  ゲーム状態を初期化します...");
        let game_state_entity = Entity(0); // GameState 用の固定エンティティID (仮)
        world.register_component::<GameState>();
        // Entity(0) が存在しない場合に備えて作成 (テスト用)
        if !world.entity_exists(game_state_entity) {
            world.create_entity_with_id(game_state_entity);
        }
        world.add_component(game_state_entity, GameState { status: GameStatus::Playing });
        println!("  ゲーム状態を Playing に設定しました！🎮");
    }
}

// System トレイトの実装 (run メソッドがシンプルになった！)
impl System for DealSystem {
    /// カードを配るロジックを実行するよ！
    fn run(&mut self, world: &mut World) {
        // すでにカードを配り終えていたら、何もしないで終了！ (一度だけ実行するため)
        if self.has_dealt {
            return; // すでに実行済みなら何もしない
        }
        println!("DealSystem 実行開始！");

        // 乱数生成器の準備
        let mut rng = thread_rng();

        // ステップ実行
        let deck = Self::create_shuffled_deck(&mut rng);
        let card_entities = Self::create_card_entities(world, &deck);
        Self::deal_cards(world, &card_entities);
        Self::initialize_game_state(world);

        // 実行完了フラグを立てる
        self.has_dealt = true; // 配り終えたフラグを立てる！
        println!("DealSystem 実行完了！✨");
    }
}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // DealSystem やインポートしたものをテストで使う
    use crate::world::World; // テスト用の World を作る
    // StackInfo と StackType をテストで使うためにインポート
    use crate::components::stack::{StackInfo, StackType};
    use crate::component::Component; // Component トレイトも必要かも
    use crate::entity::Entity; // Entity も必要かも
    use crate::components::game_state::{GameState, GameStatus}; // GameState/Status も必要
    use crate::components::card::Card; // Card も必要
    use crate::components::position::Position; // Position も必要

    #[test]
    fn deal_system_distributes_cards_correctly() {
        let mut world = World::new();
        let mut deal_system = DealSystem::new();

        // Entity(0) を先に確保しておく (GameState用)
        world.create_entity_with_id(Entity(0));

        deal_system.run(&mut world);

        // --- 基本チェック (変更なし) --- 
        // ストレージの存在とサイズをチェック
        assert!(world.storage::<Card>().is_some(), "Card storage missing");
        assert!(world.storage::<Position>().is_some(), "Position storage missing");
        assert!(world.storage::<StackInfo>().is_some(), "StackInfo storage missing");

        let card_count = world.storage::<Card>().map_or(0, |s| s.iter().filter(|o| o.is_some()).count());
        let position_count = world.storage::<Position>().map_or(0, |s| s.iter().filter(|o| o.is_some()).count());
        let stack_info_count = world.storage::<StackInfo>().map_or(0, |s| s.iter().filter(|o| o.is_some()).count());

        // GameState エンティティを除いたカード関連コンポーネントの数が52であるべき
        // (Entity(0)にはこれらのコンポーネントは無いはず)
        assert_eq!(card_count, 52, "Card count mismatch");
        assert_eq!(position_count, 52, "Position count mismatch");
        assert_eq!(stack_info_count, 52, "StackInfo count mismatch");

        let game_state = world.get_component::<GameState>(Entity(0)).expect("GameState component missing");
        assert_eq!(game_state.status, GameStatus::Playing, "GameStatus incorrect");
        assert_eq!(deal_system.has_dealt, true, "has_dealt flag incorrect");

        // --- 配布内容のチェック (変更なし) --- 
        let mut tableau_counts = vec![0; 7];
        let mut stock_count = 0;
        let mut tableau_face_up_counts = vec![0; 7];
        let mut card_entity_ids = Vec::new(); // どのエンティティIDが使われたか記録

        // World から全エンティティとそのコンポーネントを取得して集計
        // Entity ID 0 (GameState) 以外をチェック
        for entity_id in 0..world.next_entity_id {
            let entity = Entity(entity_id);
            if entity == Entity(0) { continue; } // Skip GameState entity

            if let Some(stack_info) = world.get_component::<StackInfo>(entity) {
                card_entity_ids.push(entity_id);
                match stack_info.stack_type {
                    StackType::Tableau(index) => {
                        let idx = index as usize;
                        if idx < 7 {
                            tableau_counts[idx] += 1;
                            // 表向きかチェック
                            if let Some(card) = world.get_component::<Card>(entity) {
                                if card.is_face_up {
                                    tableau_face_up_counts[idx] += 1;
                                }
                                // TODO: position_in_stack のチェック
                            } else {
                                panic!("Card component missing for Tableau entity {:?}", entity);
                            }
                        } else {
                            panic!("Invalid Tableau index {} for entity {:?}", index, entity);
                        }
                    }
                    StackType::Stock => {
                        stock_count += 1;
                        // 裏向きかチェック
                        if let Some(card) = world.get_component::<Card>(entity) {
                             assert!(!card.is_face_up, "Stock card {:?} should be face down", entity);
                        } else {
                            panic!("Card component missing for Stock entity {:?}", entity);
                        }
                         // TODO: position_in_stack のチェック
                    }
                    _ => panic!("Unexpected StackType {:?} found for entity {:?}", stack_info.stack_type, entity),
                }
            } else {
                // Entity ID 0 以外のカードエンティティには StackInfo が必須のはず
                // (ただし、world 実装によっては next_entity_id までに空きがある可能性も？)
                // 厳密には、Card コンポーネントを持つ Entity には StackInfo があるべき
                if world.get_component::<Card>(entity).is_some() {
                     panic!("StackInfo not found for card entity {:?}", entity);
                }
            }
        }

        // カードエンティティがちゃんと 52 個存在するか
        assert_eq!(card_entity_ids.len(), 52, "Number of entities with StackInfo");

        // 各場札の枚数を確認 (1, 2, ..., 7枚)
        for i in 0..7 {
            assert_eq!(tableau_counts[i], i + 1, "Tableau {} count", i);
            // 各場札で表向きは1枚だけか確認
            assert_eq!(tableau_face_up_counts[i], 1, "Tableau {} face up count", i);
        }

        // 山札の枚数を確認 (52 - 28 = 24枚)
        assert_eq!(stock_count, 24, "Stock count");

        println!("DealSystem のカード配布テスト、成功！🎉");

        // 2回目の実行防止チェック
        let card_count_before = world.storage::<Card>().map_or(0, |s| s.iter().filter(|o| o.is_some()).count());
        deal_system.run(&mut world); // 2回目実行
        let card_count_after = world.storage::<Card>().map_or(0, |s| s.iter().filter(|o| o.is_some()).count());
        assert_eq!(card_count_before, card_count_after, "Card count should not increase on second run");
    }
} 