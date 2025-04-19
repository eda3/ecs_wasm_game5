//! ソリティアのゲームルール判定ロジックをまとめるモジュールだよ！🃏✅
//!
//! ここに関数を追加していくことで、カードがどこからどこへ移動できるか、
//! といったルールをチェックできるようにするんだ。

// 必要な型をインポートしておくよ！
use crate::components::card::{Card, Suit, Rank}; // ★修正: Color を削除！ (このファイル内で CardColor を定義してるから)
use crate::components::stack::{StackType, StackInfo}; // components の StackInfo, StackType を使う！
// use crate::world::World;                        // ゲーム世界の全体像 <-- これは使わない！
use crate::entity::Entity;                      // エンティティID (これは crate::entity のもの)
use crate::log;
use crate::world::World; // 自作 World を使うため
// use hecs::{World as HecsWorld, Entity as HecsEntity}; // <-- これを削除！

// TODO: 必要に応じて他のコンポーネントや型もインポートする！
// --- ここから自作ECSの型を定義していくことになる ---
// 例: type HecsWorld = crate::world::World; // 仮に自作Worldを使うようにする？
//     type HecsEntity = crate::entity::Entity;

/// カードの色（赤か黒か）を表すヘルパーenumだよ。
/// 場札 (Tableau) への移動ルール (色違い) で使う！❤️🖤
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CardColor {
    Red,
    Black,
}

impl CardColor {
    /// スートからカードの色を取得する関数。
    pub fn from_suit(suit: Suit) -> Self {
        match suit {
            Suit::Heart | Suit::Diamond => CardColor::Red, // ハートとダイヤは赤！♦️❤️
            Suit::Club | Suit::Spade => CardColor::Black,  // クラブとスペードは黒！♣️♠️
        }
    }
}

// --- カード移動の基本ルールチェック関数 ---
// これからここに具体的なルールチェック関数を追加していくよ！

/// 指定されたカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
/// **World の状態を考慮するバージョン！** 🌍
///
/// # 引数
/// * `world`: 現在のゲーム世界の `World` インスタンスへの参照。状態の読み取りに使うよ！
/// * `card_to_move_entity`: 移動させようとしているカードの `Entity` ID。
/// * `target_foundation_index`: 移動先の組札 (Foundation) のインデックス (0-3)。どのスートの組札かを示すよ！
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_to_foundation(
    world: &World,               // World インスタンスへの参照を受け取る
    card_to_move_entity: Entity, // 移動させたいカードの Entity ID
    target_foundation_index: u8, // 移動先の組札のインデックス (0-3)
) -> bool {
    // --- 1. 移動元のカード情報を取得 ---
    // まずは、移動させようとしているカードの Card コンポーネントを取得する。
    // get_component は Option<&Card> を返すから、見つからない可能性もあるよ。
    let card_to_move = match world.get_component::<Card>(card_to_move_entity) {
        // カードが見つかった！ card_to_move 変数に束縛する。
        Some(card) => card,
        // カードが見つからなかった… 移動元が不明なので false を返す。
        None => {
            log(&format!("[Rules Error] 移動元エンティティ {:?} に Card コンポーネントが見つかりません！", card_to_move_entity));
            return false;
        }
    };

    // --- 2. 移動先の組札 (Foundation) が受け入れるべきスートを取得 ---
    // target_foundation_index (0-3) を基に、その組札がどのスート (Suit) のカードを
    // 受け入れるべきかを get_foundation_suit ヘルパー関数で調べるよ。
    // このヘルパー関数は Option<Suit> を返す。
    let target_suit = match get_foundation_suit(target_foundation_index) {
        // 正しいスートが見つかった！ target_suit 変数に束縛する。
        Some(suit) => suit,
        // 無効なインデックス (0-3 以外) が指定されたなどでスートが見つからなかった…
        // この組札には置けないので false を返す。
        None => {
            log(&format!("[Rules Error] 無効な Foundation インデックス {} が指定されました！", target_foundation_index));
            return false;
        }
    };

    // --- 3. 移動元カードのスートが、移動先の組札のスートと一致するかチェック ---
    // Foundation ルールの基本！ スートが違ったら絶対に置けないよ。
    if card_to_move.suit != target_suit {
        // スートが違う！🙅‍♀️ false を返す。
        return false;
    }

    // --- 4. 移動先の組札の一番上のカード情報を取得 ---
    // まず、移動先の組札の StackType を作る。
    let target_stack_type = StackType::Foundation(target_foundation_index);
    // get_top_card_entity ヘルパーを使って、移動先組札の一番上のカード Entity を取得 (Option<Entity>)。
    let target_top_card_entity_option = get_top_card_entity(world, target_stack_type);

    // --- 5. ルール判定！ (ランクのチェック) ---
    // 移動先の組札の一番上のカード Entity が見つかったかどうかで場合分けするよ。
    match target_top_card_entity_option {
        // --- 5a. 移動先の組札が空の場合 (一番上のカード Entity が見つからなかった) ---
        None => {
            // 組札が空の場合、置けるのはエース (A) だけ！👑
            // 移動元のカード (card_to_move) のランクが Ace かどうかをチェックする。
            // スートの一致はステップ3で既に確認済みだよ！👍
            card_to_move.rank == Rank::Ace
        }
        // --- 5b. 移動先の組札にカードがある場合 (一番上のカード Entity が見つかった！) ---
        Some(target_top_card_entity) => {
            // 見つかった Entity ID を使って、そのカードの Card コンポーネントへの参照を取得する。
            let target_top_card = match world.get_component::<Card>(target_top_card_entity) {
                // カードコンポーネントが見つかった！👍
                Some(card) => card,
                // カードコンポーネントが見つからなかった…😱
                // ルール判断できないので false を返す。
                None => {
                    log(&format!("[Rules Error] 移動先トップエンティティ {:?} に Card コンポーネントが見つかりません！", target_top_card_entity));
                    return false;
                }
            };

            // これで移動元 (card_to_move) と移動先のトップ (target_top_card) の両方の
            // カード情報が手に入った！🙌 いよいよランクのルールチェックだ！

            // **ルール: ランクが連続しているか？** 📈
            // 移動元カードのランクが、移動先トップカードのランクよりちょうど1つ大きい必要があるよ。
            // (例: 移動先トップが A なら、移動元は 2 である必要がある)
            // スートの一致はステップ3で確認済み！
            // Rank enum を usize に変換して比較する。
            (card_to_move.rank as usize) == (target_top_card.rank as usize) + 1
            // 条件を満たせば true (移動可能)、満たさなければ false (移動不可) が返るよ。
        }
    }
}

/// 指定されたカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
/// **World の状態を考慮するバージョン！** 🌍
///
/// # 引数
/// * `world`: 現在のゲーム世界の `World` インスタンスへの参照。状態の読み取りに使うよ！
/// * `card_to_move_entity`: 移動させようとしているカードの `Entity` ID。
/// * `target_tableau_index`: 移動先の場札 (Tableau) のインデックス (0-6)。どの列かを示すよ！
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_to_tableau(
    world: &World,             // World インスタンスへの参照を受け取る
    card_to_move_entity: Entity, // 移動させたいカードの Entity ID
    target_tableau_index: u8,   // 移動先の場札の列番号 (0-6)
) -> bool {
    // --- 1. 移動元のカード情報を取得 ---
    // world から、指定された Entity ID (`card_to_move_entity`) に紐づく
    // Card コンポーネントへの参照を取得しようと試みるよ。
    // get_component は Option<&Card> を返すので、カードが見つからない可能性もあるんだ。
    let card_to_move = match world.get_component::<Card>(card_to_move_entity) {
        // カードコンポーネントが見つかった！やったね！🙌
        // `card` という変数名で束縛して、次の処理で使えるようにするよ。
        Some(card) => card,
        // カードコンポーネントが見つからなかった…🥺
        // 移動元のカード情報がないとルールを判断できないので、即座に false (移動不可) を返すよ。
        None => {
            log(&format!("[Rules Error] 移動元エンティティ {:?} に Card コンポーネントが見つかりません！", card_to_move_entity));
            return false;
        }
    };

    // --- 2. 移動先の場札 (Tableau) の一番上のカード情報を取得 ---
    // まず、移動先の場札の StackType を作るよ。Tableau はインデックスを持つからね！
    let target_stack_type = StackType::Tableau(target_tableau_index);

    // get_top_card_entity ヘルパー関数を使って、指定された場札 (target_stack_type) の
    // 一番上にあるカードの Entity ID (Option<Entity>) を取得するよ。
    let target_top_card_entity_option = get_top_card_entity(world, target_stack_type);

    // --- 3. ルール判定！ ---
    // 移動先の場札の一番上のカード Entity が見つかったかどうかで場合分けするよ。
    match target_top_card_entity_option {
        // --- 3a. 移動先の場札にカードがある場合 (一番上のカード Entity が見つかった！) ---
        Some(target_top_card_entity) => {
            // 見つかった Entity ID (`target_top_card_entity`) を使って、
            // そのカードの Card コンポーネントへの参照を取得するよ。
            let target_top_card = match world.get_component::<Card>(target_top_card_entity) {
                // カードコンポーネントが見つかった！👍
                Some(card) => card,
                // カードコンポーネントが見つからなかった…😱
                // 移動先のカード情報がないとルールを判断できないので、false (移動不可) を返すよ。
                None => {
                    log(&format!("[Rules Error] 移動先トップエンティティ {:?} に Card コンポーネントが見つかりません！", target_top_card_entity));
                    return false;
                }
            };

            // これで移動元 (card_to_move) と移動先 (target_top_card) の両方の
            // カード情報が手に入った！🙌 いよいよルールチェックだ！

            // **ルール1: 色が交互になっているか？** ❤️🖤
            // 移動元カードの色と移動先カードの色が違う必要があるよ。
            // CardColor ヘルパー enum を使って色を取得して比較する。
            let move_color = CardColor::from_suit(card_to_move.suit);
            let target_color = CardColor::from_suit(target_top_card.suit);
            if move_color == target_color {
                // 同じ色だったらダメ！🙅‍♀️ false を返す。
                return false;
            }

            // **ルール2: ランクが1つ小さいか？** 📉
            // 移動元カードのランクが、移動先カードのランクよりちょうど1つ小さい必要があるよ。
            // (例: 移動先が Q なら、移動元は J である必要がある)
            // Rank enum は usize に変換できるので、数値として比較する。
            if (card_to_move.rank as usize) != (target_top_card.rank as usize) - 1 {
                // ランクが連続していなければダメ！🙅‍♂️ false を返す。
                return false;
            }

            // 両方のルールをクリアした！🎉 移動可能なので true を返すよ！
            true
        }
        // --- 3b. 移動先の場札が空の場合 (一番上のカード Entity が見つからなかった) ---
        None => {
            // 場札の列が空の場合、置けるのはキング (K) だけ！🤴
            // 移動元のカード (card_to_move) のランクが King かどうかをチェックする。
            card_to_move.rank == Rank::King
        }
    }
}

/// ストック（山札）からウェスト（捨て札）にカードを配れるかチェックする。
/// (この関数は単純化されており、実際には World の状態を見る必要があるかもしれない)
///
/// # 引数
/// * `stock_is_empty`: ストックが現在空かどうか。
///
/// # 戻り値
/// * ストックから配れるなら `true`、そうでなければ `false`。
pub fn can_deal_from_stock(stock_is_empty: bool) -> bool {
    !stock_is_empty // ストックが空でなければ配れる
}

/// ストック（山札）が空のときに、ウェスト（捨て札）からストックにカードを戻せるかチェックする。
/// (この関数は単純化されており、実際には World の状態を見る必要があるかもしれない)
///
/// # 引数
/// * `stock_is_empty`: ストックが現在空かどうか。
/// * `waste_is_empty`: ウェストが現在空かどうか。
///
/// # 戻り値
/// * ウェストからストックに戻せる（リセットできる）なら `true`、そうでなければ `false`。
pub fn can_reset_stock_from_waste(stock_is_empty: bool, waste_is_empty: bool) -> bool {
    stock_is_empty && !waste_is_empty // ストックが空で、ウェストにカードがあればリセットできる
}

/// ウェスト（捨て札）の一番上のカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
/// **World の状態を考慮するバージョン！** 🌍
///
/// # 引数
/// * `world`: 現在のゲーム世界の `World` インスタンスへの参照。状態の読み取りに使うよ！
/// * `waste_top_card_entity`: 移動させようとしているウェストの一番上のカードの `Entity` ID。
/// * `target_tableau_index`: 移動先の場札 (Tableau) のインデックス (0-6)。どの列かを示すよ！
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_from_waste_to_tableau(
    world: &World,               // World インスタンスへの参照を受け取る
    waste_top_card_entity: Entity, // 移動させたいウェストのトップカードの Entity ID
    target_tableau_index: u8,     // 移動先の場札の列番号 (0-6)
) -> bool {
    // 基本的なロジックは `can_move_to_tableau` と全く同じだよ！✨
    // なので、ここでは World, 移動元Entity, 移動先インデックス をそのまま渡して
    // `can_move_to_tableau` 関数を呼び出して、その結果を返すだけ！シンプル！👍
    can_move_to_tableau(world, waste_top_card_entity, target_tableau_index)
}

/// ウェスト（捨て札）の一番上のカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
/// **World の状態を考慮するバージョン！** 🌍
///
/// # 引数
/// * `world`: 現在のゲーム世界の `World` インスタンスへの参照。状態の読み取りに使うよ！
/// * `waste_top_card_entity`: 移動させようとしているウェストの一番上のカードの `Entity` ID。
/// * `target_foundation_index`: 移動先の組札 (Foundation) のインデックス (0-3)。どのスートの組札かを示すよ！
///
/// # 戻り値
/// * 移動可能なら `true`、そうでなければ `false`。
pub fn can_move_from_waste_to_foundation(
    world: &World,                 // World インスタンスへの参照を受け取る
    waste_top_card_entity: Entity,   // 移動させたいウェストのトップカードの Entity ID
    target_foundation_index: u8,   // 移動先の組札のインデックス (0-3)
) -> bool {
    // 基本的なロジックは `can_move_to_foundation` と全く同じだよ！💖
    // なので、ここでは World, 移動元Entity, 移動先インデックス をそのまま渡して
    // `can_move_to_foundation` 関数を呼び出して、その結果を返すだけ！超簡単！👍
    can_move_to_foundation(world, waste_top_card_entity, target_foundation_index)
}

/// ゲームのクリア条件（全てのカードが組札にあるか）を判定する。
///
/// # 引数
/// * `foundation_card_count`: 現在、全ての組札（Foundation）にあるカードの合計枚数。
///
/// # 戻り値
/// * クリア条件を満たしていれば `true`、そうでなければ `false`。
pub fn check_win_condition(foundation_card_count: usize) -> bool {
    foundation_card_count == 52 // 標準的な52枚デッキの場合
}

// --- 自動移動関連のヘルパー関数 ---

/// 組札 (Foundation) のインデックス (0-3) から対応するスートを取得する。
/// 約束事: 0: Heart ❤️, 1: Diamond ♦️, 2: Club ♣️, 3: Spade ♠️
/// 引数のインデックスが無効 (0-3以外) の場合は None を返すよ。
/// `pub(crate)` なので、`logic` モジュールとそのサブモジュール内からのみ呼び出せる。
pub(crate) fn get_foundation_suit(foundation_index: u8) -> Option<Suit> {
    match foundation_index {
        0 => Some(Suit::Heart),   // インデックス 0 はハート ❤️
        1 => Some(Suit::Diamond), // インデックス 1 はダイヤ ♦️
        2 => Some(Suit::Club),    // インデックス 2 はクラブ ♣️
        3 => Some(Suit::Spade),   // インデックス 3 はスペード ♠️
        _ => None, // 0, 1, 2, 3 以外のインデックスは無効なので None を返す
    }
}

/// 指定された組札 (Foundation) の一番上にあるカードを取得するヘルパー関数。
/// World の状態を調べて、StackInfo を持つエンティティから見つける。
/// TODO: 自作Worldからデータを取得するロジックを実装する必要あり！ -> これは get_top_card_entity が担当するはず！コメント古い？🤔
// pub(crate) fn get_foundation_top_card<'a>(world: &'a World, foundation_index: u8) -> Option<&'a Card> {
//     // 1. StackType::Foundation(foundation_index) の StackInfo を持つ Entity を探す。
//     // 2. その Entity に関連付けられた StackItem コンポーネントのうち、pos_in_stack が最大のものを探す。
//     // 3. 見つかった StackItem の Card コンポーネントへの参照を返す。

//     // 仮実装: とりあえず None を返す
//     None
// }

/// 特定のカードが、現在のワールドの状態において、自動的に移動できる組札（Foundation）があるかどうかを探す関数。
/// 見つかった場合は、移動先の StackType (Foundation のインデックス付き) を返す。
///
/// # 引数
/// - `card_to_move`: 移動させたいカードのコンポーネントへの参照 (`component::Card`)。
/// - `world`: 現在の World の状態への参照 (自作World)。
///
/// # 戻り値
/// - `Some(StackType)`: 移動可能な組札が見つかった場合、その組札の StackType (`component::StackType`)。
///                     注意: StackType::Foundation(index) の形で返すよ！
/// - `None`: 移動可能な組札が見つからなかった場合。
// pub fn find_automatic_foundation_move<'a>(
//     card_to_move: &Card,
//     world: &'a World
// ) -> Option<StackType> {
//     log(&format!("[Rules] Finding automatic foundation move for {:?}...", card_to_move));

//     for i in 0..4u8 { // 4つの Foundation をチェック
//         let foundation_suit = get_foundation_suit(i);

//         if foundation_suit.is_none() { continue; } // 無効なインデックスはスキップ
//         let foundation_suit = foundation_suit.unwrap();

//         // Foundation の一番上のカードを取得
//         let foundation_top_card: Option<&Card> = get_foundation_top_card(world, i);

//         // 移動可能かチェック
//         if can_move_to_foundation(card_to_move, foundation_top_card, foundation_suit) {
//             log(&format!("  Found valid foundation [{}] for {:?}. Top card: {:?}", i, card_to_move, foundation_top_card));
//             // 移動可能な Foundation が見つかったので、StackType::Foundation(i) を返す
//             return Some(StackType::Foundation(i));
//         }
//     }

//     log(&format!("  No suitable foundation found for {:?}.", card_to_move));
//     None // 適切な移動先が見つからなかった
// }

/// 指定されたスタック (`target_stack`) の一番上にあるカードのエンティティID (`Entity`) を取得するよ。
// ... (関数コメント略) ...
fn get_top_card_entity(world: &World, target_stack: StackType) -> Option<Entity> {
    // log(&format!("[Rules Helper] get_top_card_entity for {:?} called", target_stack)); // デバッグログ

    // StackInfo コンポーネントを持つ全てのエンティティを取得するイテレータを作成。
    let stack_entities = world.get_all_entities_with_component::<StackInfo>();

    // イテレータを処理していくよ！
    // ★★★ エラー修正: Vec<Entity> をイテレータにするために .into_iter() を追加！ ★★★
    stack_entities
        .into_iter() // <- これを追加！ Vec をイテレータに変換！
        // filter を使って、各エンティティの StackInfo をチェックする。
        .filter(|&entity| {
            // world から StackInfo コンポーネントを取得。
            world.get_component::<StackInfo>(entity)
                // map_or を使って、Option の中身を処理する。
                .map_or(false, |stack_info| stack_info.stack_type == target_stack)
        })
        // フィルターされたエンティティの中から、position_in_stack が最大のものを探す。
        .max_by_key(|&entity| {
            // world から StackInfo を取得。
            world.get_component::<StackInfo>(entity)
                // map_or を使って、Some(stack_info) なら position_in_stack を返す。
                .map_or(0, |stack_info| stack_info.position_in_stack)
        })
    // max_by_key は Option<Entity> を返すので、それをそのまま関数の戻り値とする。
}

// TODO: 他の移動パターン (Stock -> Waste, Waste -> Tableau/Foundation など) の
//       ルールチェック関数も必要に応じて追加していく！💪

// --- テストコード ---
#[cfg(test)]
mod tests {
    use super::*; // 親モジュールの要素を使う
    use crate::components::card::Rank; // こっちが正しいパス！
    use crate::world::World; // 自作Worldを使う
    use crate::entity::Entity; // 自作Entityを使う
    use crate::components::card::{Card, Suit}; // Card, Suit 追加
    use crate::components::stack::{StackType, StackInfo}; // StackType, StackInfo 追加

    // --- テスト用ヘルパー関数 ---
    /// テストワールドにカードエンティティを追加するヘルパー関数だよ。
    /// 指定されたスート、ランク、スタックタイプ、スタック内位置を持つカードエンティティを作成して、
    /// World に Card と StackInfo コンポーネントを登録し、その Entity ID を返すよ。
    fn add_card_for_test(world: &mut World, suit: Suit, rank: Rank, stack_type: StackType, pos: u8) -> Entity {
        // 新しいエンティティを作成
        let entity = world.create_entity();
        // カードコンポーネントを作成 (is_face_up は常に true でテストするよ)
        let card = Card { suit, rank, is_face_up: true };
        // スタック情報コンポーネントを作成
        let stack_info = StackInfo { stack_type, position_in_stack: pos };
        // 作成したエンティティにコンポーネントを追加
        world.add_component(entity, card);
        world.add_component(entity, stack_info);
        // 作成したエンティティの ID を返す
        entity
    }

    // --- 既存のテスト ... ---
    #[test]
    fn test_card_color() {
        assert_eq!(CardColor::from_suit(Suit::Heart), CardColor::Red);
        assert_eq!(CardColor::from_suit(Suit::Diamond), CardColor::Red);
        assert_eq!(CardColor::from_suit(Suit::Club), CardColor::Black);
        assert_eq!(CardColor::from_suit(Suit::Spade), CardColor::Black);
        println!("CardColor テスト、成功！🎉");
    }

    /* // TODO: World を使うようにテストを修正・追加する必要がある！
    #[test]
    fn test_can_move_to_foundation_rules() {
        // テスト用のカードを作成 (component::Card)
        let ace_hearts = Card { suit: Suit::Heart, rank: Rank::Ace, is_face_up: true };
        let two_hearts = Card { suit: Suit::Heart, rank: Rank::Two, is_face_up: true };
        let three_hearts = Card { suit: Suit::Heart, rank: Rank::Three, is_face_up: true };
        let ace_spades = Card { suit: Suit::Spade, rank: Rank::Ace, is_face_up: true };

        // --- Foundation が空の場合 ---
        assert!(can_move_to_foundation(&ace_hearts, None, Suit::Heart), "空のHeart Foundation に Ace of Hearts は置けるはず");
        assert!(!can_move_to_foundation(&two_hearts, None, Suit::Heart), "空のHeart Foundation に 2 of Hearts は置けないはず");
        assert!(!can_move_to_foundation(&ace_spades, None, Suit::Heart), "空のHeart Foundation に Ace of Spades は置けないはず");

        // --- Foundation に Ace がある場合 ---
        assert!(can_move_to_foundation(&two_hearts, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 2 of Hearts は置けるはず");
        assert!(!can_move_to_foundation(&three_hearts, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 3 of Hearts は置けないはず");
        let two_spades = Card { suit: Suit::Spade, rank: Rank::Two, is_face_up: true };
        assert!(!can_move_to_foundation(&two_spades, Some(&ace_hearts), Suit::Heart), "Heart Foundation (Ace) に 2 of Spades は置けないはず");

        // --- Foundation に 2 がある場合 ---
        assert!(can_move_to_foundation(&three_hearts, Some(&two_hearts), Suit::Heart), "Heart Foundation (Two) に 3 of Hearts は置けるはず");

        println!("Foundation 移動ルールテスト、成功！🎉");
    }
    */

    #[test]
    fn test_stock_waste_rules() {
        // ストックがある場合
        assert!(can_deal_from_stock(false), "ストックがあれば配れるはず");
        assert!(!can_reset_stock_from_waste(false, false), "ストックがある場合はリセットできないはず");
        assert!(!can_reset_stock_from_waste(false, true), "ストックがある場合はリセットできないはず");

        // ストックが空の場合
        assert!(!can_deal_from_stock(true), "ストックが空なら配れないはず");
        assert!(can_reset_stock_from_waste(true, false), "ストックが空でウェストにあればリセットできるはず");
        assert!(!can_reset_stock_from_waste(true, true), "ストックもウェストも空ならリセットできないはず");
        println!("Stock/Waste ルールテスト、成功！🎉");
    }

    #[test]
    fn test_win_condition() {
        assert!(check_win_condition(52), "カードが52枚あればクリアなはず！🏆");
        assert!(!check_win_condition(51), "カードが51枚ではクリアじゃないはず！🙅");
        assert!(!check_win_condition(0), "カードが0枚ではクリアじゃないはず！🙅");
        println!("ゲームクリア判定テスト、成功！🎉");
    }

    // --- find_automatic_foundation_move のテストは src/logic/auto_move.rs に移動しました ---

    #[test]
    fn test_can_move_to_tableau_world() {
        println!("--- test_can_move_to_tableau_world 開始 ---");
        // --- 準備 ---
        // テスト用の World を作成
        let mut world = World::new();
        // テストに必要なコンポーネントを World に登録
        world.register_component::<Card>();
        world.register_component::<StackInfo>();

        // --- テストカードエンティティの作成 ---
        // King of Spades (Waste の 0番目にあるとする)
        let king_spades_entity = add_card_for_test(&mut world, Suit::Spade, Rank::King, StackType::Waste, 0);
        // Queen of Hearts (Waste の 1番目にあるとする)
        let queen_hearts_entity = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Waste, 1);
        // Jack of Spades (Waste の 2番目にあるとする)
        let jack_spades_entity = add_card_for_test(&mut world, Suit::Spade, Rank::Jack, StackType::Waste, 2);
        // Jack of Diamonds (Waste の 3番目にあるとする)
        let jack_diamonds_entity = add_card_for_test(&mut world, Suit::Diamond, Rank::Jack, StackType::Waste, 3);
        // Ten of Spades (Waste の 4番目にあるとする)
        let ten_spades_entity = add_card_for_test(&mut world, Suit::Spade, Rank::Ten, StackType::Waste, 4);

        // --- シナリオ 1: 空の Tableau への移動 ---
        println!("Scenario 1: 空の Tableau への移動");
        // 空の Tableau (インデックス 0) に King of Spades (黒) は移動できるはず！
        assert!(
            can_move_to_tableau(&world, king_spades_entity, 0),
            "空の Tableau 0 に King of Spades は置けるはず"
        );
        // 空の Tableau (インデックス 1) に Queen of Hearts (赤) は移動できないはず！ (Kingじゃないから)
        assert!(
            !can_move_to_tableau(&world, queen_hearts_entity, 1),
            "空の Tableau 1 に Queen of Hearts は置けないはず"
        );

        // --- シナリオ 2: 空でない Tableau への有効な移動 ---
        println!("Scenario 2: 空でない Tableau への有効な移動");
        // Tableau 2 の一番上に Queen of Hearts (赤) を置く (位置 0)
        let target_q_hearts_t2 = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Tableau(2), 0);
        // Tableau 2 (一番上が Q❤️) に Jack of Spades (黒, Qよりランク-1) は移動できるはず！
        assert!(
            can_move_to_tableau(&world, jack_spades_entity, 2),
            "Tableau 2 (Q❤️) に Jack of Spades (黒) は置けるはず"
        );

        // --- シナリオ 3: 空でない Tableau への無効な移動 (同色) ---
        println!("Scenario 3: 空でない Tableau への無効な移動 (同色)");
        // Tableau 3 の一番上に Queen of Hearts (赤) を置く (位置 0)
        let target_q_hearts_t3 = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Tableau(3), 0);
        // Tableau 3 (一番上が Q❤️) に Jack of Diamonds (赤, Qよりランク-1だけど同色) は移動できないはず！
        assert!(
            !can_move_to_tableau(&world, jack_diamonds_entity, 3),
            "Tableau 3 (Q❤️) に Jack of Diamonds (赤) は置けないはず (同色)"
        );

        // --- シナリオ 4: 空でない Tableau への無効な移動 (ランク違い) ---
        println!("Scenario 4: 空でない Tableau への無効な移動 (ランク違い)");
        // Tableau 4 の一番上に Queen of Hearts (赤) を置く (位置 0)
        let target_q_hearts_t4 = add_card_for_test(&mut world, Suit::Heart, Rank::Queen, StackType::Tableau(4), 0);
        // Tableau 4 (一番上が Q❤️) に Ten of Spades (黒, 色は違うけどランクがQより-2) は移動できないはず！
        assert!(
            !can_move_to_tableau(&world, ten_spades_entity, 4),
            "Tableau 4 (Q❤️) に Ten of Spades (黒) は置けないはず (ランク違い)"
        );

        println!("--- test_can_move_to_tableau_world 完了 ---");
        // 注意: このテストは World の状態を変更したまま終了する。
        // より厳密なテストでは、テスト後に World をクリーンアップするか、
        // 各シナリオで独立した World を使うのが望ましい場合があるよ。
    }

    /* // TODO: World を使うようにテストを修正・追加する必要がある！
    #[test]
    fn test_stock_waste_rules() {
        // ... (略) ...
        println!("Stock/Waste ルールテスト、成功！🎉");
    }
    */

    /* // TODO: World を使うようにテストを修正・追加する必要がある！
    #[test]
    fn test_can_move_from_waste_rules() {
        // ... (古いテストコードは削除) ...
        println!("Waste からの移動ルールテスト、成功！🎉");
    }
    */

    // ... (略) ...
}

// ▲▲▲ HecsWorld を使っている部分を修正する必要がある ▲▲▲ -> これはもう関係ないコメントだね！削除してもいいかも！