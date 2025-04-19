# マルチプレイソリティア開発計画書 🚀 (詳細版)

これは、Rust + WASM + 自作ECSで作るマルチプレイソリティアゲームの開発計画チェックリストだよ！✅

## フェーズ 1: プロジェクトセットアップ & ECS基盤構築 🏗️ (**一部課題あり**)

- [x] Rust WASM プロジェクトの基本セットアップ (`Cargo.toml`, `wasm-pack`, ビルド設定)
- [x] `wasm-bindgen` の設定と JS 連携の準備
- [x] ログ出力 (`console.log`) とパニックハンドリングの設定 (`console_error_panic_hook`)
- [x] ECSコア実装: `Entity` 型の定義 (`entity.rs`)
- [x] ECSコア実装: `Component` の概念定義 (`component.rs`) - **基本的な概念定義完了！** ✅
- [x] ECSコア実装: `World` 構造体の実装 (`world.rs`) - **基本的な構造体と主要メソッド実装完了！** 🎉
- [x] ECSコア実装: `World` エンティティ作成・削除 - `create_entity`, `create_entity_with_id`, `destroy_entity` 実装済。 **`destroy_entity` 時のコンポーネント自動削除も実装済み！** ✅🧹
- [x] ECSコア実装: `World` コンポーネント追加・削除・取得 - `register_component`, `add_component`, `get_component`, `get_component_mut`, `remove_component` 実装完了！ ✅
- [x] ECSコア実装: `World` 特定コンポーネントを持つエンティティのクエリ - `get_all_entities_with_component` 実装完了！ ✅
- [x] ECSコア実装: `System` の概念定義 (`system.rs` トレイト)
- [x] 基本コンポーネント定義: `Position` - **定義は `components/position.rs` に統一済み！** ✅
- [x] 基本コンポーネント定義: `Card` - **定義は `components/card.rs` に統一済み！** ✅
- [x] 基本コンポーネント定義: `Player` - **定義は `components/player.rs` に統一済み！** ✅
- [x] 基本コンポーネント定義: `StackInfo` - **定義は `components/stack.rs` に統一済み！** ✅
- [x] 基本コンポーネント定義: `GameState` - **定義は `components/game_state.rs` に統一済み！** ✅

## フェーズ 2: 基本ゲームロジック & 画面表示 (クライアント単体) 🃏🖼️ (Canvasへ移行中！)

- [x] ゲーム初期化: カードデッキ生成ロジック (`logic/deck.rs` の `create_standard_deck`) - **定義は `logic/deck.rs` に統一済み！** ✅
- [x] ゲーム初期化: 初期カード配置 (ディール) システムの実装 (`systems/deal_system.rs`) - **実装完了！** ✅
- [x] レンダリング準備: `GameApp` (WASM側メイン構造体) の実装 (`app/game_app.rs`) ✅
- [x] レンダリング準備: `GameApp` から World の状態を取得するメソッド実装 (`app/game_app.rs` の `get_world_state_json`) - **実装完了！ (`GameStateData` をJSON化, 詳細コメント付)** ✅
- [x] レンダリング準備: JS側でWASMをロードし、`GameApp` インスタンスを作成 (`bootstrap.js`) ✅
- [x] レンダリング準備: 基本的なHTML/CSS構造の作成 (`index.html` に `<canvas>` を設置！✅)
- [x] レンダリング準備: JSから定期的にWASMのゲーム状態を取得し、コンソール等に表示 (`requestAnimationFrame` で `render_game_rust` を呼ぶ形) ✅
- [x] レンダリング実装(Rust/Wasm): **Canvas** を使った基本的なカード描画 (`app/renderer.rs` の `render_game_rust`) - **基本描画ロジック実装完了！(矩形+テキスト)** 🖌️✅

## フェーズ 3: ネットワーク実装 🌐🤝 (ほぼ完了！🎉)

- [x] 通信プロトコル定義: `ClientMessage` (`protocol.rs`) ✅
- [x] 通信プロトコル定義: `ServerMessage` (`protocol.rs`) ✅
- [x] 通信プロトコル定義: メッセージ共通で使うデータ構造 (`protocol.rs`) ✅
- [x] 通信プロトコル定義: `serde` を使ったシリアライズ/デシリアライズ設定 ✅
- [x] クライアント側ネットワーク処理: WebSocket接続マネージャー (`NetworkManager` in `network.rs`) ✅
- [x] クライアント側ネットワーク処理: 接続・切断処理 ✅
- [x] クライアント側ネットワーク処理: メッセージ送信・受信キュー ✅
- [x] クライアント側ネットワーク処理: 接続状態管理 (`ConnectionStatus`) ✅
- [ ] クライアント側ネットワーク処理: (任意) 切断時の自動再接続ロジック
- [x] クライアント側ネットワーク処理: `GameApp` から `NetworkManager` を利用 (`app/game_app.rs`) ✅
- [x] クライアント側ネットワーク処理: `connect()` メソッド (`app/network_handler.rs`) ✅
- [x] クライアント側ネットワーク処理: `send_message()` ヘルパーメソッド (`network.rs`) ✅
- [x] クライアント側ネットワーク処理: `send_join_game()`, `send_make_move()` などWASM公開メソッド (`app/game_app.rs` & `app/network_handler.rs`) ✅
- [x] クライアント側ネットワーク処理: `send_initial_state()` メソッド (`app/init_handler.rs`) ✅
- [x] クライアント側ネットワーク処理: `process_received_messages()` で受信キューを処理 (`app/network_handler.rs`) ✅
- [/] クライアント側ネットワーク処理: `apply_game_state()` で受信データ (`GameStateData`) を `World` に反映 (`app/state_handler.rs`) - **`World` 実装済、次は修正・動作確認！** 🛠️
- [x] サーバー側(JS): WebSocketサーバー起動 (`localhost:8101`, `server/ws_server.js`) ✅
- [x] サーバー側(JS): クライアント接続/切断管理 ✅
- [x] サーバー側(JS): `JoinGame` メッセージ受信処理と `GameJoined` 応答実装 ✅
- [x] サーバー側(JS): プレイヤーリストの管理と `PlayerJoined`/`PlayerLeft` のブロードキャスト実装 ✅
- [x] サーバー側(JS): ゲーム状態管理: カード情報 (`gameState.cards`) の保持と初期化 (`ProvideInitialState` 受信時) ✅
- [x] サーバー側(JS): `MakeMove` メッセージ受信処理 ✅
- [x] サーバー側(JS): カード移動を `gameState.cards` に反映させるロジック ✅
- [x] サーバー側(JS): ゲーム状態の変更を全クライアントに通知する `GameStateUpdate` メッセージのブロードキャスト実装 ✅

## フェーズ 4: ソリティアのルールとインタラクション実装 🃏👆 (Canvasへ移行中！一部進行可能に)

- [/] ルール実装(Rust): `StackType` ごとのカード移動可否判定ヘルパー関数 (`logic/rules.rs`) - **`get_top_card_entity` ヘルパー実装＆バグ修正完了！ World 依存部分の基礎OK！** 🛠️✅🐞✨
- [/] ルール実装(Rust): タブローからタブローへの移動ルール実装 (`logic/rules.rs` の `can_move_to_tableau`) - **World 依存版に改造＆コメント追加完了！テストはTODO！** 🛠️✅📝
- [/] ルール実装(Rust): タブローからファンデーションへの移動ルール実装 (`logic/rules.rs` の `can_move_to_foundation`) - **World 参照版 (`_world`) を実装！ 要テスト/連携！** 🛠️✅
- [x] ルール実装(Rust): ストックからウェストへの移動ルール実装 (`logic/rules.rs` の `can_deal_from_stock`, `can_reset_stock_from_waste`) ✅ (World状態を見ない単純版)
- [/] ルール実装(Rust): ウェストからタブロー/ファンデーションへの移動ルール実装 (`logic/rules.rs` の `can_move_from_waste_to_tableau`, `can_move_from_waste_to_foundation`) - **World 参照版 (`_world`) を実装！ 要テスト/連携！** 🛠️✅
- [/] ルール実装(Rust): (任意) カード自動移動ロジック (`logic/auto_move.rs` の `find_automatic_foundation_move`) - **コメントアウト中。World 版への修正が必要！** 🛠️
- [/] ルール実装(Rust): ゲームクリア判定ロジック (`logic/rules.rs` の `check_win_condition`) ✅ (`WinConditionSystem` は **`World` 実装済、次は修正・動作確認！** 🛠️)

// --- UIインタラクション(JS) - Canvas ベース ---
- [ ] UIインタラクション(JS): Canvas クリックイベントの検知とログ出力
- [ ] UIインタラクション(JS): クリック座標からカード/スタックを特定するロジック (Rust側で実装中？ 要確認)
- [ ] UIインタラクション(JS): ダブルクリックイベントの検知 (Canvas 上で)
- [ ] UIインタラクション(JS): ダブルクリック時にRust側の自動移動ロジック (`app/game_app.rs` の `handle_double_click`) を呼び出す
- [ ] UIインタラクション(JS): Canvas 上でのドラッグ開始 (`mousedown`)
- [ ] UIインタラクション(JS): ドラッグ中のカード追従表示 (Canvas 上で描画) - **Rust 側の `render_game_rust` と連携要**
- [ ] UIインタラクション(JS): ドラッグ終了 (`mouseup`)
- [ ] UIインタラクション(JS): ドロップ位置から移動先スタックを判定するロジック
- [ ] UIインタラクション(JS): Canvas 上の操作を `gameApp.send_make_move()` 呼び出しに変換

// --- 状態更新と表示 (Canvas) ---
- [x] 状態更新と表示(JS): サーバーからの `GameStateUpdate` 受信時に `apply_game_state` を呼び出す (`app/network_handler.rs`) ✅
- [/] 状態更新と表示(JS): `apply_game_state` (Rust) 後、**Canvas描画関数** (`app/renderer.rs` の `render_game_rust`) を呼び出して画面を更新 - **`apply_game_state` 修正後に `render_game_rust` 実装へ！** ➡️

// --- レンダリング実装 (Rust/Wasm - Canvas版！) ---
- [x] レンダリング実装(Rust/Wasm): Canvas要素と2Dコンテキストを取得 (`app/game_app.rs` の `new`) ✅
- [ ] レンダリング実装(Rust/Wasm): Canvas をクリアする処理 (`app/renderer.rs` の `render_game_rust` 内)
- [ ] レンダリング実装(Rust/Wasm): カードデータに基づき Canvas に図形やテキストを描画 (カード描画関数) (`app/renderer.rs` の `render_game_rust` 内)
- [ ] レンダリング実装(Rust/Wasm): (任意) カード画像をロードして描画する機能

- [ ] 状態更新と表示(JS): サーバーから `MoveRejected` を受け取った場合に、ユーザーにフィードバックを表示

## フェーズ 5: マルチプレイヤー同期と仕上げ ✨💅 (ほぼ未着手)

- [ ] プレイヤー表示(JS): 画面上に参加プレイヤーリストを表示するエリアを作成
- [ ] プレイヤー表示(JS): `PlayerJoined`