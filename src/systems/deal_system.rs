// src/systems/deal_system.rs

// === 使うものを宣言するよ！ ===
// World: エンティティやコンポーネントを管理する世界の中心！🌍
// components モジュール: カード(Card)とか場所(StackInfo)とか、色々なデータ部品(コンポーネント)が入ってるよ。🃏📍
// card モジュール: 特にカードに関するもの (create_standard_deck 関数とか Suit, Rank 列挙型とか)
// stack モジュール: カードを置く場所の種類 (StackType) とか、場所情報 (StackInfo)
// system モジュール: システムの基本となるトレイト (今は使わないけど、将来的に使うかも！)
// entity モジュール: エンティティ (ゲーム世界のモノを表すID)
// rand クレート: カードをシャッフルするのに使うよ！🎲 (さっき追加したやつ！)
use crate::world::World;
use crate::components::{card::{self, Card}, stack::{StackInfo, StackType}};
// use crate::system::System; // 削除: 今は直接使わないのでコメントアウトまたは削除
use crate::entity::Entity;
use rand::seq::SliceRandom; // Vec (配列みたいなもの) の要素をシャッフルする機能 (shuffle) を使うために必要！
use rand::thread_rng; // OS が提供する安全な乱数生成器を使うために必要！

// === 初期カード配置システム！ ===
// ゲーム開始時に、山札と7つの場札にカードを配る役割を担うシステムだよ。
// 構造体 (struct) は、関連するデータをまとめるためのもの。ここでは DealInitialCardsSystem という名前の空の構造体を作ってる。
//メソッド (処理) を関連付けるために構造体を使ってる感じだね！
#[derive(Default)] // `DealInitialCardsSystem::default()` で簡単にインスタンスを作れるようにするおまじない ✨
pub struct DealInitialCardsSystem;

// DealInitialCardsSystem にメソッド (関数みたいなもの) を実装していくよ！
impl DealInitialCardsSystem {
    /// ゲームの初期カード配置を実行する関数だよ！ 🎉
    ///
    /// # 引数
    /// - `world`: 可変参照 (&mut World)。World の中身 (エンティティやコンポーネント) を変更する必要があるから `&mut` が付いてるよ。
    ///
    /// # 処理の流れ
    /// 1. 新しいカードデッキ (52枚、全部裏向き) を作る。
    /// 2. デッキをシャッフルする。
    /// 3. 既存のカードエンティティがあれば削除する (念のためのお掃除🧹)。
    /// 4. シャッフルされたデッキからカードを取り出して、クロンダイクのルールに従って配置していく。
    ///    - 山札 (Stock): 24枚、全部裏向き。
    ///    - 場札 (Tableau): 7列。1列目は1枚(表向き)、2列目は2枚(一番上だけ表向き)、... 7列目は7枚(一番上だけ表向き)。
    /// 5. 各カードエンティティに `Card` コンポーネントと `StackInfo` コンポーネントを追加する。
    pub fn execute(&self, world: &mut World) {
        // --- 1. デッキの準備 ---
        // card モジュールにある create_standard_deck 関数を呼び出して、52枚のカードデッキを作るよ。
        // `mut` を付けてるから、後でシャッフル (中身の順番を変える) できる！
        let mut deck = card::create_standard_deck();
        println!("🃏 デッキ作成完了！ ({}枚)", deck.len()); // デバッグ用に枚数をログ出力！

        // --- 2. シャッフル ---
        // `thread_rng()` で乱数生成器を取得して、`shuffle` メソッドでデッキの順番をランダムに入れ替えるよ！
        // これで毎回違うゲームが楽しめるね！🥳
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);
        println!("🎲 デッキをシャッフルしました！");

        // --- 3. 既存カードのクリア (念のため) ---
        // ゲーム開始時に前のゲームのカードが残ってたら大変だから、先に掃除しておくよ！🧹
        // `world.query_entities_with_component::<Card>()` で Card コンポーネントを持つ全てのエンティティIDを取得する。
        // `collect::<Vec<_>>()` で取得したIDを一時的な Vec (配列みたいなの) に集める。
        //   -> なぜ一時的な Vec に？: world の中身をループしながら world を変更しようとすると、Rust に怒られちゃう (借用規則違反)。
        //      なので、先にIDだけ集めておいて、そのIDリストを使ってループするんだ。賢い！🧠
        let existing_card_entities: Vec<Entity> = world.get_all_entities_with_component::<Card>().into_iter().collect();
        if !existing_card_entities.is_empty() {
            println!("🧹 既存のカードエンティティ {} 個を削除します...", existing_card_entities.len());
            for entity in existing_card_entities {
                // world から Card コンポーネントを削除
                world.remove_component::<Card>(entity);
                // Card に関連する他のコンポーネント (StackInfo や Position もあれば) も削除するのが親切かも。
                // 今は StackInfo だけ削除しておくね。Position はまだ使ってないから大丈夫かな？🤔
                world.remove_component::<StackInfo>(entity);
                // 本当はエンティティ自体を削除 (world.delete_entity(entity)) したいけど、
                // 他のコンポーネントがまだ付いてる可能性もあるから、一旦コンポーネント削除だけに留めておくね。
            }
            println!("🧹 既存カードの削除完了。");
        }


        // --- 4. カードの配置 ---
        // `deck.into_iter()` でデッキのカードを1枚ずつ取り出せるようにするよ。
        // `into_iter()` は元の `deck` の所有権を奪うから、もう `deck` は使えなくなる。注意！⚠️
        let mut card_iterator = deck.into_iter();

        // 配置するカードのインデックス (何枚目のカードか) を追跡するカウンター
        let mut card_index = 0;

        // --- 4a. 場札 (Tableau) への配置 ---
        println!("⏳ 場札 (Tableau) にカードを配置中...");
        // 7つの場札の列を作るよ (0番目から6番目まで)。
        for tableau_index in 0..7 { // 0 から 6 までの数字を順番に tableau_index に入れて繰り返す
            // 各列に配置するカード枚数は (列番号 + 1) 枚。
            for card_in_tableau in 0..(tableau_index + 1) {
                // デッキからカードを1枚取り出す。
                // `next()` は Option<Card> を返す (カードがあれば Some(card), なければ None)。
                // `expect()` は None の場合にプログラムをクラッシュさせる。ここではデッキが足りないことは無いはずだから使う！💥
                let mut card = card_iterator.next().expect("デッキにカードが足りません！(場札配置中)");

                // エンティティ (カードの実体) を World に作成する。
                // `create_entity()` は新しいユニークなID (Entity) を返す。
                let entity = world.create_entity();

                // その列の一番上のカードだけ表向きにするよ！👀
                let is_face_up = card_in_tableau == tableau_index;
                if is_face_up {
                    card.is_face_up = true; // カードの is_face_up フラグを true に更新！
                }

                // Card コンポーネントをエンティティに追加！これで「このエンティティはこういうカードだ」とわかる。
                world.add_component(entity, card);

                // StackInfo コンポーネントも追加！これで「このカードはどこにあるか」がわかる。
                world.add_component(entity, StackInfo {
                    // `StackType::Tableau(tableau_index)` で「場札の〇番目の列」という場所を指定。
                    stack_type: StackType::Tableau(tableau_index),
                    // `order` はその場札列の中での順番 (0が一番奥/下)。
                    position_in_stack: card_in_tableau,
                });

                // デバッグ用にログ出力
                // println!("  配置: {:?} を 場札[{}] の {}番目 に (表向き: {})", world.get_component::<Card>(entity).unwrap(), tableau_index, card_in_tableau, is_face_up);

                card_index += 1; // 配置したカード枚数をカウントアップ
            }
        }
        println!("✅ 場札への配置完了！ ({}枚配置)", card_index);

        // --- 4b. 山札 (Stock) への配置 ---
        // 残りのカードを全部、山札に裏向きで置くよ。
        println!("⏳ 山札 (Stock) にカードを配置中...");
        let mut stock_order = 0; // 山札の中での順番カウンター
        // `card_iterator` に残っているカードをすべてループで処理する。
        for card in card_iterator { // `card` は最初から裏向き (`is_face_up: false`) のはず！
            // 新しいエンティティを作成
            let entity = world.create_entity();
            // Card コンポーネントを追加 (中身は card 変数そのもの)
            world.add_component(entity, card);
            // StackInfo コンポーネントを追加
            world.add_component(entity, StackInfo {
                // 場所は `StackType::Stock` (山札)
                stack_type: StackType::Stock,
                // 順番は `stock_order`
                position_in_stack: stock_order,
            });
            // デバッグ用にログ出力
            // println!("  配置: {:?} を 山札 の {}番目 に", world.get_component::<Card>(entity).unwrap(), stock_order);
            stock_order += 1; // 順番カウンターを増やす
            card_index += 1; // 全体の配置枚数カウンターも増やす
        }
        println!("✅ 山札への配置完了！ ({}枚配置)", stock_order);
        println!("🎉 合計 {} 枚のカードを配置しました！", card_index);

        // --- 5. ファンデーションとウェスト用の空スタック情報も作る？ ---
        // クロンダイクには、カードを最終的に移動させる4つの「上がり札置き場 (Foundation)」と、
        // 山札からめくったカードを一時的に置く「捨て札置き場 (Waste)」があるよね。
        // これらは最初は空だけど、「ここがFoundationだよ」「ここがWasteだよ」という情報だけは
        // World に持たせておくと、後でカード移動のルールを実装する時に便利かも？🤔
        // 例えば、特定のエンティティを作って、それに StackInfo だけ付けておくとか？
        // 今回はカード配置がメインだから、一旦省略するね！後で必要になったら追加しよう！👍
    }
}


// --- テストコード ---
// `#[cfg(test)]` アトリビュートは、`cargo test` コマンドを実行した時だけコンパイルされるコードブロックを示すよ。
#[cfg(test)]
mod tests {
    // `use super::*;` で、この test モジュールが属している親モジュール (このファイルの上部) で定義されているもの
    // (DealInitialCardsSystem, World, Card, StackInfo, StackType など) を全部使えるようにするよ！便利！🌟
    use super::*;
    use crate::components::card::{Rank, Suit}; // テストで具体的なカードを確認するために Rank と Suit も使うよ

    // `#[test]` アトリビュートが付いた関数が、個別のテストケースになるよ。
    #[test]
    fn test_initial_deal() {
        // --- 準備 ---
        // 1. テスト用の World インスタンスを作成
        let mut world = World::new();
        // 2. 必要なコンポーネントを World に登録 (実際の GameApp::new でもやってるはず！)
        //    これがないと add_component とか get_component が失敗しちゃう！😱
        world.register_component::<Card>();
        world.register_component::<StackInfo>();
        // Position とか Player とかは、このテストでは直接使わないけど、登録しておいても害はないかな。
        // world.register_component::<Position>();
        // world.register_component::<Player>();

        // 3. テスト対象のシステム (DealInitialCardsSystem) のインスタンスを作成
        let deal_system = DealInitialCardsSystem::default(); // #[derive(Default)] のおかげで簡単！

        // --- 実行 ---
        // 4. システムの execute メソッドを実行して、カードを配置してもらう！
        println!("--- test_initial_deal 開始 ---");
        deal_system.execute(&mut world);
        println!("--- deal_system.execute() 完了 ---");

        // --- 検証 ---
        // 5. 配置されたカードの枚数を確認！ 合計52枚のはず！
        let all_card_entities: Vec<Entity> = world.get_all_entities_with_component::<Card>().collect();
        assert_eq!(all_card_entities.len(), 52, "配置されたカードの総数が52枚ではありません！");
        println!("✔️ カード総数チェックOK ({}枚)", all_card_entities.len());

        // 6. 各スタックタイプごとの枚数と状態を確認！
        let mut stock_count = 0;
        let mut tableau_counts = [0; 7]; // 7つの場札列の枚数をカウントする配列
        let mut foundation_count = 0; // 上がり札 (今回は配置されないはず)
        let mut waste_count = 0;      // 捨て札 (今回は配置されないはず)

        let mut tableau_face_up_counts = [0; 7]; // 各場札列の表向きカード枚数

        // 配置された全カードエンティティをループして、StackInfo を確認するよ
        for entity in all_card_entities {
            // Card コンポーネントを取得 (これは存在するはず！)
            let card = world.get_component::<Card>(entity)
                .expect("Card コンポーネントが見つかりません！");
            // StackInfo コンポーネントを取得 (これも存在するはず！)
            let stack_info = world.get_component::<StackInfo>(entity)
                .expect("StackInfo コンポーネントが見つかりません！");

            // StackType によってカウントを振り分ける
            match stack_info.stack_type {
                StackType::Stock => {
                    stock_count += 1;
                    // 山札のカードは全部裏向きのはず！
                    assert!(!card.is_face_up, "山札に表向きのカードがあります！{:?}", card);
                }
                StackType::Tableau(index) => {
                    // index が 0..7 の範囲内かチェック (念のため)
                    assert!(index < 7, "無効な Tableau インデックスです: {}", index);
                    tableau_counts[index] += 1; // その列のカウントを増やす
                    // 場札の一番上のカード (position_in_stack == index) だけが表向きのはず！
                    if stack_info.position_in_stack == index {
                        assert!(card.is_face_up, "場札の[{}]番目({}) が裏向きです！{:?}", index, stack_info.position_in_stack, card);
                        tableau_face_up_counts[index] += 1;
                    } else {
                        assert!(!card.is_face_up, "場札の[{}]番目({}) が表向きです！{:?}", index, stack_info.position_in_stack, card);
                    }
                    // position_in_stack が正しい範囲 (0 <= position_in_stack <= index) かチェック
                    assert!(stack_info.position_in_stack <= index, "Tableau[{}] の position_in_stack が不正です: {}", index, stack_info.position_in_stack);
                }
                StackType::Foundation(_) => foundation_count += 1,
                StackType::Waste => waste_count += 1,
            }
        }

        // --- 結果の確認 ---
        // 山札 (Stock) の枚数チェック (52 - (1+2+3+4+5+6+7)) = 52 - 28 = 24 枚
        assert_eq!(stock_count, 24, "山札のカード枚数が24枚ではありません！ ({})", stock_count);
        println!("✔️ 山札の枚数チェックOK ({})", stock_count);

        // 場札 (Tableau) の枚数チェック
        for i in 0..7 {
            assert_eq!(tableau_counts[i], i + 1, "場札[{}]の枚数が{}枚ではありません！ ({})", i, i + 1, tableau_counts[i]);
            assert_eq!(tableau_face_up_counts[i], 1, "場札[{}]の表向きカードが1枚ではありません！ ({})", i, tableau_face_up_counts[i]);
        }
        println!("✔️ 場札の枚数チェックOK (合計 {}枚)", tableau_counts.iter().sum::<usize>());
        println!("✔️ 場札の表向きカードチェックOK");

        // Foundation と Waste にはカードがないはず
        assert_eq!(foundation_count, 0, "Foundation にカードが配置されています！ ({})", foundation_count);
        assert_eq!(waste_count, 0, "Waste にカードが配置されています！ ({})", waste_count);
        println!("✔️ Foundation/Waste が空であることのチェックOK");

        // 7. カードの重複がないかチェック (念のため)
        //    配置された全カードの (Suit, Rank) の組み合わせを HashSet に入れて、重複がないか確認する。
        use std::collections::HashSet;
        let mut unique_cards = HashSet::new();
        let all_cards: Vec<Card> = world.query_components::<Card>().cloned().collect(); // 全Cardコンポーネントを取得
        let mut duplicate_found = false;
        for card in all_cards {
            if !unique_cards.insert((card.suit, card.rank)) {
                println!("重複カード発見！ Suit: {:?}, Rank: {:?}", card.suit, card.rank);
                duplicate_found = true;
            }
        }
        assert!(!duplicate_found, "配置されたカードに重複が見つかりました！");
        println!("✔️ カードの重複チェックOK");


        println!("✅✅✅ test_initial_deal 成功！ 🎉🎉🎉");
    }
} 