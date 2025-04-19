# マルチプレイソリティア開発計画書 🚀 (詳細版)

これは、Rust + WASM + 自作ECSで作るマルチプレイソリティアゲームの開発計画チェックリストだよ！✅

## フェーズ 1: プロジェクトセットアップ & ECS基盤構築 🏗️ (完了)

- [x] Rust WASM プロジェクトの基本セットアップ (`Cargo.toml`, `wasm-pack`, ビルド設定) // 基本はOK
- [x] `wasm-bindgen` の設定と JS 連携の準備 // 基本はOK
- [x] ログ出力 (`console.log`) とパニックハンドリングの設定 (`console_error_panic_hook`)
- [x] ECSコア実装: `Entity` 型の定義
- [x] ECSコア実装: `Component` の概念定義 (トレイト？ マーカー？) // 基本はOK
- [x] ECSコア実装: `World` 構造体の実装 (エンティティとコンポーネントの管理)
- [x] ECSコア実装: `World` エンティティ作成・削除 // コンポーネント単位削除は実装済
- [x] ECSコア実装: `World` コンポーネント追加・削除・取得
- [x] ECSコア実装: `World` 特定コンポーネントを持つエンティティのクエリ
- [x] ECSコア実装: `System` の概念定義 (トレイト？ 関数？) // 基本ファイルは作成済
- [x] 基本コンポーネント定義: `Position` (x, y座標)
- [x] 基本コンポーネント定義: `Card` (スート, ランク, 表裏)
- [x] 基本コンポーネント定義: `Player` (ID, 名前など)
- [x] 基本コンポーネント定義: `StackInfo` (どの場のカードか、場の中での位置)
- [x] 基本コンポーネント定義: `GameState` (ゲーム全体の状態管理用？)

## フェーズ 2: 基本ゲームロジック & 画面表示 (クライアント単体) 🃏🖼️ (完了)

- [x] ゲーム初期化: カードデッキ生成ロジック // create_standard_deck in card.rs
- [x] ゲーム初期化: 初期カード配置 (ディール) システムの実装 // deal_system.rs & lib.rs に deal_initial_cards() を実装！✨ & Position追加済！
- [x] レンダリング準備: `GameApp` (WASM側メイン構造体) の実装 (`src/lib.rs`)
- [x] レンダリング準備: `GameApp` から World の状態を取得するメソッド実装 // get_world_state_json() を実装！🦴→✅ & Position追加済！
- [x] レンダリング準備: JS側でWASMをロードし、`GameApp` インスタンスを作成 // www/bootstrap.js で実装！🚀
- [x] レンダリング準備: 基本的なHTML/CSS構造の作成 (`index.html`, `style.css`) // www/ に作成！🎨
- [x] レンダリング準備: JSから定期的にWASMのゲーム状態を取得し、コンソール等に表示 (デバッグ用) // bootstrap.js のボタンと定期更新で部分的に実装！⚙️
- [x] レンダリング準備: Canvas や DOM を使った基本的なカード描画 (JS側) // bootstrap.js と style.css で実装！🃏

## フェーズ 3: ネットワーク実装 🌐🤝 (ほぼ完了)

- [x] 通信プロトコル定義: `ClientMessage` (クライアント → サーバー) の定義 (JoinGame, MakeMove など) // ProvideInitialState追加済！✅
- [x] 通信プロトコル定義: `ServerMessage` (サーバー → クライアント) の定義 (GameJoined, GameStateUpdate, MoveRejected など)
- [x] 通信プロトコル定義: メッセージ共通で使うデータ構造 (PlayerId, CardData, GameStateData など) の定義
- [x] 通信プロトコル定義: `serde` を使ったシリアライズ/デシリアライズ設定
- [x] クライアント側ネットワーク処理: WebSocket接続マネージャー (`NetworkManager`) の実装
- [x] クライアント側ネットワーク処理: 接続・切断処理 (URL: `ws://localhost:8101`) // 接続は実装済 & URL修正済！
- [x] クライアント側ネットワーク処理: メッセージ送信・受信キュー
- [x] クライアント側ネットワーク処理: 接続状態管理 (`ConnectionStatus`)
- [ ] クライアント側ネットワーク処理: (任意) 切断時の自動再接続ロジックを実装 (例: `NetworkManager` にタイマーと再接続試行を追加)
- [x] クライアント側ネットワーク処理: `GameApp` から `NetworkManager` を利用
- [x] クライアント側ネットワーク処理: `connect()` メソッド
- [x] クライアント側ネットワーク処理: `send_message()` ヘルパーメソッド
- [x] クライアント側ネットワーク処理: `send_join_game()`, `send_make_move()` などWASM公開メソッド
- [x] クライアント側ネットワーク処理: `send_initial_state()` メソッド追加 (`deal_initial_cards` 内で呼び出し) ✅
- [x] クライアント側ネットワーク処理: `process_received_messages()` で受信キューを処理
- [x] クライアント側ネットワーク処理: `apply_game_state()` で受信データ (`GameStateData`) を `World` に反映 // 基本実装済
- [x] サーバー側(JS): WebSocketサーバー起動 (`localhost:8101`) // server/ws_server.js と npm script で実装！🔌 & ファイル移動済！✅
- [x] サーバー側(JS): クライアント接続/切断管理 // server/ws_server.js で簡易実装！🤝
- [x] サーバー側(JS): `JoinGame` メッセージ受信処理と `GameJoined` 応答実装 // server/ws_server.js で実装！📥 & gameState.cards 送信するように修正！✅
- [x] サーバー側(JS): プレイヤーリストの管理と `PlayerJoined`/`PlayerLeft` のブロードキャスト実装 // server/ws_server.js で実装！💾
- [x] サーバー側(JS): ゲーム状態管理: カード情報 (`gameState.cards`) の保持と初期化 // server/ws_server.js で `ProvideInitialState` 受信時に設定するように変更！✅
- [ ] サーバー側(JS): `MakeMove` メッセージ受信処理 (今は無視)
- [ ] サーバー側(JS): カード移動を `gameState.cards` に反映させるロジック
- [ ] サーバー側(JS): ゲーム状態の変更を全クライアントに通知する `GameStateUpdate` メッセージのブロードキャスト実装

## フェーズ 4: ソリティアのルールとインタラクション実装 🎮👆

- [ ] ルール実装(Rust): `StackType` ごとのカード移動可否判定ヘルパー関数を作成 (`src/rules.rs`？)
- [ ] ルール実装(Rust): タブローからタブローへの移動ルール実装 (色違い、ランク連続)
- [ ] ルール実装(Rust): タブローからファンデーションへの移動ルール実装 (同スート、ランク昇順)
- [ ] ルール実装(Rust): ストックからウェストへの移動ルール実装 (山札クリック時の処理)
- [ ] ルール実装(Rust): ウェストからタブロー/ファンデーションへの移動ルール実装
- [ ] ルール実装(Rust): (任意) カード自動移動ロジック (ダブルクリック時、ファンデーションへ移動できるか判定して `MakeMove` を生成？)
- [ ] ルール実装(Rust): ゲームクリア判定ロジック (全カードがファンデーションにあるか？)
- [x] UIインタラクション(JS): カードクリックによる選択状態表示 (`.selected` クラス) // bootstrap.js と style.css で実装！🖱️
- [x] UIインタラクション(JS): カードダブルクリックイベントの検知とログ出力 // bootstrap.js で実装！🖱️🖱️
- [ ] UIインタラクション(JS): カードダブルクリック時にRust側の自動移動ロジックを呼び出す (`GameApp` にメソッド追加)
- [ ] UIインタラクション(JS): カードのドラッグ開始 (`mousedown` or `dragstart`)
- [ ] UIインタラクション(JS): カードのドラッグ中 (`mousemove` or `drag`) のカード追従表示
- [ ] UIインタラクション(JS): カードのドロップ (`mouseup` or `drop`)
- [ ] UIインタラクション(JS): ドロップ位置から移動先スタックを判定するロジック
- [ ] UIインタラクション(JS): クリック/ダブルクリック/ドロップ操作を `gameApp.send_make_move()` 呼び出しに変換
- [ ] 状態更新と表示(JS): サーバーからの `GameStateUpdate` 受信時に `apply_game_state` を呼び出す (今は `GameJoined` でのみ実行)
- [ ] 状態更新と表示(JS): `apply_game_state` 内で、受け取ったカード情報を元に `renderGame` を呼び出して画面を更新
- [ ] 状態更新と表示(JS): サーバーから `MoveRejected` を受け取った場合に、ユーザーにフィードバックを表示 (例: アラート、メッセージ表示)

## フェーズ 5: マルチプレイヤー同期と仕上げ ✨💅

- [ ] プレイヤー表示(JS): 画面上に参加プレイヤーリストを表示するエリアを作成 (`index.html`, `style.css`)
- [ ] プレイヤー表示(JS): `PlayerJoined` / `PlayerLeft` 受信時、または定期的に `gameState.players` を参照してプレイヤーリスト表示を更新
- [ ] 同期の安定化: (必要に応じて) 複数プレイヤー同時操作時の競合対策 (例: サーバー側でのロック、操作キュー)
- [ ] 同期の安定化: (必要に応じて) 遅延補償 (Lag Compensation) の検討
- [ ] UI/UX改善: カード移動アニメーションの実装 (CSS Transition or JS Animation)
- [ ] UI/UX改善: カードや背景のグラフィック改善 (CSS or 画像)
- [ ] UI/UX改善: ボタンやステータス表示のデザイン調整 (CSS)
- [ ] UI/UX改善: 操作感の微調整 (ドラッグ感度、クリック反応など)
- [ ] エラーハンドリング(JS): ネットワーク切断時のメッセージ表示と再接続ボタンの有効化
- [ ] エラーハンドリング(JS): サーバーからのエラーメッセージ (`ServerMessage::Error`) をユーザーに分かりやすく表示

## フェーズ 6: テストとデプロイ 🧪🚀

- [ ] テスト(Rust): ECSコア (`World`, `Entity`, `Component`) のユニットテスト作成
- [ ] テスト(Rust): カード生成 (`deal_system`) のユニットテスト作成
- [ ] テスト(Rust): カード移動ルール のユニットテスト作成
- [ ] テスト(JS/連携): ネットワーク通信 (`JoinGame`, `MakeMove` 等) の結合テスト (簡易サーバー or モックを使用)
- [ ] ビルドと最適化: `wasm-pack build --release` の実行確認
- [ ] ビルドと最適化: (任意) `wasm-opt` を使用したWASMバイナリサイズ最適化
- [ ] デプロイ準備: 本番用静的ファイルサーバー (nginx等) の設定方法調査・決定
- [ ] デプロイ準備: WebSocketサーバー (Node.js) の本番環境での実行方法 (pm2等) 調査・決定
- [ ] デプロイ準備: `deploy.sh` スクリプトの最終確認と調整
- [ ] デプロイ！ 🎉 