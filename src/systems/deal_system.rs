// src/systems/deal_system.rs

// === 使うものたちを宣言！ (use 文) ===
// これからコードの中で使う部品 (構造体、トレイト、関数とか) を他のファイルから持ってくるよ！
// `crate::` は、このプロジェクト (クレート) の一番上の階層から見てるって意味だよ！

// World: エンティティやコンポーネントを管理する、アタシたちのゲーム世界の"神様"みたいな存在！🌍
use crate::ecs::world::World;
// components モジュールの中の card, position, stack サブモジュールにあるものをまとめて使う宣言！
// Card: カードのスートやランク、表か裏かの情報を持つデータ部品。
// Position: エンティティの画面上の座標 (x, y) を持つデータ部品。
// StackInfo: カードがどの場所 (山札？場札？) の何番目にあるかの情報を持つデータ部品。
// StackType: `StackInfo` の中で使う、場所の種類 (山札、場札、組札、捨て札) を表すマーカー。
use crate::components::{card::{/*self, */Card, /*Suit, Rank*/}, position::Position, stack::{StackInfo, StackType}}; // self, Suit, Rank は削除 (Card は使ってる)
// Entity: ゲーム世界のモノ (カードとかプレイヤーとか) を識別するためのユニークなID。
use crate::ecs::entity::Entity;
// rand クレート (外部ライブラリ) から、ランダム系の機能をもらうよ！
// SliceRandom: 配列 (スライス) の要素をシャッフルする機能 (`shuffle` メソッド) を提供してくれる。
// use rand::seq::SliceRandom; // logic/deck.rs の shuffle_deck を使うため不要
// thread_rng: OSが提供する、暗号学的に安全な乱数生成器を使うための関数。
// use rand::thread_rng; // logic/deck.rs の shuffle_deck を使うため不要
// config::layout モジュールから、カード配置の座標とかオフセットの定数をもらうよ！レイアウト調整はこっちでやるのがスマート！✨
use crate::config::layout::*;
// logic::deck モジュールから、デッキ作成とシャッフルのヘルパー関数をもらうよ！ロジックは別ファイルに分けるのがお作法！👍
use crate::logic::deck::{create_standard_deck, shuffle_deck};

// === 初期カード配置システム (DealInitialCardsSystem) ===
// これが今回の主役！✨ ゲームが始まった時に、カードをシャッフルして場に配るっていう大事な役目を持ってる「システム」だよ！
// ECSでいう「システム」は、特定のルールに基づいて World の中のコンポーネントを読み書きして、ゲームの状態を進める役割を持つんだ。

// `#[derive(Default)]` は、`DealInitialCardsSystem::default()` って書くだけで、
// この構造体のインスタンス (実体) を簡単に作れるようにしてくれる便利なおまじないだよ！🪄
// 中にデータを持たない構造体だから、`Default` が自動で実装できるんだ。
#[derive(Default)]
pub struct DealInitialCardsSystem;

// `impl` ブロックを使って、`DealInitialCardsSystem` 構造体に関連するメソッド (関数みたいなもの) を定義していくよ！
impl DealInitialCardsSystem {
    /// ゲームの初期カード配置を実行するメイン関数だよ！ 🎉 Let's deal! 🃏
    /// この関数が呼ばれると、まっさらな World にソリティアの初期盤面が作られるんだ！
    ///
    /// # 引数 (ひきすう)
    /// - `world: &mut World`: ゲーム世界の管理人 `World` さんの **可変参照** (`&mut`) を受け取るよ。
    ///   可変参照っていうのは、「`world` の中身を読み書きする権利をもらいますよ！」っていう意味。
    ///   カードエンティティを作ったり、コンポーネントを追加したり、`world` の状態を変えるから `&mut` が必須なんだ！🔥
    ///
    /// # 処理の流れ (ざっくり！)
    /// 1. **デッキ準備！**: まずは新品の52枚のカードデッキを用意して、よーくシャッフル！섞어섞어！🔀
    /// 2. **お掃除タイム！**: もし前にプレイした時のカードが残ってたら大変だから、先に全部キレイにするよ！🧹
    /// 3. **カード配置！**: シャッフルしたデッキからカードを一枚ずつ取り出して、ソリティア (クロンダイク) のルール通りに配置していくよ！
    ///    - 場札 (Tableau): 7つの列に、1枚、2枚、...7枚って感じで配って、各列の一番上だけ表向きにする！👀
    ///    - 山札 (Stock): 残ったカードは全部、山札に裏向きで積む！⛰️
    /// 4. **情報付与！**: 配置したカード一枚一枚に、「私はスペードのエースだよ！」(`Card` コンポーネント)、「私は場札の3列目の2番目だよ！」(`StackInfo` コンポーネント)、「私の画面上の位置はここだよ！」(`Position` コンポーネント) っていう情報を `world` に登録していく！✍️
    pub fn execute(&self, world: &mut World) {
        println!("🚀 DealInitialCardsSystem: 実行開始！ 初期カード配置を始めます！");

        // --- 1. デッキの準備 --- ✨♠️♥️♦️♣️✨
        // `logic/deck.rs` にある `create_standard_deck` 関数を呼び出して、52枚のカードデータ (Card 構造体のリスト) を作るよ。
        // `let mut deck_cards` の `mut` は、「この変数 `deck_cards` の中身は後で変えるかもよ！」っていう印。
        // シャッフルで順番を変えるから `mut` が必要！🔥
        let mut deck_cards = create_standard_deck();
        // 同じく `logic/deck.rs` の `shuffle_deck` 関数で、デッキをごちゃ混ぜにする！ランダム大事！🎲
        // `&mut deck_cards` で、`deck_cards` の可変参照を渡してるよ。
        shuffle_deck(&mut deck_cards); // <- こっちを使う！便利関数！
        // もし `logic/deck.rs` に `shuffle_deck` がなかったら、こっちの rand クレートの直接的なやり方でもOK！👍
        // let mut rng = thread_rng(); // 乱数生成器を用意して…
        // deck_cards.shuffle(&mut rng); // shuffle メソッドで直接シャッフル！
        println!("  🃏 デッキ作成 & シャッフル完了！ ({}枚)", deck_cards.len());

        // --- 2. 既存カードのお掃除タイム！ --- 🧹💨
        // もし前のゲームのカードが残ってたら、新しいゲームを始める前にお掃除しとかないとね！
        // `world.get_all_entities_with_component::<Card>()` で、現在 `Card` コンポーネントを持ってるエンティティのIDを全部もらう。
        // `.into_iter().collect::<Vec<_>>()` で、もらったIDのリストを一時的な `Vec<Entity>` (Entity の配列みたいなやつ) にコピーしてる。
        // なんでコピーするの？🤔 -> ループ (`for entity in ...`) の中で `world` の中身を削除 (`world.remove_component` とか) しようとすると、
        // Rustの所有権・借用ルールに引っかかってコンパイラに怒られちゃうことがあるんだ😭 (ループで `world` を借りてるのに、中で `world` を書き換えようとするのは危ない！って)。
        // だから、先に削除対象のIDリストだけ安全な場所にコピーしておいて、そのコピーを使ってループすれば、`world` を安全に変更できるってわけ！頭いい！🧠✨
        let existing_card_entities: Vec<Entity> = world.get_all_entities_with_component::<Card>().into_iter().collect();
        if !existing_card_entities.is_empty() {
            println!("  🧹 既存のカードエンティティ {} 個のコンポーネントを削除します...", existing_card_entities.len());
            for entity in existing_card_entities {
                // エンティティから Card コンポーネントを削除！ データ部品をポイッ！🚮
                world.remove_component::<Card>(entity);
                // 同じように StackInfo コンポーネントも削除！
                world.remove_component::<StackInfo>(entity);
                // Position コンポーネントも削除！
                world.remove_component::<Position>(entity);
                // TODO: もしカードエンティティが他のコンポーネントを持つ可能性があるなら、エンティティ自体を消す (`world.destroy_entity(entity)`) のは危ないかも？
                //       でも、カードエンティティは基本的に Card, StackInfo, Position しか持たない想定なら、 destroy_entity の方がメモリ的にはキレイになるね！今回はコンポーネント削除でいくよ！👍
            }
            println!("  🧹 既存カードのコンポーネント削除完了。");
        } else {
            println!("  🧹 既存のカードエンティティはありませんでした。お掃除不要！✨");
        }

        // --- 3. カードを配るよ！ --- 🃏💨
        // `deck_cards.into_iter()` で、シャッフル済みのデッキからカードを1枚ずつ順番に取り出せるようにする「イテレータ」を作るよ。
        // `into_iter()` は元の `deck_cards` の所有権を完全に持っていくから、この後 `deck_cards` を直接使うことはできなくなる！注意！⚠️
        // (データを効率よく移動させるためのRustの仕組みだよ！)
        let mut card_iterator = deck_cards.into_iter();

        // --- 3a. 場札 (Tableau) に配る！ (7列あるやつね！) ---
        println!("  ⏳ 場札 (Tableau) にカードを配置中...");
        // `0..7` は、0から6までの連続した数字を表す「範囲 (Range)」だよ。`for` ループで使うと、0, 1, 2, 3, 4, 5, 6 って順番に処理できる！
        let mut total_tableau_cards = 0;
        for tableau_index in 0..7 { // 7つの列 (index 0 から 6) に対してループ
            // 各列に配るカードの枚数は `tableau_index + 1` 枚 (1列目は1枚, 2列目は2枚...)
            // 列ごとのカードのY座標を計算するために、その列でどれだけ下にずらすかのオフセット値を覚えておく変数。
            let mut current_y_offset = 0.0;
            for card_in_tableau_index in 0..(tableau_index + 1) { // 各列に必要な枚数だけループ
                // デッキ (イテレータ) からカードを1枚取り出す！ `.next()` はカードがあれば `Some(Card)`、もう無ければ `None` を返す `Option` 型。
                // `.expect()` は、もし `None` だったらカッコ内のメッセージを表示してプログラムを強制終了させる！💥
                // ここでは、デッキの枚数は足りてるはずだから、もし None が来たらそれはプログラムのバグ！ってことで `expect` を使ってるよ。
                let mut card: Card = card_iterator.next().expect("デッキにカードが足りません！(場札配置バグ！)");

                // 新しいエンティティ (このカードの実体) を World に誕生させる！ ✨🐣✨
                let entity: Entity = world.create_entity();

                // --- カードの状態と位置を決める！ ---
                // このカードが、その列の一番上 (手前) に置かれるカードかどうかをチェック。
                let is_top_card_in_pile = card_in_tableau_index == tableau_index;
                // 一番上のカードだけ表向き！👀 それ以外は裏向きのまま。
                if is_top_card_in_pile {
                    card.is_face_up = true; // Card 構造体の中身を書き換える！
                }

                // カードの画面上の位置 (Position) を計算するよ！ 座標は `config/layout.rs` の定数を使う！
                let pos_x = TABLEAU_START_X + tableau_index as f32 * TABLEAU_X_OFFSET;
                // Y座標は、同じ列の前のカードのオフセットに基づいて決まる。
                let pos_y = TABLEAU_START_Y + current_y_offset;
                // 次のカードのために、Y座標のずれ (オフセット) を更新する。
                // 表向きのカードは、下に積むときに大きくずらす (カードが見えるように)。裏向きは小さくずらす。
                current_y_offset += if is_top_card_in_pile { TABLEAU_Y_OFFSET_FACE_UP } else { TABLEAU_Y_OFFSET_FACE_DOWN };
                // 計算した座標で Position コンポーネントを作る！
                let position_component = Position { x: pos_x, y: pos_y };

                // --- コンポーネントをエンティティに追加！ ---✍️
                // これで、この `entity` がどんなカードで、どこにあって、どの位置に表示されるかが `World` に記録される！
                world.add_component(entity, card); // Card コンポーネントを追加 (さっき is_face_up を更新したやつ！)
                world.add_component(entity, StackInfo {
                    stack_type: StackType::Tableau(tableau_index), // 「場札の tableau_index 列目だよ！」
                    position_in_stack: card_in_tableau_index,      // 「その列の中で card_in_tableau_index 番目 (0が奥) だよ！」
                });
                world.add_component(entity, position_component); // Position コンポーネントを追加！

                // println!("    配置: {:?} を 場札[{}] の {}番目 に (表向き: {}, Pos: {:?})", world.get_component::<Card>(entity).unwrap(), tableau_index, card_in_tableau_index, is_top_card_in_pile, position_component);
                total_tableau_cards += 1;
            }
        }
        println!("  ✅ 場札への配置完了！ ({}枚配置)", total_tableau_cards);

        // --- 3b. 残りのカードを山札 (Stock) へ！ --- ⛰️
        println!("  ⏳ 山札 (Stock) にカードを配置中...");
        let mut stock_card_count = 0; // 山札に何枚入れたかカウント
        // `card_iterator` には、場札に配らなかった残りのカードが入ってるはず。
        // `enumerate()` を使うと、(インデックス, カード) のペアでループできるから、山札内の順番 (position_in_stack) を付けるのに便利！✨
        for (index_in_stock, card) in card_iterator.enumerate() {
            // 新しいエンティティを作成！
            let entity = world.create_entity();

            // 山札のカードは全部同じ位置に表示する想定。座標は `config/layout.rs` から。
            let position_component = Position { x: STOCK_POS_X, y: STOCK_POS_Y };

            // コンポーネントを追加！✍️
            // カード情報はそのまま (全部裏向きのはず！)
            world.add_component(entity, card);
            // 場所情報は「山札だよ！」って設定。
            world.add_component(entity, StackInfo {
                stack_type: StackType::Stock,
                position_in_stack: index_in_stock as u8, // enumerate のインデックスを順番として使う (u8に変換！)
            });
            // 位置情報も追加！
            world.add_component(entity, position_component);

            // println!("    配置: {:?} を 山札 の {}番目 に (Pos: {:?})", world.get_component::<Card>(entity).unwrap(), index_in_stock, position_component);
            stock_card_count += 1;
        }
        println!("  ✅ 山札への配置完了！ ({}枚配置)", stock_card_count);

        let total_cards_placed = total_tableau_cards + stock_card_count;
        if total_cards_placed == 52 {
            println!("🎉 合計 {} 枚のカードを正しく配置しました！ゲーム開始準備OK！", total_cards_placed);
        } else {
            // もし52枚じゃなかったら、何かロジックがおかしい！😱
            eprintln!("🚨 エラー！配置されたカードの合計が52枚ではありません！({})", total_cards_placed);
        }

        // --- 4. 空の組札 (Foundation) や捨て札 (Waste) の場所について --- 🤔
        // これらは最初はカードが無い「空っぽの場所」だよね。
        // ECSでは、こういう「場所」自体を表すエンティティを作ることもあるんだけど、
        // 今回はシンプルに、カードが無い場所はエンティティも無い、っていう状態にしておくね！
        // カード移動のルール (System) を作る時に、「移動先が Foundation で、そこにカードが無い場合は…」みたいに条件分岐すればOK！👍

        println!("✅ DealInitialCardsSystem: 実行完了！");
    }
}

// === ユニットテスト === ✨🧪✨
// このシステムがちゃんと動くかチェックするコードだよ！
// `cargo test` ってコマンドを打つと、この中の `#[test]` が付いた関数が実行されるんだ。
#[cfg(test)]
mod tests {
    // 親モジュール (このファイルの上部) のアイテム (`*`) と、テストで使う他のモジュールをインポート！
    use super::*;
    use crate::components::position::Position;
    use crate::components::card::{Rank, Suit};
    use std::collections::HashMap; // テスト結果の集計とかに使うかも？

    // `#[test]` アトリビュートが付いた関数が、個別のテストケースになるよ！
    #[test]
    fn test_initial_deal_creates_correct_setup() {
        println!("--- test_initial_deal_creates_correct_setup 開始 ---🧪");
        // --- 準備 (Arrange) ---
        // 1. テスト用のまっさらな World を作成！
        let mut world = World::new();

        // 2. このシステムが必要とするコンポーネントを World に登録！これを忘れるとパニックする！😱
        world.register_component::<Card>();
        world.register_component::<StackInfo>();
        world.register_component::<Position>();

        // 3. テスト対象のシステム (DealInitialCardsSystem) のインスタンスを作成！
        let deal_system = DealInitialCardsSystem::default();

        // --- 実行 (Act) ---
        // 4. システムを実行して、カードを配ってもらう！
        deal_system.execute(&mut world);
        println!("--- deal_system.execute() 完了、検証開始！---🔬");

        // --- 検証 (Assert) ---
        // 5. カードエンティティがちゃんと52個作られたか？
        let all_card_entities: Vec<Entity> = world.get_all_entities_with_component::<Card>().into_iter().collect();
        assert_eq!(all_card_entities.len(), 52, "カードエンティティの総数が52個であるべきですが、{}個でした", all_card_entities.len());
        println!("✔️ カード総数 (52): OK");

        // 6. 全てのカードエンティティが StackInfo と Position も持っているか？
        let stack_info_count = all_card_entities.iter().filter(|&&e| world.get_component::<StackInfo>(e).is_some()).count();
        assert_eq!(stack_info_count, 52, "StackInfoを持つカードエンティティが52個であるべきですが、{}個でした", stack_info_count);
        println!("✔️ StackInfo保有数 (52): OK");
        let position_count = all_card_entities.iter().filter(|&&e| world.get_component::<Position>(e).is_some()).count();
        assert_eq!(position_count, 52, "Positionを持つカードエンティティが52個であるべきですが、{}個でした", position_count);
        println!("✔️ Position保有数 (52): OK");


        // 7. 各スタックタイプのカード枚数を数える！
        let mut counts: HashMap<StackType, usize> = HashMap::new();
        let mut face_up_counts: HashMap<StackType, usize> = HashMap::new();
        let mut cards_data: HashMap<StackType, Vec<(u8, Card)>> = HashMap::new(); // (position_in_stack, Card)

        for entity in all_card_entities {
            // StackInfo と Card を取得 (存在することは上で確認済みなので unwrap しちゃう！)
            let stack_info = world.get_component::<StackInfo>(entity).unwrap();
            let card = world.get_component::<Card>(entity).unwrap();

            // StackType ごとの枚数をカウントアップ
            *counts.entry(stack_info.stack_type).or_insert(0) += 1;

            // 表向きのカード枚数もカウントアップ
            if card.is_face_up {
                *face_up_counts.entry(stack_info.stack_type).or_insert(0) += 1;
            }

            // デバッグ用にカード情報も保存 (stack_type -> Vec<(順番, カード情報)>)
            cards_data.entry(stack_info.stack_type).or_default().push((stack_info.position_in_stack, card.clone()));
        }

        // 8. 山札 (Stock) の枚数と状態を確認！
        let stock_count = counts.get(&StackType::Stock).copied().unwrap_or(0);
        assert_eq!(stock_count, 24, "山札 (Stock) のカード枚数が24枚であるべきですが、{}枚でした", stock_count);
        let stock_face_up_count = face_up_counts.get(&StackType::Stock).copied().unwrap_or(0);
        assert_eq!(stock_face_up_count, 0, "山札 (Stock) に表向きのカードがあってはいけませんが、{}枚ありました", stock_face_up_count);
        println!("✔️ 山札 (Stock) 枚数 (24) と向き (全部裏): OK");
        // 山札カードの順番 (position_in_stack) が 0 から 23 まで連続しているかチェック
        if let Some(stock_cards) = cards_data.get_mut(&StackType::Stock) {
            stock_cards.sort_by_key(|(pos, _)| *pos); // 順番でソート
            for (i, (pos, _)) in stock_cards.iter().enumerate() {
                assert_eq!(*pos as usize, i, "山札カードの position_in_stack が連続していません (期待: {}, 実際: {})", i, *pos);
            }
        } else {
            panic!("山札のカードデータが収集できませんでした！");
        }
        println!("✔️ 山札 (Stock) カード順序: OK");

        // 9. 場札 (Tableau) の枚数と状態を確認！
        let mut total_tableau_cards = 0;
        for i in 0..7 {
            let stack_type = StackType::Tableau(i);
            let pile_count = counts.get(&stack_type).copied().unwrap_or(0);
            let expected_pile_count = (i + 1) as usize;
            assert_eq!(pile_count, expected_pile_count, "場札{} のカード枚数が{}枚であるべきですが、{}枚でした", i, expected_pile_count, pile_count);

            let pile_face_up_count = face_up_counts.get(&stack_type).copied().unwrap_or(0);
            assert_eq!(pile_face_up_count, 1, "場札{} の表向きカードが1枚であるべきですが、{}枚でした", i, pile_face_up_count);

            // 場札カードの順番と向きを確認
            if let Some(pile_cards) = cards_data.get_mut(&stack_type) {
                pile_cards.sort_by_key(|(pos, _)| *pos); // 順番でソート
                for (j, (pos, card)) in pile_cards.iter().enumerate() {
                    assert_eq!(*pos as usize, j, "場札{} のカード順序が不正です (期待: {}, 実際: {})", i, j, *pos);
                    let should_be_face_up = j == expected_pile_count - 1; // 一番上(最後)のカードか？
                    assert_eq!(card.is_face_up, should_be_face_up, "場札{}[{}] の向きが不正です (期待: {}, 実際: {})", i, j, should_be_face_up, card.is_face_up);
                }
            } else {
                panic!("場札{} のカードデータが収集できませんでした！", i);
            }
            total_tableau_cards += pile_count;
        }
        assert_eq!(total_tableau_cards, 28, "場札の合計枚数が28枚であるべきですが、{}枚でした", total_tableau_cards);
        println!("✔️ 場札 (Tableau) 枚数 (計28) と各列の状態 (枚数/向き/順序): OK");

        // 10. Foundation と Waste のカードが存在しないことを確認
        assert!(counts.get(&StackType::Foundation(0)).is_none(), "Foundation(0) にカードがあってはいけません");
        assert!(counts.get(&StackType::Waste).is_none(), "Waste にカードがあってはいけません");
        println!("✔️ 組札 (Foundation) と捨て札 (Waste) が空: OK");

        println!("--- test_initial_deal_creates_correct_setup 完了 ---✅✨");
    }

    // TODO: エッジケースのテスト (World に既に変なデータがある場合とか？) も追加すると、もっと頑丈になるかも！
} 