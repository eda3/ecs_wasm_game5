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
use crate::logic::rules;
use crate::app::layout_calculator;
use web_sys::console;
use wasm_bindgen::JsValue;

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

    /// 指定されたカードを指定されたスタックに移動させる処理を実行する。
    /// ルールチェック (is_move_valid) は呼び出し元 (System::run など) で行う前提。
    ///
    /// # 引数
    /// * `world` - World への可変参照。
    /// * `moved_entity` - 移動させるカードのエンティティ。
    /// * `target_stack` - 移動先のスタックタイプ。
    fn process_move(
        &mut self,
        world: &mut World,
        moved_entity: Entity,
        target_stack: StackType,
    ) {
        console::log_1(&JsValue::from_str(&format!(
            "[MoveCardSystem] Processing move for {:?} to {:?}...",
            moved_entity,
            target_stack
        )));

        // --- 1. 移動させるカードの StackInfo を更新 --- 
        let new_position_in_stack = {
            // 移動先のスタックに既に存在するカードの数を数える
            let target_entities = world.get_all_entities_with_component::<StackInfo>();
            target_entities
                .iter()
                .filter(|&&e| {
                    world.get_component::<StackInfo>(e)
                        .map_or(false, |si| si.stack_type == target_stack)
                })
                .count() as u8 // 新しいカードは一番上に追加されるので、既存の数がそのまま position になる
        };

        if let Some(stack_info) = world.get_component_mut::<StackInfo>(moved_entity) {
            console::log_1(&JsValue::from_str(&format!(
                "  Updating StackInfo for {:?}: {:?} -> {:?}, pos: {} -> {}",
                moved_entity,
                stack_info.stack_type,
                target_stack,
                stack_info.position_in_stack,
                new_position_in_stack
            )));
            stack_info.stack_type = target_stack;
            stack_info.position_in_stack = new_position_in_stack;
        } else {
            console::log_1(&JsValue::from_str(&format!(
                "[MoveCardSystem Error] Failed to get StackInfo for moved entity {:?}!",
                moved_entity
            )));
            return; // StackInfo がないと位置計算などができないので中断
        }

        // --- 2. 移動させるカードの Position を更新 --- 
        // layout_calculator を使って新しい座標を計算する
        let new_position = layout_calculator::calculate_card_position(
            target_stack,           // 新しいスタックタイプ
            new_position_in_stack, // 新しいスタック内位置
            world,                 // World の現在の状態を参照して計算
        );

        if let Some(position) = world.get_component_mut::<Position>(moved_entity) {
            console::log_1(&JsValue::from_str(&format!(
                "  Updating Position for {:?}: ({}, {}) -> ({}, {})",
                moved_entity,
                position.x, position.y,
                new_position.x, new_position.y
            )));
            position.x = new_position.x;
            position.y = new_position.y;
        } else {
             console::log_1(&JsValue::from_str(&format!(
                "[MoveCardSystem Error] Failed to get Position for moved entity {:?}!",
                moved_entity
            )));
             // Position がなくても処理は続けられるかもしれないが、一応ログは出す
        }

        // --- 3. 移動させるカードの Card 状態を更新 (必要なら) ---
        // 例: Tableau に移動したら表向きにする、など (クロンダイク固有のルール)
        if let Some(card) = world.get_component_mut::<Card>(moved_entity) {
            if matches!(target_stack, StackType::Tableau(_)) {
                if !card.is_face_up {
                    console::log_1(&JsValue::from_str(&format!(
                        "  Flipping card {:?} face up.",
                        moved_entity
                    )));
                    card.is_face_up = true;
                }
            }
            // 他のスタックタイプでのルール (例: Stock に戻ったら裏向きとか) があればここに追加
        }

        // --- 4. 移動元のスタックの状態更新 (必要なら) ---
        // 例: 移動元が Tableau で、その下に裏向きカードがあったら表向きにする (クロンダイク)
        // これは、移動するカードの元の StackInfo が必要になるので、
        // この関数の最初で保存しておく必要がある。
        // TODO: 元のスタック情報を使った処理を追加する

        console::log_1(&JsValue::from_str(&format!(
            "[MoveCardSystem] Move processed successfully for {:?}.
",
            moved_entity
        )));
    }
}

impl System for MoveCardSystem {
    /// カード移動のロジックを実行するよ！(リファクタリング後)
    fn run(&mut self, world: &mut World) {
        // ここに、移動リクエスト (例: イベントキューやネットワークメッセージから) を取得し、
        // logic::rules::is_move_valid でチェックし、
        // 問題なければ self.process_move(world, moved_entity, target_stack) を呼び出す
        // ロジックを実装する。

        // --- ダミー実装 (テスト用) ---
        // 仮の移動リクエストを処理する例
        let move_requests: Vec<(Entity, StackType)> = Vec::new(); // 本来はどこかから取得

        if !move_requests.is_empty() {
            console::log_1(&JsValue::from_str(&format!(
                "[MoveCardSystem] Running... Processing {} move requests.",
                move_requests.len()
            )));

            for (moved_entity, target_stack) in move_requests {
                console::log_1(&JsValue::from_str(&format!(
                    "  Checking move validity for {:?} -> {:?}...",
                    moved_entity, target_stack
                )));
                // ルールチェック！
                if rules::is_move_valid(world, moved_entity, target_stack) {
                     console::log_1(&JsValue::from_str("  Move is valid! Processing..."));
                    // 有効なら移動処理を実行！
                    self.process_move(world, moved_entity, target_stack);
                } else {
                    console::log_1(&JsValue::from_str("  Move is invalid!"));
                    // 無効な場合は何もしないか、エラー通知などを行う
                }
            }
            console::log_1(&JsValue::from_str("[MoveCardSystem] Finished processing requests."));
        } else {
            // console::log_1(&JsValue::from_str("[MoveCardSystem] Running... No move requests to process."));
        }
        // --- ダミー実装ここまで ---
    }
}

// --- 削除: テストコード (rules 側に移動したため) --- 