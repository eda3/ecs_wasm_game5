// www/bootstrap.js

// まず、wasm-pack が生成した JS ファイルから必要なものをインポートするよ！
// `init` 関数: WASM モジュールを非同期で初期化する関数。
// `GameApp` クラス: Rust 側で #[wasm_bindgen] を付けた構造体が JS ではクラスみたいに見える！
// パスはプロジェクトの構成に合わせてね (http-server がルートを配信するので、ルートからの絶対パス /pkg/ になる)
import init, { GameApp } from '/pkg/ecs_wasm_game5.js';

// グローバルスコープ (どこからでもアクセスできる場所) に gameApp インスタンスを保持する変数を用意するよ。
// 最初は null (まだ無い状態) にしておく。
let gameApp = null;

// --- ドラッグ＆ドロップの状態管理変数 ---
let isDragging = false;
// let draggedCardElement = null; // Canvas 描画なので DOM 要素は不要
let draggedEntityId = null;
// let offsetX = 0; // オフセットは Rust 側の DraggingInfo に持たせる
// let offsetY = 0;

// --- ★追加: requestAnimationFrame のループID --- ★
let animationFrameId = null;

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

        // --- ★修正: 定期実行を setInterval から requestAnimationFrame ループに変更 --- ★
        // setInterval(updateStatusDisplay, 1000); // ← これを削除！
        console.log("🎨 ゲームループ (requestAnimationFrame) を開始します...");
        gameLoop(); // 新しいゲームループ関数を呼び出す！

    } catch (error) {
        console.error("❌ WASM モジュールの初期化または GameApp の作成に失敗しました:", error);
        // エラー発生時はユーザーに知らせる (例: アラート表示)
        alert("ゲームの読み込みに失敗しました。コンソールを確認してください。");
        // ボタンを全部無効にするなど
        connectButton.disabled = true;
        joinButton.disabled = true;
        dealButton.disabled = true;
        getStateButton.disabled = true;
        // ★ エラー時にループを止める処理も追加 ★
        if (animationFrameId) {
            cancelAnimationFrame(animationFrameId);
            animationFrameId = null;
            console.log("🛑 エラー発生のためゲームループを停止しました。");
        }
    }
}

// --- ★新しい関数: ゲームループ --- ★
function gameLoop() {
    // まず、次のフレームで再度 gameLoop を呼び出すように予約！
    // これでループが継続するよ。
    animationFrameId = requestAnimationFrame(gameLoop);

    // --- ループ内で行う処理 --- //
    // 1. 接続状態などの表示を更新 (これは頻繁じゃなくていいかもだけど、一旦入れる)
    updateStatusDisplay();

    // 2. Rust 側のゲーム状態に基づいて Canvas を再描画！
    //    update_dragged_position で Position が更新されていれば、
    //    ここでドラッグ中のカードが新しい位置に描画される！✨
    if (gameApp) {
        try {
            // ★ render_game_rust の呼び出しをここに移動 ★
            // console.log("🎨 Rendering game state..."); // ログが多すぎる場合はコメントアウト
            gameApp.render_game_rust();
        } catch (e) {
            console.error("💥 Rust レンダリング中にエラー:", e);
            // エラーが起きたらループを止める？ (とりあえず止めないでおく)
            // cancelAnimationFrame(animationFrameId);
            // animationFrameId = null;
        }
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

    // --- Canvas のリスナー --- ★★★ Rust側で設定するため、以下のリスナーは削除 ★★★
    const canvas = document.getElementById('game-canvas');
    if (!canvas) {
        console.error("イベントリスナー設定失敗: Canvas 要素 'game-canvas' が見つかりません。");
        return;
    }

    /* --- 削除: クリックリスナー ---
    canvas.addEventListener('click', (event) => {
        console.log("Canvas クリック！ ✨ イベント:", event);
        if (isDragging) {
            console.log("  isDragging is true, ignoring click event to prevent conflict with drag end.");
            return;
        }
        const coords = getCanvasCoordinates(event);
        if (coords) {
            console.log(`>>> Canvas 内クリック座標: x=${coords.x.toFixed(2)}, y=${coords.y.toFixed(2)} <<<`);
            gameApp.handle_click(coords.x, coords.y);
        }
    });
    */

    /* --- 削除: ダブルクリックリスナー ---
    canvas.addEventListener('dblclick', (event) => {
        console.log("Canvas ダブルクリック！ 🖱️🖱️ イベント:", event);
        if (!gameApp) { console.error("GameApp 未初期化"); return; }

        const coords = getCanvasCoordinates(event);
        if (!coords) return;

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
    */

    /* --- 削除: マウスダウンリスナー (と、その中の Window リスナーアタッチ) ---
    canvas.addEventListener('mousedown', (event) => {
        console.log("[DEBUG] mousedown リスナー開始");

        if (!gameApp) { console.error("GameApp 未初期化"); return; }
        if (event.button !== 0) { console.log("左クリックではないため無視"); return; }

        const coords = getCanvasCoordinates(event);
        if (!coords) { console.log("[DEBUG] mousedown: 座標取得失敗"); return; }
        console.log(`[DEBUG] mousedown: 座標 (${coords.x.toFixed(2)}, ${coords.y.toFixed(2)})`);

        let clickedEntityId = undefined;
        try {
            console.log(`[DEBUG] mousedown: gameApp.get_entity_id_at(${coords.x.toFixed(2)}, ${coords.y.toFixed(2)}) 呼び出し`);
            clickedEntityId = gameApp.get_entity_id_at(coords.x, coords.y);
            console.log(`[DEBUG] mousedown: get_entity_id_at 応答: ${clickedEntityId}`);
        } catch (error) {
            console.error("💥 gameApp.get_entity_id_at 呼び出しエラー:", error);
            return;
        }

        if (clickedEntityId !== undefined) {
            console.log(`[DEBUG] mousedown: カード発見 (ID: ${clickedEntityId})。ドラッグ開始処理へ`);
            isDragging = true;
            draggedEntityId = clickedEntityId;

            try {
                console.log(`[DEBUG] mousedown: gameApp.handle_drag_start(${draggedEntityId}, ${coords.x.toFixed(2)}, ${coords.y.toFixed(2)}) 呼び出し`);
                gameApp.handle_drag_start(draggedEntityId, coords.x, coords.y);
                console.log("[DEBUG] mousedown: handle_drag_start 呼び出し成功");

                // ★★★ 削除: Rust側でやるため Window リスナーのアタッチ処理は不要 ★★★
                // window.addEventListener('mousemove', handleMouseMove);
                // window.addEventListener('mouseup', handleMouseUp);
                // console.log("[DEBUG] mousedown: Window リスナー追加完了");

            } catch (error) {
                console.error("💥 gameApp.handle_drag_start 呼び出しエラー:", error);
                isDragging = false;
                draggedEntityId = null;
            }
        } else {
            console.log("[DEBUG] mousedown: カードが見つからなかったためドラッグ開始せず");
        }
        console.log("[DEBUG] mousedown リスナー終了");
    });
    */

    // ★ 他のリスナー (mousemove, mouseup のヘルパー関数自体) はまだ残しておく
    //   -> Rust 側の detach から呼ばれる可能性は低いが、コード整理するまでは一旦残す

    console.log("🎧 Button イベントリスナー設定完了 (Canvas リスナーは Rust側で設定)");
}

// --- Canvas 座標取得ヘルパー関数 ---
function getCanvasCoordinates(event) {
    const canvas = document.getElementById('game-canvas');
    if (!canvas) {
        console.error("getCanvasCoordinates: Canvas 要素が見つかりません。");
        return null;
    }
    const rect = canvas.getBoundingClientRect();
    // ★★★ デバッグログ追加 ★★★
    console.log(`[DEBUG] getCanvasCoordinates: clientX=${event.clientX}, clientY=${event.clientY}, rect.left=${rect.left}, rect.top=${rect.top}`);
    // ★★★ ここまで ★★★
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    // ★★★ デバッグログ追加 ★★★
    console.log(`[DEBUG] getCanvasCoordinates: calculated x=${x}, y=${y}`);
    // ★★★ ここまで ★★★
    // Canvas 範囲外のイベントも拾うことがあるのでチェック (マイナス座標など)
    if (x < 0 || x > canvas.width || y < 0 || y > canvas.height) {
        // console.log("座標が Canvas 範囲外です。");
        // return null; // 範囲外でも座標を返す方が良い場合もあるのでコメントアウト
    }
    return { x, y };
}

// --- 接続状態などを表示する関数 ---
function updateStatusDisplay() {
    if (!gameApp) return;

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

    // --- ★削除: メッセージ処理と描画呼び出しを gameLoop に移動 --- ★
    // try {
    //     const stateDidChange = gameApp.process_received_messages();
    //     console.log(`[デバッグ] stateDidChange: ${stateDidChange}`);
    //     console.log("常に Rust の描画関数を呼び出します...");
    //     gameApp.render_game_rust(); // ← gameLoop に移動！
    //     console.log("  render_game_rust 呼び出し完了。");
    // } catch (e) {
    //     console.error("メッセージ処理またはRustレンダリング呼び出し中にエラー:", e);
    // }

    // ★追加: メッセージ処理はここで行う (描画とは別タイミング) ★
    //     描画は毎フレームやるけど、メッセージ処理はここ (1秒ごと) でいいかも？
    //     もっと頻繁にしたいなら gameLoop に移してもOK
    try {
        // ★修正: Rust側のメッセージ処理を呼び出し、戻り値を受け取る！★
        //   戻り値は Option<usize> 型。usize は拒否されたカードのIDだよ。
        //   (JSでは number | undefined として扱われる)
        const rejected_card_id = gameApp.process_received_messages();

        // ★追加: 戻り値をチェックして、拒否されたカードがあれば警告を出す！★
        //   rejected_card_id が undefined じゃなければ (つまり Some(id) だったら)
        if (rejected_card_id !== undefined) {
            // 警告メッセージをコンソールに出力！⚠️
            // どのカードの移動がダメだったか ID も表示するよ。
            console.warn(`⚠️ サーバーから移動が拒否されました！ (カードID: ${rejected_card_id}) ルールを確認してね！`);
            // TODO: ここに、もっとリッチなフィードバック処理を追加できるよ！
            //   例: アラートを表示する (alert(...)), カードを元の位置に戻すアニメーションを開始する、など
        }

    } catch (e) {
        console.error("メッセージ処理中にエラー:", e);
    }
}

// --- ★ Window 用の MouseMove イベントハンドラー ★ ---
function handleMouseMove(event) {
    // ドラッグ中でなければ何もしない！
    if (!isDragging || !gameApp || draggedEntityId === null) {
        return;
    }

    // console.log("[DEBUG] handleMouseMove 開始"); // ログが多すぎるのでコメントアウト推奨

    // Canvas 座標を取得
    const coords = getCanvasCoordinates(event);
    if (!coords) {
        // console.log("[DEBUG] handleMouseMove: 座標取得失敗"); // ログが多すぎるのでコメントアウト推奨
        return;
    }

    // console.log(`[DEBUG] handleMouseMove: 座標 (${coords.x.toFixed(2)}, ${coords.y.toFixed(2)}), EntityID: ${draggedEntityId}`); // ログが多すぎるのでコメントアウト推奨

    // Rust 側に座標を渡して、ドラッグ中のカード位置を更新する
    try {
        gameApp.update_dragged_position(draggedEntityId, coords.x, coords.y);
    } catch (error) {
        console.error("💥 gameApp.update_dragged_position 呼び出しエラー:", error);
        // エラーが起きてもドラッグは継続？一旦継続。
    }
    // console.log("[DEBUG] handleMouseMove 終了"); // ログが多すぎるのでコメントアウト推奨
}

// --- ★ Window 用の MouseUp イベントハンドラー ★ ---
function handleMouseUp(event) {
    // ★ログ追加: 関数開始★
    console.log("[DEBUG] handleMouseUp 開始");

    // ドラッグ中でなければ何もしない！
    if (!isDragging || !gameApp || draggedEntityId === null) {
        console.log("[DEBUG] handleMouseUp: ドラッグ中でないため処理をスキップ");
        return;
    }

    console.log("[DEBUG] handleMouseUp: ドラッグ終了処理を実行します");

    // ★★★ 重要: まずリスナーをデタッチ！ ★★★
    // これを先にやらないと、クリックイベントが誤発火する可能性がある
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
    console.log("[DEBUG] handleMouseUp: Window リスナー削除完了");

    // Canvas 座標を取得
    const coords = getCanvasCoordinates(event);
    if (!coords) {
        console.error("[DEBUG] handleMouseUp: 座標取得失敗！ ドラッグ終了処理を中断します");
        // 念のためドラッグ状態はリセット
        isDragging = false;
        draggedEntityId = null;
        return;
    }
    console.log(`[DEBUG] handleMouseUp: 座標 (${coords.x.toFixed(2)}, ${coords.y.toFixed(2)})`);

    // Rust 側にドラッグ終了を通知
    try {
        console.log(`[DEBUG] handleMouseUp: ドラッグ対象エンティティ ID: ${draggedEntityId}`);
        // ★ログ追加: Rust 呼び出し直前★
        console.log(`[DEBUG] handleMouseUp: gameApp.handle_drag_end(${draggedEntityId}, ${coords.x.toFixed(2)}, ${coords.y.toFixed(2)}) 呼び出し`);
        gameApp.handle_drag_end(draggedEntityId, coords.x, coords.y);
        console.log("[DEBUG] handleMouseUp: handle_drag_end 呼び出し成功");
    } catch (error) {
        console.error("💥 gameApp.handle_drag_end 呼び出しエラー:", error);
        // エラーが起きてもドラッグ状態はリセットする
    }

    // ドラッグ状態をリセット
    console.log("[DEBUG] handleMouseUp: ドラッグ状態をリセット");
    isDragging = false;
    draggedEntityId = null;

    // ★ログ追加: 関数終了★
    console.log("[DEBUG] handleMouseUp 終了");
}

// --- メイン処理の呼び出し --- (DOMContentLoaded を待つ)
// DOM の準備ができたら main() 関数を実行するイベントリスナーを設定
document.addEventListener('DOMContentLoaded', () => {
    console.log("⏳ DOMContentLoaded イベントリスナーを設定。DOM 準備完了後に main() を実行します。");
    main();
}); 