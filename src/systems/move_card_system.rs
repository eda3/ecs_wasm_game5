use crate::{ // 必要なモジュールや型をインポート
    component::Component,
    components::{card::{Card, Suit, Rank}, position::Position, player::Player, game_state::{GameState, GameStatus}, stack::{StackInfo, StackType}},
    entity::Entity,
    system::System,
    world::World,
};

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
        // clone するのは、後で可変参照を取るための一時的な対策
        let moved_card_opt = world.get_component::<Card>(moved_entity).cloned();
        let target_card_opt = world.get_component::<Card>(target_entity).cloned();
        let target_pos_opt = world.get_component::<Position>(target_entity).cloned();

        // --- 3. ルールチェック＆状態更新 ---
        if let Some(moved_card) = moved_card_opt {
            // 移動元と移動先の種類を判定 (TODO: この判定ロジックが必要！)
            let source_type_opt = self.get_entity_stack_type(world, moved_entity);
            let target_type_opt = self.get_entity_stack_type(world, target_entity);

            if let (Some(source_type), Some(target_type)) = (source_type_opt, target_type_opt) {
                 // ルールチェックを実行
                if self.check_move_validity(world, &moved_card, target_card_opt.as_ref(), source_type, target_type) {
                    // 状態更新を実行
                    self.apply_move(world, moved_entity, target_entity, target_pos_opt);
                } else {
                    println!("  ルール違反！移動できませんでした。🙅‍♀️");
                }
            } else {
                eprintln!("MoveCardSystem: 移動元または移動先の種類を特定できませんでした。");
            }
        } else {
            eprintln!("MoveCardSystem: 移動元のカード {:?} が見つかりません！", moved_entity);
        }
    }

    /// エンティティがどの種類のスタックに属するかを返す (TODO: 実装！)
    /// StackType コンポーネントなどを Entity に持たせる必要がある
    fn get_entity_stack_type(&self, world: &World, entity: Entity) -> Option<StackType> {
        // 仮実装: WorldからStackTypeコンポーネントを取得する想定
        // world.get_component::<StackTypeComponent>(entity).map(|comp| comp.stack_type)
        println!("TODO: get_entity_stack_type 実装");
        // とりあえず仮で場札を返す (テスト用)
        if entity.0 < 52 { Some(StackType::Tableau(0)) } else { None } // 仮！
    }


    /// 移動がルール上可能かチェックする関数
    fn check_move_validity(
        &self,
        world: &World, // world が必要な場合があるかも (e.g., 組札の状態を見る)
        moved_card: &Card,
        target_card_opt: Option<&Card>, // 移動先がカードの場合
        source_type: StackType,
        target_type: StackType,
    ) -> bool {
        println!("  ルールチェック実行: {:?} ({:?}) -> {:?}", moved_card.rank, source_type, target_type);
        match (source_type, target_type) {
            // --- 場札 (Tableau) からの移動 ---
            (StackType::Tableau(_), StackType::Tableau(_)) => {
                if let Some(target_card) = target_card_opt {
                    self.can_move_tableau_to_tableau(moved_card, target_card)
                } else {
                    self.can_move_tableau_to_empty_tableau(moved_card)
                }
            }
            (StackType::Tableau(_), StackType::Foundation(target_suit_index)) => {
                // 場札 -> 組札
                // TODO: target_entity (組札の場所) に対応する組札の一番上のカードを取得する必要がある
                let foundation_top_card: Option<&Card> = None; // 仮！
                // TODO: ↓の Suit チェックは target_suit_index (u8) と比較できないのでコメントアウト。
                //       正しいチェックロジック (Foundation index がどの Suit に対応するか World から引く等) が必要。
                // if moved_card.suit != target_suit { return false; } // スートが違う組札には置けない
                self.can_move_to_foundation(moved_card, foundation_top_card)
            }

            // --- 山札 (Stock/Waste) からの移動 ---
            (StackType::Waste, StackType::Tableau(_)) => {
                 if let Some(target_card) = target_card_opt {
                    self.can_move_stock_to_tableau(moved_card, target_card) // ルールは同じ
                } else {
                    self.can_move_stock_to_empty_tableau(moved_card) // ルールは同じ
                }
            }
             (StackType::Waste, StackType::Foundation(target_suit_index)) => {
                // Waste -> 組札
                // TODO: 組札の一番上のカードを取得
                let foundation_top_card: Option<&Card> = None; // 仮！
                // TODO: ↓の Suit チェックは target_suit_index (u8) と比較できないのでコメントアウト。
                //       正しいチェックロジックが必要。
                // if moved_card.suit != target_suit { return false; }
                self.can_move_stock_to_foundation(moved_card, foundation_top_card) // ルールは同じ
            }

            // --- 他の移動パターンは基本的に不可 ---
            _ => {
                println!("  未対応または不正な移動パターンです: {:?} -> {:?}", source_type, target_type);
                false
            }
        }
    }

    /// 実際に World の状態を更新する関数
    fn apply_move(&self, world: &mut World, moved_entity: Entity, target_entity: Entity, target_pos_opt: Option<Position>) {
        println!("  カード {:?} を {:?} へ移動します！", moved_entity, target_entity);

        // 1. 移動するカードの Position コンポーネントを更新
        if let Some(target_pos) = target_pos_opt {
            if let Some(moved_pos_mut) = world.get_component_mut::<Position>(moved_entity) {
                // TODO: 重ねて表示する場合のオフセット計算 (ターゲットの種類やスタックのカード数による)
                let y_offset = 0.0; // 仮
                moved_pos_mut.x = target_pos.x;
                moved_pos_mut.y = target_pos.y + y_offset;
                println!("    {:?} の位置を ({}, {}) に更新しました。", moved_entity, moved_pos_mut.x, moved_pos_mut.y);
            }
        } else {
            eprintln!("MoveCardSystem: 移動先の Position が見つかりません！");
            // 位置の更新ができない場合は移動を中断すべき？ or エラー？
            return;
        }

        // 2. 必要ならカードの表裏状態 (is_face_up) を更新
        // 例: 場札で下に隠れていたカードを表にする
        // TODO: 移動元のスタックに残った一番上のカードが裏向きなら表にする処理が必要
        //       そのためには、カードがどのスタックの何番目にあったか、という情報も必要になるかも？ (面倒！)

        // 3. 必要ならエンティティの親子関係や所属スタック情報を更新
        // TODO: カードがどのスタックに属しているかを示すコンポーネント (e.g., Parent, StackMembership) があれば更新

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
        let maybe_move_request: Option<(Entity, Entity)> = None; // 仮

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