# マルチプレイソリティア開発計画書 🚀 (詳細版)

これは、Rust + WASM + 自作ECSで作るマルチプレイソリティアゲームの開発計画チェックリストだよ！✅

## フェーズ 1: プロジェクトセットアップ & ECS基盤構築 🏗️ (**完了！**)

- [x] Rust WASM プロジェクトの基本セットアップ (`Cargo.toml`, `wasm-pack`, ビルド設定)
- [x] `wasm-bindgen` の設定と JS 連携の準備
- [x] ログ出力 (`console.log`, `console.error`) とパニックハンドリングの設定 (`console_error_panic_hook`)
- [x] ECSコア実装: `Entity` 型の定義 (`entity.rs`)
- [x] ECSコア実装: `Component` の概念定義 (`component.rs`)
- [x] ECSコア実装: `World` 構造体の実装 (`world.rs`) - `Box<dyn Any>` と関数ポインタを使った型ごとのストレージ管理！
- [x] ECSコア実装: `World` エンティティ作成・削除 - `create_entity`, `create_entity_with_id`, `destroy_entity` (コンポーネント自動削除付き)
- [x] ECSコア実装: `World` コンポーネント追加・削除・取得 - `register_component`, `add_component`, `get_component`, `get_component_mut`, `remove_component`
- [x] ECSコア実装: `World` 特定コンポーネントを持つエンティティのクエリ - `get_all_entities_with_component`
- [x] ECSコア実装: `System` の概念定義 (`system.rs` トレイト)
- [x] 基本コンポーネント定義: `Position` (`components/position.rs`)
- [x] 基本コンポーネント定義: `Card`, `Suit`, `Rank` (`components/card.rs`)
- [x] 基本コンポーネント定義: `Player` (`components/player.rs`)
- [x] 基本コンポーネント定義: `StackInfo`, `StackType` (`components/stack.rs`)
- [x] 基本コンポーネント定義: `GameState`, `GameStatus` (`components/game_state.rs`)
- [x] 基本コンポーネント定義: `DraggingInfo` (`components/dragging_info.rs`)

## フェーズ 2: 基本ゲームロジック & 画面表示 (クライアント単体) 🃏🖼️ (**Canvas描画 完了！**)

- [x] ゲーム初期化: カードデッキ生成ロジック (`logic/deck.rs`)
- [x] ゲーム初期化: 初期カード配置 (ディール) システムの実装 (`systems/deal_system.rs`)
- [x] レンダリング準備: `GameApp` (WASM側メイン構造体) の実装 (`app/game_app.rs`) - モジュール分割済み！
- [x] レンダリング準備: `GameApp` から World の状態を取得するデバッグ用メソッド (`get_world_state_json`)
- [x] レンダリング準備: JS側でWASMをロードし、`GameApp` インスタンスを作成 (`bootstrap.js`)
- [x] レンダリング準備: 基本的なHTML/CSS構造の作成 (`index.html` に `<canvas>` を設置)
- [x] レンダリング準備: JSから定期的にWASMのゲーム状態を取得し、描画関数を呼び出す (`requestAnimationFrame` など)
- [x] レンダリング実装(Rust/Wasm): **Canvas** を使った基本的なカード描画 (`app/renderer.rs`) - 矩形+テキスト描画、描画順ソート、空スタックのプレースホルダー描画 **完了！** 🖌️✅

## フェーズ 3: ネットワーク実装 🌐🤝 (**基本完了！連携テスト待ち**)

- [x] 通信プロトコル定義: `ClientMessage`, `ServerMessage`, 共有データ構造 (`protocol.rs`) - `PositionData` も含む！
- [x] 通信プロトコル定義: `serde` を使ったシリアライズ/デシリアライズ設定
- [x] クライアント側ネットワーク処理: WebSocket接続マネージャー (`NetworkManager` in `network.rs`) - `Arc<Mutex<>>` で状態共有！
- [x] クライアント側ネットワーク処理: 接続・切断・エラー・メッセージ受信コールバック設定 (`network.rs`)
- [x] クライアント側ネットワーク処理: メッセージ送信・受信キュー (`network.rs`, `app/network_handler.rs`)
- [x] クライアント側ネットワーク処理: 接続状態管理 (`ConnectionStatus` in `network.rs`)
- [ ] クライアント側ネットワーク処理: (任意) 切断時の自動再接続ロジック
- [x] クライアント側ネットワーク処理: `GameApp` から `NetworkManager` を利用 (`app/game_app.rs`, `app/network_handler.rs`)
- [x] クライアント側ネットワーク処理: `connect()` メソッド (`app/network_handler.rs`)
- [x] クライアント側ネットワーク処理: `send_message()` ヘルパーメソッド (`app/network_handler.rs`)
- [x] クライアント側ネットワーク処理: `send_join_game()`, `send_make_move()` などWASM公開メソッド (`app/game_app.rs` & `app/network_handler.rs`)
- [x] クライアント側ネットワーク処理: `send_initial_state()` メソッド (`app/init_handler.rs`) - クライアントから初期状態を送る！
- [x] クライアント側ネットワーク処理: `process_received_messages()` で受信キューを処理 (`app/network_handler.rs`)
- [x] クライアント側ネットワーク処理: `apply_game_state()` で受信データ (`GameStateData`) を `World` に反映 (`app/state_handler.rs`) - プレイヤー・カードのコンポーネントクリア＆再適用 **完了！** (サーバー連携テスト待ち！)
- [x] サーバー側(JS): WebSocketサーバー起動 (`localhost:8101`, `server/ws_server.js`)
- [x] サーバー側(JS): クライアント接続/切断管理
- [x] サーバー側(JS): `JoinGame` メッセージ受信処理と `GameJoined` 応答実装
- [x] サーバー側(JS): プレイヤーリストの管理と `PlayerJoined`/`PlayerLeft` のブロードキャスト実装
- [x] サーバー側(JS): ゲーム状態管理: カード情報 (`gameState.cards`) の保持と初期化 (`ProvideInitialState` 受信時)
- [x] サーバー側(JS): `MakeMove` メッセージ受信処理
- [x] サーバー側(JS): カード移動を `gameState.cards` に反映させるロジック
- [x] サーバー側(JS): ゲーム状態の変更を全クライアントに通知する `GameStateUpdate` メッセージのブロードキャスト実装

## フェーズ 4: ソリティアのルールとインタラクション実装 🃏👆 (**ルール実装ほぼ完了！ UIインタラクションが最重要課題！**)

- [x] ルール実装(Rust): `StackType` ごとのカード移動可否判定ヘルパー関数 (`logic/rules.rs`) - `get_top_card_entity` ヘルパー **実装完了！**
- [x] ルール実装(Rust): タブローからタブローへの移動ルール実装 (`logic/rules.rs` の `can_move_to_tableau`) - **World 依存版 & テスト完了！**
- [x] ルール実装(Rust): タブローからファンデーションへの移動ルール実装 (`logic/rules.rs` の `can_move_to_foundation`) - **World 依存版 & テスト完了！**
- [x] ルール実装(Rust): ストックからウェストへの移動ルール実装 (`logic/rules.rs` の `can_deal_from_stock`, `can_reset_stock_from_waste`) - (World状態を見ない単純版) **実装完了！**
- [x] ルール実装(Rust): ウェストからタブロー/ファンデーションへの移動ルール実装 (`logic/rules.rs` の `can_move_from_waste_to_tableau`, `can_move_from_waste_to_foundation`) - **World 依存版完了！** (呼び出し側テストでカバー) **実装完了！**
- [x] ルール実装(Rust): カード自動移動ロジック (`logic/auto_move.rs` の `find_automatic_foundation_move`) - **World 依存版 & テスト完了！** ✨🤖✅
- [x] ルール実装(Rust): ゲームクリア判定ロジック (`logic/rules.rs` の `check_win_condition`) - システム (`systems/win_condition_system.rs`) 側で World の状態を見て **実装＆テスト完了！** 🎉

// --- UIインタラクション(JS & Rust) - Canvas ベース ---
- [x] UIインタラクション(JS): Canvas クリックイベントの検知とログ出力 (JS側)
- [x] UIインタラクション(Rust/JS): クリック座標からカード/スタックを特定するロジック 🔥最重要🔥 (Rust側判定ロジック実装 & JS連携 **完了！**)
    - [x] (Rust) クリック座標(x, y)を受け取り、Worldの状態(`Position`, `StackInfo`)とレイアウト情報(`config/layout.rs`)を元に、クリックされたカードの`Entity`または空きスタックの`StackType`を返す関数を `app/event_handler.rs` に実装 (`find_clicked_element`)。
    - [x] (JS) Rust側の判定関数 (`GameApp::handle_click`) を呼び出し、結果を取得する。(ログ出力で確認済み)
- [x] UIインタラクション(JS): ダブルクリックイベントの検知 (Canvas 上で) (ログ出力まで実装完了！)
- [x] UIインタラクション(JS): ダブルクリック時にRust側の自動移動ロジック (`GameApp::handle_double_click`) を呼び出す。
- [x] UIインタラクション(JS): ドラッグ開始 (`mousedown`): クリック判定ロジックを利用して対象カード特定、Rust側に通知して `DraggingInfo` コンポーネントを追加させる。
- [x] UIインタラクション(JS): ドラッグ終了 (`mouseup`): ドロップ位置から移動先スタックを判定 (クリック判定ロジック応用)、Rust側に通知。
- [x] UIインタラクション(Rust): ドロップ時のルールチェックと移動実行
    - [x] (Rust) `logic/rules.rs` の関数で移動可否を判定。
    - [x] (Rust) 移動可能なら `NetworkManager` を直接使ってサーバーに通知。 (旧: `app/network_handler.rs` の `send_make_move`)
    - [x] (Rust) `DraggingInfo` コンポーネントを削除。
    - [x] (Rust) 移動成功時にカードの `Position` と `StackInfo` を更新。
    - [x] (Rust) 移動成功時に元の Tableau スタックで隠れていたカードを表向きにする。
    - [x] (Rust) 移動失敗時にカードの `Position` を元に戻す。

// --- 状態更新と表示 (Canvas) ---
- [x] 状態更新(Rust): サーバーからの `GameStateUpdate` 受信時に `apply_game_state` を呼び出す (`app/network_handler.rs`)
- [x] 状態更新(Rust/JS): `apply_game_state` (Rust) 後、`render_game_rust` (Rust) を呼び出して画面を更新。
- [x] 状態更新(JS): サーバーから `MoveRejected` を受け取った場合に、ユーザーにフィードバックを表示 (例: アラート表示、カードを元の位置に戻すアニメーションなど)。

// --- その他 ---
- **[ ] 不要コード調査:** `systems/move_card_system.rs` が現状の自作ECSで機能していない可能性。役割（サーバー側で処理？）を確認し、不要なら削除、必要なら修正する。🚨

## フェーズ 5: マルチプレイヤー同期と仕上げ ✨💅 (ほぼ未着手)

- [ ] プレイヤー表示(JS): 画面上に参加プレイヤーリストを表示するエリアを作成
- [ ] プレイヤー表示(JS): `PlayerJoined` / `PlayerLeft` メッセージ受信時にリストを更新
- [ ] ターン表示(JS/Rust): 現在誰のターンかを表示 (もしターン制なら)
- [ ] アニメーション(JS/Rust): カード移動やシャッフルなどのアニメーションを追加
- [ ] サウンド(JS): カード操作時などの効果音を追加
- [ ] UI改善(JS/CSS): 見た目や操作性を向上させる
- [ ] エラーハンドリング強化(Rust/JS): ネットワークエラーや予期せぬ状態への対応を強化
- [ ] テスト拡充(Rust/JS): 特に結合テストや E2E テストを追加

// --- (以下、変更なし) ---