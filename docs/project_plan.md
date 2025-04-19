# マルチプレイソリティア開発計画書 🚀 (詳細版)

これは、Rust + WASM + 自作ECSで作るマルチプレイソリティアゲームの開発計画チェックリストだよ！✅

## フェーズ 1: プロジェクトセットアップ & ECS基盤構築 🏗️ (一部見直し要！)

- [x] Rust WASM プロジェクトの基本セットアップ (`Cargo.toml`, `wasm-pack`, ビルド設定) // 基本はOK
- [x] `wasm-bindgen` の設定と JS 連携の準備 // 基本はOK
- [x] ログ出力 (`console.log`) とパニックハンドリングの設定 (`console_error_panic_hook`)
- [x] ECSコア実装: `Entity` 型の定義 // `entity.rs` 実装OK！
- [x] ECSコア実装: `Component` の概念定義 (トレイト？ マーカー？) // `component.rs` でトレイトと `ComponentStorage` の概念はOK！ **だが注意:** `components/` フォルダと `component.rs` 内で具体的なコンポーネント定義が重複/混在している！要整理！🧹
- [ ] ECSコア実装: `World` 構造体の実装 (エンティティとコンポーネントの管理) // `world.rs` の構造体自体はあるが… ↓↓↓
- [ ] ECSコア実装: `World` エンティティ作成・削除 // `create_entity` はOK！削除は未実装。
- [ ] ECSコア実装: `World` コンポーネント追加・削除・取得 // **最重要課題！ `add/get/remove_component` 等が `unimplemented!()` のまま！** 😭 これがないと動かない！
- [ ] ECSコア実装: `World` 特定コンポーネントを持つエンティティのクエリ // 上記同様、未実装！
- [x] ECSコア実装: `System` の概念定義 (トレイト？ 関数？) // `system.rs` でトレイト定義OK！
- [x] 基本コンポーネント定義: `Position` (x, y座標) // **注意:** `components/position.rs` と `component.rs` に定義あり！要統一！
- [x] 基本コンポーネント定義: `Card` (スート, ランク, 表裏) // **注意:** `components/card.rs` と `component.rs` に定義あり！要統一！
- [x] 基本コンポーネント定義: `Player` (ID, 名前など) // **注意:** `components/player.rs` と `component.rs` に定義あり！要統一！
- [x] 基本コンポーネント定義: `StackInfo` (どの場のカードか, 場の中での位置) // **注意:** `components/stack.rs` と `component.rs` に定義あり！要統一！ `component.rs` 版は情報が少ない？
- [x] 基本コンポーネント定義: `GameState` (ゲーム全体の状態管理用？) // **注意:** `components/game_state.rs` と `component.rs` に定義あり！要統一！

## フェーズ 2: 基本ゲームロジック & 画面表示 (クライアント単体) 🃏🖼️ (Canvasへ移行中！)

- [x] ゲーム初期化: カードデッキ生成ロジック // `components/card.rs` の `create_standard_deck` はOK！
- [ ] ゲーム初期化: 初期カード配置 (ディール) システムの実装 // `deal_system.rs` は存在するが、未実装の `World` メソッドに依存しているため **未完了！** 🚧
- [x] レンダリング準備: `GameApp` (WASM側メイン構造体) の実装 // `lib.rs` に構造体はあるが、`World` 未実装のため機能不全。
- [ ] レンダリング準備: `GameApp` から World の状態を取得するメソッド実装 // `get_world_state_json` はあるが、`World` 未実装のため動作しない。
- [x] レンダリング準備: JS側でWASMをロードし、`GameApp` インスタンスを作成 // `bootstrap.js` で実施。
- [x] レンダリング準備: 基本的なHTML/CSS構造の作成 (`index.html` に `<canvas>` を設置！✅)
- [x] レンダリング準備: JSから定期的にWASMのゲーム状態を取得し、コンソール等に表示 // 基本は `requestAnimationFrame` で `render_game_rust` を呼ぶ形。
- [ ] レンダリング実装(Rust/Wasm): **Canvas** を使った基本的なカード描画 // ← DOMからCanvasに変更！ `lib.rs` の `render_game_rust` で実装が必要！

## フェーズ 3: ネットワーク実装 🌐🤝 (ほぼ完了！🎉 一部修正要)

- [x] 通信プロトコル定義: `ClientMessage` (クライアント → サーバー) の定義 (JoinGame, MakeMove など) // `protocol.rs` でOK！ `ProvideInitialState`追加済！✅
- [x] 通信プロトコル定義: `ServerMessage` (サーバー → クライアント) の定義 (GameJoined, GameStateUpdate, MoveRejected など) // `protocol.rs` でOK！
- [x] 通信プロトコル定義: メッセージ共通で使うデータ構造 (PlayerId, CardData, GameStateData など) の定義 // `protocol.rs` でOK！ `PositionData` は `Position` と統一検討？
- [x] 通信プロトコル定義: `serde` を使ったシリアライズ/デシリアライズ設定 // OK！
- [x] クライアント側ネットワーク処理: WebSocket接続マネージャー (`NetworkManager`) の実装 // `network.rs` でOK！
- [x] クライアント側ネットワーク処理: 接続・切断処理 (URL: `ws://localhost:8101`) // 接続は実装済 & URL修正済！
- [x] クライアント側ネットワーク処理: メッセージ送信・受信キュー // OK！
- [x] クライアント側ネットワーク処理: 接続状態管理 (`ConnectionStatus`) // OK！
- [ ] クライアント側ネットワーク処理: (任意) 切断時の自動再接続ロジックを実装 (例: `NetworkManager` にタイマーと再接続試行を追加)
- [x] クライアント側ネットワーク処理: `GameApp` から `NetworkManager` を利用 // OK！
- [x] クライアント側ネットワーク処理: `connect()` メソッド // OK！
- [x] クライアント側ネットワーク処理: `send_message()` ヘルパーメソッド // OK！
- [x] クライアント側ネットワーク処理: `send_join_game()`, `send_make_move()` などWASM公開メソッド // OK！
- [x] クライアント側ネットワーク処理: `send_initial_state()` メソッド追加 (`deal_initial_cards` 内で呼び出し) ✅
- [x] クライアント側ネットワーク処理: `process_received_messages()` で受信キューを処理 // OK！
- [ ] クライアント側ネットワーク処理: `apply_game_state()` で受信データ (`GameStateData`) を `World` に反映 // `lib.rs` にあるが、`World` 未実装と hecs コード残存のため **要修正！** 🔧
- [x] サーバー側(JS): WebSocketサーバー起動 (`localhost:8101`) // server/ws_server.js と npm script で実装！🔌 & ファイル移動済！✅
- [x] サーバー側(JS): クライアント接続/切断管理 // server/ws_server.js で簡易実装！🤝
- [x] サーバー側(JS): `JoinGame` メッセージ受信処理と `GameJoined` 応答実装 // server/ws_server.js で実装！📥 & gameState.cards 送信するように修正！✅
- [x] サーバー側(JS): プレイヤーリストの管理と `PlayerJoined`/`PlayerLeft` のブロードキャスト実装 // server/ws_server.js で実装！💾
- [x] サーバー側(JS): ゲーム状態管理: カード情報 (`gameState.cards`) の保持と初期化 // server/ws_server.js で `ProvideInitialState` 受信時に設定するように変更！✅
- [x] サーバー側(JS): `MakeMove` メッセージ受信処理 // server/ws_server.js で実装！✅
- [x] サーバー側(JS): カード移動を `gameState.cards` に反映させるロジック // server/ws_server.js で基本ロジック実装！✅
- [x] サーバー側(JS): ゲーム状態の変更を全クライアントに通知する `GameStateUpdate` メッセージのブロードキャスト実装 // server/ws_server.js で `broadcastGameStateUpdate()` を呼び出すように実装！✅

## フェーズ 4: ソリティアのルールとインタラクション実装 🃏👆 (Canvasへ移行中！一部見直し要)

- [/] ルール実装(Rust): `StackType` ごとのカード移動可否判定ヘルパー関数を作成 (`src/rules.rs`？) // 基本ヘルパー (`can_move_to_foundation`, `can_move_to_tableau`) と `CardColor` は実装！✅ **だが注意:** `component.rs` 版の型に依存、`World` 連携部分は未実装 or hecs 依存！🔧
- [/] ルール実装(Rust): タブローからタブローへの移動ルール実装 (色違い、ランク連続) // `can_move_to_tableau` で実装済！✅ (上記注意点あり)
- [/] ルール実装(Rust): タブローからファンデーションへの移動ルール実装 (同スート、ランク昇順) // `can_move_to_foundation` で実装済！✅ (上記注意点あり)
- [x] ルール実装(Rust): ストックからウェストへの移動ルール実装 (山札クリック時の処理) // `can_deal_from_stock`, `can_reset_stock_from_waste` を実装！✅ (World状態を見ない単純版)
- [/] ルール実装(Rust): ウェストからタブロー/ファンデーションへの移動ルール実装 // `can_move_from_waste_to_tableau`, `can_move_from_waste_to_foundation` を実装！✅ (上記注意点あり)
- [/] ルール実装(Rust): (任意) カード自動移動ロジック (ダブルクリック時、ファンデーションへ移動できるか判定して `MakeMove` を生成？) // `find_automatic_foundation_move` を実装！✅ (上記注意点あり)
- [x] ルール実装(Rust): ゲームクリア判定ロジック (全カードがファンデーションにあるか？) // `check_win_condition` を実装！✅ (`WinConditionSystem` もあるが、`World` 未実装により動作しない)

// --- UIインタラクション(JS) は Canvas ベースに変更 ---
- [x] UIインタラクション(JS): カードクリックによる選択状態表示 (`.selected` クラス) // ← Canvas では別実装 (不要)
- [ ] UIインタラクション(JS): Canvas クリックイベントの検知とログ出力
- [ ] UIインタラクション(JS): クリック座標からカード/スタックを特定するロジック (Rust側でも可)
- [ ] UIインタラクション(JS): ダブルクリックイベントの検知 (Canvas 上で)
- [ ] UIインタラクション(JS): ダブルクリック時にRust側の自動移動ロジックを呼び出す
- [ ] UIインタラクション(JS): Canvas 上でのドラッグ開始 (`mousedown`)
- [ ] UIインタラクション(JS): ドラッグ中のカード追従表示 (Canvas 上で描画)
- [ ] UIインタラクション(JS): ドラッグ終了 (`mouseup`)
- [ ] UIインタラクション(JS): ドロップ位置から移動先スタックを判定するロジック
- [ ] UIインタラクション(JS): Canvas 上の操作を `gameApp.send_make_move()` 呼び出しに変換

// --- 状態更新と表示 (Canvas) ---
- [x] 状態更新と表示(JS): サーバーからの `GameStateUpdate` 受信時に `apply_game_state` を呼び出す // 呼び出しは行われるはず (中の処理は要修正)
- [ ] 状態更新と表示(JS): `apply_game_state` (Rust) 後、**Canvas描画関数** (Rust) を呼び出して画面を更新 // ← `render_game_rust` の役割変更！ (要実装)

// --- レンダリング実装 (Rust/Wasm - Canvas版！) ---
- [ ] レンダリング実装(Rust/Wasm): Canvas要素と2Dコンテキストを取得 // `lib.rs` の `GameApp::new` で実装済み！✅
- [ ] レンダリング実装(Rust/Wasm): Canvas をクリアする処理 // `render_game_rust` 内で要実装
- [ ] レンダリング実装(Rust/Wasm): カードデータに基づき Canvas に図形やテキストを描画 (カード描画関数) // `render_game_rust` 内で要実装
- [ ] レンダリング実装(Rust/Wasm): (任意) カード画像をロードして描画する機能

// --- DOMレンダリング関連タスクは削除 ---
// (削除済み)

- [ ] 状態更新と表示(JS): サーバーから `MoveRejected` を受け取った場合に、ユーザーにフィードバックを表示

## フェーズ 5: マルチプレイヤー同期と仕上げ ✨💅 (未着手)

- [ ] プレイヤー表示(JS): 画面上に参加プレイヤーリストを表示するエリアを作成 (`index.html`, `style.css`)
- [ ] プレイヤー表示(JS): `PlayerJoined` / `PlayerLeft` 受信時、または定期的に `gameState.players` を参照してプレイヤーリスト表示を更新
- [ ] 同期の安定化: (必要に応じて) 複数プレイヤー同時操作時の競合対策 (例: サーバー側でのロック、操作キュー)
- [ ] 同期の安定化: (必要に応じて) 遅延補償 (Lag Compensation) の検討
- [ ] UI/UX改善: カード移動アニメーションの実装 (CSS Transition or JS Animation / Canvas アニメーション)
- [ ] UI/UX改善: カードや背景のグラフィック改善 (CSS or 画像 or Canvas描画)
- [ ] UI/UX改善: ボタンやステータス表示のデザイン調整 (CSS or Canvas描画)
- [ ] UI/UX改善: 操作感の微調整 (ドラッグ感度、クリック反応など)
- [ ] エラーハンドリング(JS): ネットワーク切断時のメッセージ表示と再接続ボタンの有効化
- [ ] エラーハンドリング(JS): サーバーからのエラーメッセージ (`ServerMessage::Error`) をユーザーに分かりやすく表示

## フェーズ 6: テストとデプロイ 🧪🚀 (一部開始)

- [/] テスト(Rust): ECSコア (`World`, `Entity`, `Component`) のユニットテスト作成 // `entity.rs`, `component.rs` に一部テストあり。`World` は未実装のためテスト不可。
- [/] テスト(Rust): カード生成 (`deal_system`) のユニットテスト作成 // `deal_system.rs` にテストあるが `World` 未実装のため動作しない。
- [/] テスト(Rust): カード移動ルール のユニットテスト作成 // `rules.rs` に基本的なテストあり。`World` 連携部分はテスト不足。
- [ ] テスト(JS/連携): ネットワーク通信 (`JoinGame`, `MakeMove` 等) の結合テスト (簡易サーバー or モックを使用)
- [ ] ビルドと最適化: `wasm-pack build --release` の実行確認
- [ ] ビルドと最適化: (任意) `wasm-opt` を使用したWASMバイナリサイズ最適化
- [ ] デプロイ準備: 本番用静的ファイルサーバー (nginx等) の設定方法調査・決定
- [ ] デプロイ準備: WebSocketサーバー (Node.js) の本番環境での実行方法 (pm2等) 調査・決定
- [ ] デプロイ準備: `deploy.sh` スクリプトの最終確認と調整
- [ ] デプロイ！ 🎉 