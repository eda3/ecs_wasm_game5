use crate::components::{ // components はそのまま
    card::{Card, Suit, Rank},
    position::Position,
    game_state::{GameState, GameStatus},
    stack::{StackInfo, StackType}
};
use crate::ecs::{ // ★修正: crate:: を crate::ecs:: に変更！
    entity::Entity,
    system::System,
    world::World,
};
// use crate::components::dragging_info::DraggingInfo; // 未使用
// use crate::logic::rules; // 未使用 (check_move_validity 内のロジックで直接使われる想定？)
// use crate::log; // 未使用

// --- StackType Enum (移動元・移動先の種類を示す) ---
// TODO: この enum をどこか適切な場所 (e.g., components/mod.rs や components/stack.rs?) に定義する
//       必要に応じて、場札の列番号や組札のスートなどの情報も持たせる
// ↓↓↓ この enum 定義はもう components/stack.rs にあるから不要！削除！
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum StackType {
//     Tableau(u8), // 場札 (列番号 0-6)
//     Foundation(Suit), // 組札 (スート別)
//     Stock,       // 山札
//     Waste,       // (クロンダイクの場合) 山札からめくったカード置き場
// }
// ↑↑↑ ここまで削除！

// --- CardColor enum はここに移動済み --- 
#[derive(PartialEq, Eq)]
enum CardColor { Red, Black }

/// カード移動のロジックを実行するシステムだよ！🖱️💨
///
/// プレイヤーからの入力（「どのカードをどこに動かすか」）を受け取って、
/// それがソリティアのルール上可能かチェックし、可能なら World の状態を更新するよ。
pub struct MoveCardSystem {
    // 今回は状態を持たないシステムとする
}

impl MoveCardSystem {
    /// 新しい MoveCardSystem を作るよ。
    pub fn new() -> Self {
        Self {}
    }

    // --- run メソッドから切り出されたヘルパー関数 ---

    /// カード移動リクエストを処理する本体
    fn process_move_request(&mut self, world: &mut World, moved_entity: Entity, target_entity: Entity) {
        println!("MoveCardSystem: カード移動リクエストを処理します: {:?} -> {:?}", moved_entity, target_entity);

        // --- 2. 必要なコンポーネントの取得 ---
        let moved_card_opt = world.get_component::<Card>(moved_entity);
        let target_card_opt = world.get_component::<Card>(target_entity);
        let target_pos_opt = world.get_component::<Position>(target_entity);
        let source_stack_info_opt = world.get_component::<StackInfo>(moved_entity);
        let target_stack_info_opt = world.get_component::<StackInfo>(target_entity);

        // --- 3. ルールチェック＆状態更新 ---
        if let (Some(moved_card), Some(source_stack_info)) = (moved_card_opt.cloned(), source_stack_info_opt.cloned()) {
            let target_type = target_stack_info_opt.map(|info| info.stack_type).or_else(|| {
                println!("WARN: Target entity {:?} has no StackInfo, assuming Foundation(0)!", target_entity);
                Some(StackType::Foundation(0))
            });

            if let Some(target_type) = target_type {
                let foundation_top_card = self.get_foundation_top_card(world, target_type);

                if self.check_move_validity(&moved_card, target_card_opt, source_stack_info.stack_type, target_type, foundation_top_card) {
                    self.apply_move(world, moved_entity, target_entity, target_pos_opt.cloned(), target_type);
                } else {
                    println!("  ルール違反！移動できませんでした。🙅‍♀️");
                }
            } else {
                eprintln!("MoveCardSystem: 移動先の種類を特定できませんでした。");
            }
        } else {
            eprintln!("MoveCardSystem: 移動元のカード {:?} または StackInfo が見つかりません！", moved_entity);
        }
    }

    /// 指定された Foundation スタックの一番上のカードを取得する（仮実装）
    /// TODO: 正しい実装には、World から特定の Foundation のカードを効率的に見つける方法が必要
    fn get_foundation_top_card<'a>(&self, world: &'a World, target_type: StackType) -> Option<&'a Card> {
        if let StackType::Foundation(index) = target_type {
            world.get_all_entities_with_component::<Card>()
                .iter()
                .filter_map(|&entity| {
                    world.get_component::<StackInfo>(entity)
                         .filter(|info| info.stack_type == StackType::Foundation(index))
                         .map(|info| (entity, info.position_in_stack))
                })
                .max_by_key(|&(_, pos)| pos)
                .and_then(|(entity, _)| world.get_component::<Card>(entity))
        } else {
            None
        }
    }

    /// 移動がルール上可能かチェックする関数
    fn check_move_validity(
        &self,
        moved_card: &Card,
        target_card_opt: Option<&Card>,
        source_type: StackType,
        target_type: StackType,
        foundation_top_card: Option<&Card>,
    ) -> bool {
        println!("  ルールチェック実行: {:?} ({:?}) -> {:?}", moved_card.rank, source_type, target_type);
        match (source_type, target_type) {
            (StackType::Tableau(_), StackType::Tableau(_)) => {
                if let Some(target_card) = target_card_opt {
                    self.can_move_tableau_to_tableau(moved_card, target_card)
                } else {
                    self.can_move_tableau_to_empty_tableau(moved_card)
                }
            }
            (StackType::Tableau(_), StackType::Foundation(target_index)) => {
                let target_suit = match target_index {
                    0 => Some(Suit::Heart),
                    1 => Some(Suit::Diamond),
                    2 => Some(Suit::Club),
                    3 => Some(Suit::Spade),
                    _ => None,
                };
                if target_suit != Some(moved_card.suit) {
                    println!("    組札への移動失敗: スート不一致 ({:?} vs {:?})", moved_card.suit, target_suit);
                    return false;
                }
                self.can_move_to_foundation(moved_card, foundation_top_card)
            }
            (StackType::Waste, StackType::Tableau(_)) => {
                 if let Some(target_card) = target_card_opt {
                    self.can_move_stock_to_tableau(moved_card, target_card)
                } else {
                    self.can_move_stock_to_empty_tableau(moved_card)
                }
            }
             (StackType::Waste, StackType::Foundation(target_index)) => {
                let target_suit = match target_index {
                    0 => Some(Suit::Heart), 1 => Some(Suit::Diamond),
                    2 => Some(Suit::Club), 3 => Some(Suit::Spade),
                    _ => None,
                };
                if target_suit != Some(moved_card.suit) {
                    println!("    組札への移動失敗: スート不一致 ({:?} vs {:?})", moved_card.suit, target_suit);
                    return false;
                }
                self.can_move_stock_to_foundation(moved_card, foundation_top_card)
            }
            _ => {
                println!("  未対応または不正な移動パターンです: {:?} -> {:?}", source_type, target_type);
                false
            }
        }
    }

    /// 実際に World の状態を更新する関数
    fn apply_move(
        &self,
        world: &mut World,
        moved_entity: Entity,
        target_entity: Entity,
        target_pos_opt: Option<Position>,
        target_type: StackType,
    ) {
        println!("  カード {:?} を {:?} ({:?}) へ移動します！", moved_entity, target_entity, target_type);

        let old_stack_info = world.get_component::<StackInfo>(moved_entity).cloned();

        let max_pos_in_target_stack = world
            .get_all_entities_with_component::<StackInfo>()
            .iter()
            .filter_map(|&entity| {
                if entity == moved_entity { return None; }
                world.get_component::<StackInfo>(entity)
                    .filter(|info| info.stack_type == target_type)
                    .map(|info| info.position_in_stack)
            })
            .max();
        let new_position_in_stack = max_pos_in_target_stack.map_or(0, |max| max + 1);
        println!("    移動先の最大 position_in_stack: {:?}, 新しい position: {}", max_pos_in_target_stack, new_position_in_stack);

        // 1. 移動するカードの Position コンポーネントを更新
        if let Some(target_pos) = target_pos_opt {
            if let Some(moved_pos_mut) = world.get_component_mut::<Position>(moved_entity) {
                let y_offset = 0.0;
                moved_pos_mut.x = target_pos.x;
                moved_pos_mut.y = target_pos.y + y_offset;
                println!("    {:?} の位置を ({}, {}) に更新しました。", moved_entity, moved_pos_mut.x, moved_pos_mut.y);
            }
        } else {
            if let Some(moved_pos_mut) = world.get_component_mut::<Position>(moved_entity) {
                if let StackType::Foundation(index) = target_type {
                    moved_pos_mut.x = 500.0 + (index as f32 * 110.0);
                    moved_pos_mut.y = 100.0;
                    println!("    {:?} の位置を Foundation {} ({}, {}) に更新しました。", moved_entity, index, moved_pos_mut.x, moved_pos_mut.y);
                } else {
                    eprintln!("MoveCardSystem: 移動先の Position が見つかりません (非 Foundation)！");
                }
            }
        }

        // 2. 移動するカードの StackInfo コンポーネントを更新
        if let Some(stack_info) = world.get_component_mut::<StackInfo>(moved_entity) {
             stack_info.stack_type = target_type;
             stack_info.position_in_stack = new_position_in_stack;

             if let Some(ref old_info) = old_stack_info { 
                println!("    {:?} の StackInfo を {:?} (元: {:?}) に更新しました。", moved_entity, stack_info, old_info);
             } else {
                 println!("    {:?} の StackInfo を {:?} (元情報なし) に更新しました。", moved_entity, stack_info);
             }
        } else {
            eprintln!("MoveCardSystem: 移動元 {:?} の StackInfo が見つかりません！ 更新できませんでした。", moved_entity);
            // StackInfo が更新できない場合でも、is_face_up 処理は試みる
        }

        // --- ここから is_face_up の更新処理 ---
        if let Some(old_info) = old_stack_info { 
            if let StackType::Tableau(_) = old_info.stack_type { 
                if old_info.position_in_stack > 0 { 
                    let position_to_reveal = old_info.position_in_stack - 1;
                    println!("    移動元 ({:?}) の位置 {} にあったカードを表にするかチェックします...", old_info.stack_type, position_to_reveal);

                    let entity_to_reveal: Option<Entity> = world
                        .get_all_entities_with_component::<StackInfo>()
                        .iter()
                        .find_map(|&entity| { 
                            if entity == moved_entity { return None; } 
                            if world.get_component::<StackInfo>(entity)
                                .map_or(false, |info| {
                                    info.stack_type == old_info.stack_type &&
                                    info.position_in_stack == position_to_reveal
                                })
                            {
                                Some(entity) 
                            } else {
                                None 
                            }
                        });

                    if let Some(found_entity) = entity_to_reveal { 
                        println!("      -> 位置 {} にエンティティ {:?} を発見！", position_to_reveal, found_entity);
                        if let Some(card_to_reveal) = world.get_component_mut::<Card>(found_entity) {
                            if !card_to_reveal.is_face_up {
                                println!("        -> カード {:?} を表向きにします！", card_to_reveal);
                                card_to_reveal.is_face_up = true;
                            } else {
                                println!("        -> カードは既に表向きでした。");
                            }
                        } else {
                             println!("      -> WARN: エンティティ {:?} に Card コンポーネントがありません！", found_entity);
                        }
                    } else {
                         println!("      -> 位置 {} にエンティティが見つかりませんでした。", position_to_reveal);
                    }
                } else {
                     println!("    移動したカードは場札の一番下だったので、表にするカードはありません。");
                }
            }
        } else {
             println!("    WARN: 移動元の StackInfo が取得できなかったため、カードを表にする処理をスキップします。");
        }
        // --- is_face_up の更新処理 ここまで ---


        println!("  状態更新完了！");
    }

    // --- ルールチェックのヘルパー関数群 ---

    /// 場札 (Tableau) から場札への移動が可能かチェックする関数
    fn can_move_tableau_to_tableau(&self, moved_card: &Card, target_card: &Card) -> bool {
        if !target_card.is_face_up { return false; }
        if moved_card.rank as usize != target_card.rank as usize - 1 { return false; }
        let moved_color = Self::get_suit_color(moved_card.suit);
        let target_color = Self::get_suit_color(target_card.suit);
        if moved_color == target_color { return false; }
        true
    }

    /// 場札 (Tableau) から空の場札列へ移動が可能かチェックする関数 (キングのみ)
    fn can_move_tableau_to_empty_tableau(&self, moved_card: &Card) -> bool {
        moved_card.rank == Rank::King
    }

    /// 場札 (Tableau) から組札 (Foundation) へ移動が可能かチェックする関数
    fn can_move_to_foundation(&self, moved_card: &Card, foundation_top_card: Option<&Card>) -> bool {
        match foundation_top_card {
            None => moved_card.rank == Rank::Ace,
            Some(top_card) => {
                moved_card.suit == top_card.suit &&
                moved_card.rank as usize == top_card.rank as usize + 1
            }
        }
    }

    /// 山札 (Stock) から場札 (Tableau) へ移動が可能かチェックする関数
    fn can_move_stock_to_tableau(&self, moved_card: &Card, target_card: &Card) -> bool {
        self.can_move_tableau_to_tableau(moved_card, target_card)
    }

    /// 山札 (Stock) から空の場札列へ移動が可能かチェックする関数
    fn can_move_stock_to_empty_tableau(&self, moved_card: &Card) -> bool {
        self.can_move_tableau_to_empty_tableau(moved_card)
    }

    /// 山札 (Stock) から組札 (Foundation) へ移動が可能かチェックする関数
    fn can_move_stock_to_foundation(&self, moved_card: &Card, foundation_top_card: Option<&Card>) -> bool {
        self.can_move_to_foundation(moved_card, foundation_top_card)
    }

    // スートの色を取得するヘルパー関数
    fn get_suit_color(suit: Suit) -> CardColor {
        match suit {
            Suit::Heart | Suit::Diamond => CardColor::Red,
            Suit::Club | Suit::Spade => CardColor::Black,
        }
    }
}

impl System for MoveCardSystem {
    /// カード移動のロジックを実行するよ！(リファクタリング後)
    fn run(&mut self, world: &mut World) {
        // --- 0. ゲーム状態の確認 ---
        let game_state_entity = Entity(0); // 仮のID
        let is_playing = world.get_component::<GameState>(game_state_entity)
            .map_or(false, |gs| gs.status == GameStatus::Playing);

        if !is_playing {
            return; // ゲーム中でなければ何もしない
        }

        // --- 1. 移動リクエストの取得 ---
        // TODO: プレイヤーからの入力を受け取る (別のシステムやイベントキューから)
        let maybe_move_request: Option<(Entity, Entity)> = Some((Entity(1), Entity(3))); // 仮！要修正！

        // --- 2. リクエスト処理 ---
        if let Some((moved_entity, target_entity)) = maybe_move_request {
            // 切り出した関数を呼び出す！
            self.process_move_request(world, moved_entity, target_entity);
        }
        // リクエストがなければ run メソッドはここで終了
    }
}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; 
    use crate::world::World;
    use crate::components::card::{Suit, Rank, Card};
    use crate::components::position::Position;
    use crate::components::game_state::{GameState, GameStatus};
    use crate::entity::Entity;

    // ルールチェック関数の単体テストはそのまま使える！
    #[test] fn test_can_move_tableau_to_tableau() { 
        let system = MoveCardSystem::new();
        let queen_red = Card { suit: Suit::Heart, rank: Rank::Queen, is_face_up: true };
        let jack_black = Card { suit: Suit::Spade, rank: Rank::Jack, is_face_up: true };
        assert!(system.can_move_tableau_to_tableau(&jack_black, &queen_red));
        assert!(!system.can_move_tableau_to_tableau(&queen_red, &jack_black));
        let jack_red = Card { suit: Suit::Diamond, rank: Rank::Jack, is_face_up: true };
        assert!(!system.can_move_tableau_to_tableau(&jack_red, &queen_red));
        let queen_red_facedown = Card { suit: Suit::Heart, rank: Rank::Queen, is_face_up: false };
        assert!(!system.can_move_tableau_to_tableau(&jack_black, &queen_red_facedown));
        println!("場札->場札ルールチェックテスト、成功！🎉");
     }
    #[test] fn test_can_move_to_foundation() { 
        let system = MoveCardSystem::new();
        let ace_heart = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_heart = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let three_heart = Card { suit: Suit::Heart, rank: Rank::Three, is_face_up: true };
        let ace_spade = Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true };
        assert!(system.can_move_to_foundation(&ace_heart, None));
        assert!(!system.can_move_to_foundation(&two_heart, None));
        assert!(system.can_move_to_foundation(&two_heart, Some(&ace_heart)));
        assert!(!system.can_move_to_foundation(&three_heart, Some(&ace_heart)));
        assert!(!system.can_move_to_foundation(&ace_spade, Some(&ace_heart)));
        assert!(system.can_move_to_foundation(&three_heart, Some(&two_heart))); 
        println!("組札ルールチェックテスト、成功！🎉");
     }
    #[test] fn test_can_move_to_empty_tableau() { 
         let system = MoveCardSystem::new();
         let king = Card { suit: Suit::Club, rank: Rank::King, is_face_up: true };
         let queen = Card { suit: Suit::Diamond, rank: Rank::Queen, is_face_up: true };
         assert!(system.can_move_tableau_to_empty_tableau(&king));
         assert!(!system.can_move_tableau_to_empty_tableau(&queen));
         println!("空の場札ルールチェックテスト、成功！🎉");
     }

    // TODO: run / process_move_request / check_move_validity / apply_move のテストを追加！
    //       - World に適切なエンティティとコンポーネントを設定する必要がある
    //       - 移動リクエストをどうやって注入するか？ (テスト用の関数を作る？)
    //       - StackType をどうやって判定・設定するか？ (テスト用のダミーコンポーネント？)
    //       - 副作用 (Position の変更など) をちゃんと確認する！
} 