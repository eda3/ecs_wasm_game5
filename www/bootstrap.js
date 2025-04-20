// www/bootstrap.js

// まず、wasm-pack が生成した JS ファイルから必要なものをインポートするよ！
// `init` 関数: WASM モジュールを非同期で初期化する関数。
// `GameApp` クラス: Rust 側で #[wasm_bindgen] を付けた構造体が JS ではクラスみたいに見える！
// パスはプロジェクトの構成に合わせてね (http-server がルートを配信するので、ルートからの絶対パス /pkg/ になる)
import init, { GameApp } from '/pkg/ecs_wasm_game5.js';

// グローバルスコープ (どこからでもアクセスできる場所) に gameApp インスタンスを保持する変数を用意するよ。
// 最初は null (まだ無い状態) にしておく。
let gameApp = null;

// --- ドラッグ＆ドロップの状態管理変数 --- ★追加★
let isDragging = false;
let draggedCardElement = null;
let draggedEntityId = null;
let offsetX = 0;
let offsetY = 0;

// --- DOM 要素を取得 --- (後でイベントリスナーを設定するために先に取っておく！)
const connectButton = document.getElementById('connect-button');
const joinButton = document.getElementById('join-button');
const dealButton = document.getElementById('deal-button');
const getStateButton = document.getElementById('get-state-button');
const connectionStatusSpan = document.getElementById('connection-status');
const playerIdSpan = document.getElementById('player-id');
const gameAreaDiv = document.getElementById('game-area'); // ゲーム描画用の div を取得！

// --- メインの非同期処理 --- (WASM のロードは非同期だから async/await を使うよ)
async function main() {
    console.log("🚀 bootstrap.js: WASM モジュールの初期化を開始します...");

    try {
        // init() 関数を呼び出して WASM モジュールを初期化！
        // これが終わるまで待つ (await)
        await init();
        console.log("✅ WASM モジュール初期化完了！");

        // GameApp のインスタンスを作成！ Rust 側の GameApp::new() が呼ばれるよ。
        gameApp = new GameApp();
        console.log("🎮 GameApp インスタンス作成完了！", gameApp);

        // --- 初期状態のボタン制御 ---
        // 最初はサーバーに接続しないとゲームに参加したりできないようにする
        connectButton.disabled = false; // 接続ボタンは有効
        joinButton.disabled = true;    // 参加ボタンは無効
        dealButton.disabled = true;    // 配るボタンは無効
        getStateButton.disabled = false; // 状態取得はいつでもOK?

        // --- イベントリスナーを設定 --- (ボタンがクリックされた時の処理)
        setupEventListeners();
        console.log("🎧 イベントリスナー設定完了！");

        // 定期的に接続状態をチェックして表示を更新する (例)
        setInterval(updateStatusDisplay, 1000); // 1秒ごとに更新

    } catch (error) {
        console.error("❌ WASM モジュールの初期化または GameApp の作成に失敗しました:", error);
        // エラー発生時はユーザーに知らせる (例: アラート表示)
        alert("ゲームの読み込みに失敗しました。コンソールを確認してください。");
        // ボタンを全部無効にするなど
        connectButton.disabled = true;
        joinButton.disabled = true;
        dealButton.disabled = true;
        getStateButton.disabled = true;
    }
}

// --- イベントリスナー設定関数 ---
function setupEventListeners() {
    // gameApp がちゃんと作られてないとダメだからチェック！
    if (!gameApp) {
        console.error("イベントリスナー設定失敗: gameApp が初期化されていません。");
        return;
    }

    // 「サーバーに接続」ボタン
    connectButton.addEventListener('click', () => {
        console.log("🖱️ 接続ボタンクリック");
        gameApp.connect(); // Rust 側の connect() を呼び出す！
        // TODO: 接続試行中の表示とか？
    });

    // 「ゲームに参加」ボタン
    joinButton.addEventListener('click', () => {
        console.log("🖱️ 参加ボタンクリック");
        // とりあえず仮のプレイヤー名で参加！ 本当は入力させるべきだね。
        const playerName = prompt("プレイヤー名を入力してください:", "ギャルゲーマー");
        if (playerName) { // prompt でキャンセルされなかったら
            gameApp.send_join_game(playerName); // Rust 側の send_join_game() を呼び出す！
            // TODO: 参加後のボタン状態変更など
        }
    });

    // 「カードを配る」ボタン
    dealButton.addEventListener('click', () => {
        console.log("🖱️ 配るボタンクリック");
        try {
            gameApp.deal_initial_cards(); // Rust 側の deal_initial_cards() を呼び出す！
            console.log("🃏 Rust 側でカード配布完了。");
            gameApp.render_game_rust();
        } catch (e) {
            console.error("カード配布または描画中にエラー:", e);
        }
    });

    // 「状態取得(Console)」ボタン (描画も行うように変更！)
    getStateButton.addEventListener('click', () => {
        console.log("🖱️ 状態取得ボタンクリック");
        try {
            const stateJson = gameApp.get_world_state_json(); // Rust 側のメソッド呼び出し
            console.log("--- World 状態 (JSON) ---");
            console.log(JSON.parse(stateJson)); // JSON 文字列をパースしてオブジェクトとして表示
            console.log("-------------------------");
            gameApp.render_game_rust();
        } catch (e) {
            console.error("状態の取得、JSONパース、または描画中にエラー: ", e);
        }
    });

    // --- Canvas のリスナー --- ★ここから追加★
    const canvas = document.getElementById('game-canvas');
    if (!canvas) {
        console.error("イベントリスナー設定失敗: Canvas 要素 'game-canvas' が見つかりません。");
        return;
    }

    // -- クリックリスナー (ログ出力のみ) --
    canvas.addEventListener('click', (event) => {
        console.log("Canvas クリック！ ✨ イベント:", event);
        const coords = getCanvasCoordinates(event);
        if (coords) {
            console.log(`>>> Canvas 内クリック座標: x=${coords.x.toFixed(2)}, y=${coords.y.toFixed(2)} <<<`);
            // gameApp.handle_click(coords.x, coords.y); // 必要なら Rust の handle_click を呼ぶ
        }
    });

    // -- ダブルクリックリスナー (Rust の handle_double_click 呼び出し) --
    canvas.addEventListener('dblclick', (event) => {
        console.log("Canvas ダブルクリック！ 🖱️🖱️ イベント:", event);
        if (!gameApp) { console.error("GameApp 未初期化"); return; }

        const coords = getCanvasCoordinates(event);
        if (!coords) return; // 座標が取れなければ何もしない

        console.log(`>>> Canvas 内ダブルクリック座標: x=${coords.x.toFixed(2)}, y=${coords.y.toFixed(2)} <<<`);

        let clickedEntityId = undefined;
        try {
            console.log(`  📞 Rust 呼び出し中: gameApp.get_entity_id_at(${coords.x.toFixed(2)}, ${coords.y.toFixed(2)})`);
            clickedEntityId = gameApp.get_entity_id_at(coords.x, coords.y);
            console.log(`  Rust からの応答 Entity ID: ${clickedEntityId}`);
        } catch (error) {
            console.error("💥 gameApp.get_entity_id_at 呼び出しエラー:", error);
            return;
        }

        if (clickedEntityId !== undefined) {
            console.log(`  ✅ カード発見！ Entity ID: ${clickedEntityId}。Rust のダブルクリックハンドラーを呼び出します...`);
            try {
                console.log(`  🚀 Rust 呼び出し中: gameApp.handle_double_click(${clickedEntityId})`);
                gameApp.handle_double_click(clickedEntityId);
                console.log("  Rust の handle_double_click 関数呼び出し成功！");
            } catch (error) {
                console.error("💥 gameApp.handle_double_click 呼び出しエラー:", error);
            }
        } else {
            console.log("  🤷 この座標にカードは見つかりませんでした。自動移動のためのダブルクリックは無視します。");
        }
    });

    // -- マウスダウンリスナー (ドラッグ開始) --
    canvas.addEventListener('mousedown', (event) => {
        console.log("Canvas マウスダウン！ 🖱️ イベント:", event);
        if (!gameApp) { console.error("GameApp 未初期化"); return; }

        // 左クリック以外は無視 (event.button === 0 が左クリック)
        if (event.button !== 0) {
            console.log("左クリックではないため無視します。");
            return;
        }

        const coords = getCanvasCoordinates(event);
        if (!coords) return;

        console.log(`>>> Canvas 内マウスダウン座標: x=${coords.x.toFixed(2)}, y=${coords.y.toFixed(2)} <<<`);

        let clickedEntityId = undefined;
        try {
            clickedEntityId = gameApp.get_entity_id_at(coords.x, coords.y);
        } catch (error) {
            console.error("💥 gameApp.get_entity_id_at 呼び出しエラー:", error);
            return;
        }

        if (clickedEntityId !== undefined) {
            console.log(`  ✅ カード発見！ Entity ID: ${clickedEntityId}。ドラッグ開始します...`);
            isDragging = true;
            draggedEntityId = clickedEntityId;
            offsetX = coords.x; // ドラッグ開始時のオフセットを記録 (描画用だが一旦保存)
            offsetY = coords.y;

            try {
                console.log(`  🚀 Rust 呼び出し中: gameApp.handle_drag_start(${draggedEntityId}, ${coords.x.toFixed(2)}, ${coords.y.toFixed(2)})`);
                // Rust 側の handle_drag_start を呼び出す (現時点では内部的に DraggingInfo を追加するだけ)
                gameApp.handle_drag_start(draggedEntityId, coords.x, coords.y);
                console.log("  Rust の handle_drag_start 関数呼び出し成功！");

                // Window に mousemove と mouseup リスナーを追加
                // 重要: リスナーには名前付き関数を渡すことで、後で removeEventListener できるようにする！
                window.addEventListener('mousemove', handleMouseMove);
                window.addEventListener('mouseup', handleMouseUp);
                console.log("  Window に mousemove/mouseup リスナーを追加しました。");

            } catch (error) {
                console.error("💥 gameApp.handle_drag_start 呼び出しエラー:", error);
                // エラーが起きたらドラッグ状態をリセット
                isDragging = false;
                draggedEntityId = null;
            }
        } else {
            console.log("  🤷 カードがない場所でマウスダウン。ドラッグは開始しません。");
        }
    });

    console.log("🎧 イベントリスナー設定完了！");
}

// --- Canvas 座標取得ヘルパー関数 ---
function getCanvasCoordinates(event) {
    const canvas = document.getElementById('game-canvas');
    if (!canvas) {
        console.error("getCanvasCoordinates: Canvas 要素が見つかりません。");
        return null;
    }
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    // Canvas 範囲外のイベントも拾うことがあるのでチェック (マイナス座標など)
    if (x < 0 || x > canvas.width || y < 0 || y > canvas.height) {
        // console.log("座標が Canvas 範囲外です。");
        // return null; // 範囲外でも座標を返す方が良い場合もあるのでコメントアウト
    }
    return { x, y };
}

// --- 接続状態などを表示する関数 ---
function updateStatusDisplay() {
    if (!gameApp) return; // gameApp がまだなければ何もしない

    let status = 'Disconnected'; // ★ 変数 status を try の外で定義

    try {
        // Rust 側からデバッグ用の接続状態とプレイヤーIDを取得
        status = gameApp.get_connection_status_debug(); // ★ let を削除
        const playerId = gameApp.get_my_player_id_debug();

        connectionStatusSpan.textContent = status;
        playerIdSpan.textContent = playerId !== undefined ? playerId.toString() : '未参加';

        // --- 接続状態に応じてボタンの有効/無効を切り替え ---
        if (status === 'Connected') {
            connectButton.disabled = true;
            joinButton.disabled = false;
            dealButton.disabled = false;
        } else if (status === 'Connecting') {
            connectButton.disabled = true;
            joinButton.disabled = true;
            dealButton.disabled = true;
        } else { // Disconnected or Error
            connectButton.disabled = false;
            joinButton.disabled = true;
            dealButton.disabled = true;
        }

    } catch (e) {
        console.error("ステータス更新中にエラー:", e);
        connectionStatusSpan.textContent = "エラー";
        playerIdSpan.textContent = "-";
        connectButton.disabled = true;
        joinButton.disabled = true;
        dealButton.disabled = true;
    }

    // 受信メッセージを処理し、状態が変わった場合のみ Rust側のレンダリング関数を呼ぶ
    try {
        // Rust側のメッセージ処理関数を呼び出し、状態が変わったかどうか(true/false)を受け取る
        const stateDidChange = gameApp.process_received_messages();
        // ★デバッグログ追加★ 状態が変わったか、renderを呼ぶか出力
        console.log(`[デバッグ] stateDidChange: ${stateDidChange}`);

        // if (stateDidChange) { // ★★★ 条件分岐をコメントアウト ★★★
        //     console.log("Rust によると状態が変更されました。Rust の描画関数を呼び出します...");
        //     // ★修正: renderGame() の代わりに render_game_rust() を呼び出す！★
        //     gameApp.render_game_rust();
        //     console.log("  render_game_rust 呼び出し完了。"); // ★デバッグログ追加★
        // } else {
        //     // console.log("状態変更なし。再描画はスキップします。"); // 必要ならコメント解除
        // }

        // ★★★ 常に再描画するように変更 ★★★
        console.log("常に Rust の描画関数を呼び出します...");
        gameApp.render_game_rust();
        console.log("  render_game_rust 呼び出し完了。");

    } catch (e) {
        console.error("メッセージ処理またはRustレンダリング呼び出し中にエラー:", e);
    }
}

// --- マウスムーブハンドラー (ドラッグ中) ---
function handleMouseMove(event) {
    // isDragging フラグが false なら何もしない (ドラッグ中じゃない)
    if (!isDragging) {
        return;
    }

    // 重要: ドラッグ中にテキスト選択などが起こらないようにデフォルト動作を抑制
    event.preventDefault();

    // 現在のマウス座標 (Canvas ローカル座標) を取得
    const coords = getCanvasCoordinates(event);
    if (!coords) return;

    // デバッグ用に座標とドラッグ中の ID をログ出力
    // console.log(`-- ドラッグ中 -- ID: ${draggedEntityId}, x: ${coords.x.toFixed(2)}, y: ${coords.y.toFixed(2)}`);

    // --- TODO: ドラッグ中の描画更新 --- ★将来の課題★
    // ここで、ドラッグされているカード (draggedEntityId) の Position を
    // Rust 側で更新し (例: `gameApp.update_dragged_position(draggedEntityId, coords.x, coords.y);`)
    // その後、`gameApp.render_game_rust()` を呼び出して画面を再描画する、
    // という処理が必要になります。
    // Rust 側に `update_dragged_position` のような関数を実装する必要があります。
}

// --- マウスアップハンドラー (ドラッグ終了) ---
function handleMouseUp(event) {
    // isDragging フラグが false なら何もしない (ドラッグ開始してないのに mouseup だけ発生した場合など)
    if (!isDragging) {
        return;
    }

    console.log("マウスアップ！ ドラッグ終了処理を開始します。 🖱️⬆️ イベント:", event);

    // まず isDragging フラグを false にして、これ以上 mousemove が処理されないようにする
    isDragging = false;

    // Window からリスナーを削除！ これを忘れると、ドラッグしてなくても mousemove や mouseup が発生し続けてしまう！
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
    console.log("  Window から mousemove/mouseup リスナーを削除しました。");

    // マウスが離された座標を取得
    const coords = getCanvasCoordinates(event);
    if (!coords) {
        console.warn("マウスアップ座標が Canvas 外のようです。ドラッグ終了座標は (0, 0) として処理を試みます。");
        //座標が取れない場合もドラッグ終了処理は呼び出す（エラーになるかもしれないが）
        coords = { x: 0, y: 0 };
    }

    // ドラッグされていたカードの ID を取得 (null チェック)
    const entityIdToEnd = draggedEntityId;
    draggedEntityId = null; // 状態をリセット

    if (entityIdToEnd !== null && gameApp) {
        console.log(`>>> マウスアップ座標: x=${coords.x.toFixed(2)}, y=${coords.y.toFixed(2)} <<<`);
        try {
            console.log(`  🚀 Rust 呼び出し中: gameApp.handle_drag_end(${entityIdToEnd}, ${coords.x.toFixed(2)}, ${coords.y.toFixed(2)})`);
            // Rust 側の handle_drag_end を呼び出す！
            // これにより、移動ルールのチェック、World の更新、サーバーへの通知が行われるはず！
            gameApp.handle_drag_end(entityIdToEnd, coords.x, coords.y);
            console.log("  Rust の handle_drag_end 関数呼び出し成功！");
            // 注意: ここでも画面更新は Rust 側 + サーバーからの応答で行われる想定。
        } catch (error) {
            console.error("💥 gameApp.handle_drag_end 呼び出しエラー:", error);
        }
    } else {
        console.warn("ドラッグ終了処理をスキップ: entityIdToEnd が null または gameApp が未初期化です。");
    }

    console.log("ドラッグ終了処理完了。");
}

// --- 実行開始！ ---
main(); 