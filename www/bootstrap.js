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

        // --- ここから Canvas クリックイベントの処理を追加！ ---
        console.log("Canvas クリックリスナー設定中...🖱️");

        // 1. HTML から Canvas 要素を取得！
        // `document.getElementById()` は、HTML の中で指定された ID を持つ要素を探してきてくれる関数だよ。
        // ID が 'game-canvas' の要素 (index.html で <canvas id="game-canvas"> ってなってるやつ) をゲット！
        const canvas = document.getElementById('game-canvas');

        // 2. Canvas がちゃんと見つかったかチェック！ (もし見つからなかったらエラー出す)
        if (!canvas) {
            console.error("致命的エラー: ID 'game-canvas' の Canvas 要素が見つかりません！😱 index.html に存在するか確認してください！");
            // Canvas がないと何もできないので処理中断するけど、init 関数自体は完了させたいので return はしない。
        } else {
            console.log("Canvas 要素発見！👍");

            // 3. Canvas にクリックイベントリスナーを追加！
            // `addEventListener('click', callback)` は、指定した要素 (canvas) で
            // 特定のイベント ('click') が発生した時に、指定した関数 (callback) を実行するように設定するメソッドだよ。
            // ここではアロー関数 `(event) => { ... }` をコールバックとして使ってる。
            // アロー関数は `this` の扱いがシンプルで書きやすいからモダン JS ではよく使うよ！✨
            canvas.addEventListener('click', (event) => {
                // --- クリックイベントが発生した時の処理をここに書く！ ---
                console.log("Canvas クリック！ ✨ イベント:", event); // クリックされたことをログに出力！ event オブジェクトの中身も見てみよ！

                // 4. Canvas の画面上の位置とサイズを取得！
                // `getBoundingClientRect()` は、要素 (canvas) が画面のどこに表示されてるかの情報 (左上の x, y 座標、幅、高さなど) をくれるメソッドだよ。
                // これがないと、画面全体のどこをクリックしたか分かっても、それが Canvas の中のどこなのか正確に計算できないんだ。📐
                const rect = canvas.getBoundingClientRect();
                // console.log("Canvas bounding rect:", rect); // デバッグ用に矩形情報をログに！

                // 5. クリックされた画面上の座標を取得！
                // `event.clientX` と `event.clientY` は、クリックされた瞬間のマウスカーソルの
                // X座標とY座標 (ブラウザウィンドウの左上からの距離) を教えてくれるプロパティだよ。
                const mouseX = event.clientX;
                const mouseY = event.clientY;
                // console.log(`Mouse click position (viewport): x=${mouseX}, y=${mouseY}`); // デバッグ用

                // 6. Canvas 内のローカル座標を計算！ここがキモ！💡
                // 画面上のクリック座標 (mouseX, mouseY) から、Canvas の左上の画面座標 (rect.left, rect.top) を
                // 引き算することで、Canvas の左上を (0, 0) としたときの相対的な座標 (ローカル座標) が求まる！
                // これで Canvas の中のどこがクリックされたか分かるね！🎯
                const canvasX = mouseX - rect.left;
                const canvasY = mouseY - rect.top;

                // 7. 計算結果をコンソールに出力！
                // `` (バッククォート) で囲むと、文字列の中に ${変数名} って書くだけで変数の値を埋め込めるテンプレートリテラルが使えるよ！超便利！💖
                console.log(`>>> Canvas 内クリック座標: x=${canvasX.toFixed(2)}, y=${canvasY.toFixed(2)} <<<`); // `toFixed(2)` で小数点以下2桁まで表示！見やすい！

                // --- TODO: 次のステップ！ ---
                // ここで計算した canvasX, canvasY を使って、どのカードやスタックがクリックされたか判定するロジックを
                // Rust 側 (gameApp のメソッド) に渡して呼び出すことになるよ！
                // 例: gameApp.handle_canvas_click(canvasX, canvasY); みたいな感じ！ (これはまだ実装してない！)
                // 今回はログ出力まで！👍
            });

            // 🌟🌟🌟 Canvas にダブルクリックイベントリスナーを追加！ 🌟🌟🌟
            canvas.addEventListener('dblclick', (event) => {
                // --- ダブルクリックイベントが発生した時の処理 ---
                console.log("Canvas ダブルクリック！ 🖱️🖱️ イベント:", event); // ダブルクリックログ

                // まず、gameApp がちゃんと使えるかチェック！ (ないと Rust 呼べない！)
                if (!gameApp) {
                    console.error("GameApp が初期化されていません！ダブルクリックを処理できません。");
                    return; // 何もせず終了
                }

                // 1. Canvas の画面上の位置とサイズを取得
                const rect = canvas.getBoundingClientRect();

                // 2. ダブルクリックされた画面上の座標を取得
                const mouseX = event.clientX;
                const mouseY = event.clientY;

                // 3. Canvas 内のローカル座標を計算
                const canvasX = mouseX - rect.left;
                const canvasY = mouseY - rect.top;
                console.log(`>>> Canvas 内ダブルクリック座標: x=${canvasX.toFixed(2)}, y=${canvasY.toFixed(2)} <<<`); // 座標ログ

                // 4. ★★★ Rust に問い合わせて、クリック座標にあるカードのIDを取得！ ★★★
                // さっき Rust 側に作った get_entity_id_at(x, y) 関数を呼び出すよ！
                // この関数は、もしカードがあればそのID (number) を、なければ undefined を返すよ。
                let clickedEntityId = undefined; // 結果を保存する変数を用意 (最初は undefined)
                try {
                    console.log(`  📞 Rust 呼び出し中: gameApp.get_entity_id_at(${canvasX.toFixed(2)}, ${canvasY.toFixed(2)})`);
                    clickedEntityId = gameApp.get_entity_id_at(canvasX, canvasY);
                    console.log(`  Rust からの応答 Entity ID: ${clickedEntityId}`); // Rust からの返り値をログに！
                } catch (error) {
                    console.error("💥 gameApp.get_entity_id_at 呼び出しエラー:", error);
                    return; // エラーが起きたら処理中断
                }

                // 5. ★★★ カードIDが取得できたら、Rustのダブルクリック処理を呼び出す！ ★★★
                // clickedEntityId が undefined じゃなかったら = カードが見つかったってこと！
                if (clickedEntityId !== undefined) {
                    // カードが見つかった場合の処理
                    console.log(`  ✅ カード発見！ Entity ID: ${clickedEntityId}。Rust のダブルクリックハンドラーを呼び出します...`);
                    try {
                        // Rust 側の GameApp::handle_double_click(entity_id) 関数を呼び出す！
                        // これで、Rust 側で自動移動のロジックが動くはず！ (移動先が見つかればメッセージ送信！)
                        console.log(`  🚀 Rust 呼び出し中: gameApp.handle_double_click(${clickedEntityId})`);
                        gameApp.handle_double_click(clickedEntityId);
                        console.log("  Rust の handle_double_click 関数呼び出し成功！");
                        // 注意: 画面の更新は、この後サーバーから GameStateUpdate が来て、
                        //       それを受け取って Rust 側が再描画 (render_game_rust) をすることで行われる想定だよ！
                        //       だから、ここではJS側で描画処理は呼ばないよ。
                    } catch (error) {
                        console.error("💥 gameApp.handle_double_click 呼び出しエラー:", error);
                        // エラーが起きても処理は続行する（かもしれない）
                    }
                } else {
                    // カードが見つからなかった場合 (スタックか背景をダブルクリックした)
                    console.log("  🤷 この座標にカードは見つかりませんでした。自動移動のためのダブルクリックは無視します。");
                }
            });

            console.log("Canvas クリックリスナー設定完了！クリック待機中！ ✅🖱️");
            console.log("Canvas ダブルクリックリスナー設定完了！ダブルクリック待機中！ ✅🖱️🖱️");
        } // if (canvas) の終わり

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
        const stateDidChange = gameApp.process_received_messages();
        if (stateDidChange) {
            console.log("Rust によると状態が変更されました。Rust の描画関数を呼び出します...");
            // ★修正: renderGame() の代わりに render_game_rust() を呼び出す！★
            gameApp.render_game_rust();
        }
    } catch (e) {
        console.error("メッセージ処理またはRustレンダリング呼び出し中にエラー:", e);
    }
}

// --- ★ 新しい関数: ゲーム状態を描画する！ --- ★
// 不要になったのでコメントアウト (または後で完全に削除！)
/*
function renderGame() {
    console.log("🎨 Rendering game state... (JS version - DEPRECATED)");
    if (!gameApp) {
        console.error("描画失敗: gameApp が初期化されていません。");
        return;
    }

    try {
        // 1. Rust から最新のゲーム状態 (JSON) を取得
        const stateJson = gameApp.get_world_state_json();
        const gameState = JSON.parse(stateJson);

        // エラーがないかチェック (Rust側でエラーJSONを返す場合)
        if (gameState.error) {
            console.error("サーバーからエラーが返されました: ", gameState.error, gameState.details);
            // TODO: ユーザーにエラー表示
            gameAreaDiv.innerHTML = `<p style="color: red;">ゲーム状態の取得に失敗しました: ${gameState.error}</p>`;
            return;
        }

        // 2. game-area の中身を一旦空にする
        gameAreaDiv.innerHTML = ''; // 古いカード要素を削除！

        // 3. 状態データ (gameState.cards) を元にカード要素を作成して配置
        if (gameState.cards && Array.isArray(gameState.cards)) {
            console.log(`  カード ${gameState.cards.length} 枚を描画中...`);
            gameState.cards.forEach(cardData => {
                // カード要素 (div) を作成
                const cardElement = document.createElement('div');
                cardElement.classList.add('card'); // 基本クラス
                cardElement.dataset.entityId = cardData.entity_id; // data-* 属性でエンティティIDを保持

                // カードの位置を計算 (CSS で position: absolute が前提！)
                const position = calculateCardPosition(cardData);
                cardElement.style.left = `${position.x}px`;
                cardElement.style.top = `${position.y}px`;
                // z-index も設定して重なり順を制御！ order が大きいほど手前
                cardElement.style.zIndex = cardData.order;

                // カードの内容 (スートとランク or 裏面)
                if (cardData.is_face_up) {
                    cardElement.classList.add('face-up');
                    cardElement.classList.add(`suit-${cardData.suit.toLowerCase()}`);
                    cardElement.classList.add(`rank-${cardData.rank.toLowerCase()}`);
                    const suitSymbol = getSuitSymbol(cardData.suit);
                    const rankText = getRankText(cardData.rank);
                    cardElement.innerHTML = `
                        <span class="rank">${rankText}</span>
                        <span class="suit">${suitSymbol}</span>
                    `;
                } else {
                    cardElement.classList.add('face-down');
                    cardElement.innerHTML = '';
                }

                // --- クリックイベントリスナーを設定 --- (変更なし)
                cardElement.addEventListener('click', () => {
                    handleCardClick(cardData, cardElement);
                });

                // --- ダブルクリックイベントリスナーを設定 --- (変更なし)
                cardElement.addEventListener('dblclick', () => {
                    handleCardDoubleClick(cardData, cardElement);
                });

                // --- ★ここから追加: マウスダウンイベントリスナーを設定 (ドラッグ開始)★ ---
                cardElement.addEventListener('mousedown', (event) => {
                    handleMouseDown(event, cardData, cardElement);
                });
                // --- ★追加ここまで★ ---

                // 作成したカード要素をゲームエリアに追加
                gameAreaDiv.appendChild(cardElement);
            });
            console.log("  カード要素をゲームエリアに追加しました。");
        } else {
            console.warn("gameState に cards 配列が含まれていません。");
            gameAreaDiv.innerHTML = '<p>カード情報がありません。</p>';
        }

    } catch (e) {
        console.error("ゲーム状態の描画中にエラーが発生しました:", e);
        gameAreaDiv.innerHTML = '<p style="color: red;">ゲーム画面の描画中にエラーが発生しました。</p>';
    }
}
*/

// --- ★ 新しい関数: カードクリック処理 ★ ---
function handleCardClick(cardData, cardElement) {
    console.log(`🖱️ カードクリック！ Entity ID: ${cardData.entity_id}`, cardData);

    // TODO: クリックされたカードに応じたゲームロジックを呼び出す
    // 例: gameApp.card_clicked(cardData.entity_id);

    // --- 見た目の選択状態を切り替える (簡易版) ---
    // 他のカードから selected クラスを削除
    document.querySelectorAll('#game-area .card.selected').forEach(el => {
        el.classList.remove('selected');
    });
    // クリックされたカードに selected クラスを追加
    cardElement.classList.add('selected');
    console.log('  クリックされたカードに .selected クラスを追加しました。');
}

// --- ★ 新しい関数: カードダブルクリック処理 ★ ---
function handleCardDoubleClick(cardData, cardElement) {
    console.log(`🖱️🖱️ カードダブルクリック！ Entity ID: ${cardData.entity_id}`, cardData);

    // gameApp が存在するかチェック
    if (!gameApp) {
        console.error("GameApp が初期化されていません。ダブルクリックを処理できません。");
        return;
    }

    // 表向きのカードだけ自動移動の対象にする（ソリティアのルール的に）
    if (cardData.is_face_up) {
        try {
            // Rust側の handle_double_click を呼び出す！ Entity ID を渡すよ！
            console.log(`  gameApp.handle_double_click をエンティティ ID: ${cardData.entity_id} で呼び出し中`);
            gameApp.handle_double_click(cardData.entity_id);
            console.log("  gameApp.handle_double_click 呼び出し成功。");
            // 注: Rust側でメッセージが送信された後、サーバーからの GameStateUpdate を待って
            //     renderGame() が呼ばれることで画面が更新されるはず！なので、ここでは描画しない。
        } catch (error) {
            console.error("gameApp.handle_double_click 呼び出し中にエラー:", error);
            // 必要ならユーザーにエラー表示
        }
    } else {
        console.log("  カードは裏向きなので、自動移動のためのダブルクリックは無視します。");
    }
}

// --- ★ 新しい関数: カードドラッグ開始処理 (mousedown) ★ ---
function handleMouseDown(event, cardData, cardElement) {
    // ドラッグできるのは表向きのカードのみ (今は Stock 以外全部OKにしてみる)
    if (cardData.is_face_up && cardData.stack_type !== 'Stock') {
        console.log(`🖱️ カードドラッグ開始検出 Entity ID: ${cardData.entity_id}`);
        event.preventDefault();
        isDragging = true;
        draggedCardElement = cardElement;
        draggedEntityId = cardData.entity_id;
        const rect = cardElement.getBoundingClientRect();
        offsetX = event.clientX - rect.left;
        offsetY = event.clientY - rect.top;
        cardElement.classList.add('dragging');
        cardElement.style.cursor = 'grabbing';

        // --- ★ここから追加: mousemove と mouseup リスナーを document に追加★ ---
        document.addEventListener('mousemove', handleMouseMove);
        // mouseup のリスナーもここで追加しちゃう（次のステップ用だけど一緒にやっとく！）
        document.addEventListener('mouseup', handleMouseUp);
        // --- ★追加ここまで★ ---

    } else {
        console.log(`カード Entity ID: ${cardData.entity_id} はドラッグできません (裏向きまたは山札)。`);
    }
}

// --- ★ 新しい関数: カードドラッグ中の処理 (mousemove) ★ --- (修正版！)
function handleMouseMove(event) {
    // ドラッグ中でなければ何もしない
    if (!isDragging || !draggedCardElement) return;

    // ゲームエリアの位置情報を取得 (座標変換のため)
    const gameAreaRect = gameAreaDiv.getBoundingClientRect();

    // マウスの現在の画面上の座標 (clientX, clientY) から、
    // ドラッグ開始時のズレ (offsetX, offsetY) を引いて、
    // カードの左上が「画面上のどこに来るべきか」を計算する。
    const desiredViewportX = event.clientX - offsetX;
    const desiredViewportY = event.clientY - offsetY;

    // 「画面上の座標」を「ゲームエリア内の座標」に変換する！
    // (画面上の座標 - ゲームエリアの左上の画面上の座標 = ゲームエリア内の座標)
    const newX = desiredViewportX - gameAreaRect.left;
    const newY = desiredViewportY - gameAreaRect.top;

    // 計算したゲームエリア内の座標をカードのスタイルに設定！
    draggedCardElement.style.left = `${newX}px`;
    draggedCardElement.style.top = `${newY}px`;
}

// --- ★ 新しい関数: カードドラッグ終了処理 (mouseup) ★ --- (send_make_move 呼び出し追加版！)
function handleMouseUp(event) {
    // ドラッグ中でなければ何もしない
    if (!isDragging || !draggedCardElement) return;

    const currentDraggedEntityId = draggedEntityId; // リスナー削除前にIDを保持

    console.log(`🖱️ カードドラッグ終了検出 Entity ID: ${currentDraggedEntityId} at (${event.clientX}, ${event.clientY})`);

    // ドラッグ中の見た目を元に戻す
    draggedCardElement.classList.remove('dragging');
    draggedCardElement.style.cursor = 'grab';

    // ★超重要: document に追加したリスナーを削除！★
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    console.log("  mousemove と mouseup リスナーを document から削除しました。");

    // --- ドロップ位置から移動先スタックを判定 --- ★ 修正箇所
    const targetStack = findDropTargetStack(event.clientX, event.clientY);
    if (targetStack) {
        console.log("  ドロップターゲット特定:", targetStack);

        // --- ★ここから追加: MakeMove メッセージを送信！★ ---
        if (gameApp && currentDraggedEntityId !== null) {
            try {
                // targetStack オブジェクトを JSON 文字列に変換する必要がある！
                const targetStackJson = JSON.stringify(targetStack);
                console.log(`  gameApp.send_make_move をエンティティ ID: ${currentDraggedEntityId}, ターゲット: ${targetStackJson} で呼び出し中`);
                gameApp.send_make_move(currentDraggedEntityId, targetStackJson);
                console.log("  gameApp.send_make_move 呼び出し成功。");
            } catch (error) {
                console.error("gameApp.send_make_move 呼び出し中にエラー:", error);
            }
        } else {
            console.error("移動を送信できません: gameApp が準備できていないか、draggedEntityId が null です。");
        }
        // --- ★追加ここまで★ ---

    } else {
        console.log("  有効なターゲットエリア外にドロップされました。");
        // TODO: カードを元の位置に戻すアニメーションとか？ (今回は renderGame を呼べば状態更新で戻るはず)
        //       即座に見た目を戻したい場合は、元の位置を保存しておいてスタイルを戻す必要あり
        //       今はサーバーからの状態更新を待つ形にする
    }

    // ドラッグ状態をリセット
    isDragging = false;
    draggedCardElement = null;
    draggedEntityId = null;
    offsetX = 0;
    offsetY = 0;
    console.log("  ドラッグ状態をリセットしました。");
}

// --- ★ 新しい関数: ドロップ位置から移動先スタックを判定するロジック ★ ---
function findDropTargetStack(dropX, dropY) {
    const cardWidth = 72;
    const cardHeight = 96;
    const horizontalSpacing = 10;
    const verticalSpacing = 15;

    // ゲームエリアの座標を取得 (ドロップ座標をエリア内座標に変換するため)
    const gameAreaRect = gameAreaDiv.getBoundingClientRect();
    const dropAreaX = dropX - gameAreaRect.left;
    const dropAreaY = dropY - gameAreaRect.top;

    // Check Foundations (0-3)
    for (let i = 0; i < 4; i++) {
        const foundationX = 10 + (cardWidth + horizontalSpacing) * (3 + i);
        const foundationY = 10;
        if (dropAreaX >= foundationX && dropAreaX <= foundationX + cardWidth &&
            dropAreaY >= foundationY && dropAreaY <= foundationY + cardHeight) {
            console.log(`ドロップ候補: 組札エリア ${i}`);
            // StackType オブジェクトを返す (Rust 側の形式に合わせる)
            return { Foundation: i };
        }
    }

    // Check Tableau drop zones (0-6) - Checking the top slot area
    for (let i = 0; i < 7; i++) {
        const tableauX = 10 + (cardWidth + horizontalSpacing) * i;
        const tableauY = 10 + cardHeight + verticalSpacing; // 列の開始Y座標
        // 判定エリア: とりあえず列の開始位置のカード1枚分の高さにする
        if (dropAreaX >= tableauX && dropAreaX <= tableauX + cardWidth &&
            dropAreaY >= tableauY && dropAreaY <= tableauY + cardHeight) {
            console.log(`ドロップ候補: 場札エリア ${i}`);
            // StackType オブジェクトを返す
            return { Tableau: i };
        }
        // TODO: 将来的には、タブローの列にカードがあれば、一番下のカードのエリアも判定対象に加えるべき
    }

    // console.log("Drop outside any defined stack area.");
    return null; // どのエリアにもドロップされなかった
}

// --- ヘルパー関数: カードの表示位置を計算 --- (修正版！)
// 不要になったのでコメントアウト (または後で完全に削除！)
/*
function calculateCardPosition(cardData) {
    const cardWidth = 72; // カードの幅 (CSSと合わせる必要あり)
    const cardHeight = 96; // カードの高さ
    const horizontalSpacing = 10; // 横の間隔
    const verticalSpacing = 15;   // 縦の間隔 (重ねる場合)
    const tableauVerticalOffset = 25; // 場札の縦の重なり具合
    const wasteHorizontalOffset = 20; // ★追加: 捨て札の横の重なり具合

    let baseX = 10;
    let baseY = 10;

    switch (cardData.stack_type) {
        case 'Stock':
            baseX = 10;
            baseY = 10;
            break;
        case 'Waste':
            // ★修正: 山札の右隣に、order に応じて少しずつ横にずらす
            baseX = 10 + cardWidth + horizontalSpacing + (cardData.order * wasteHorizontalOffset);
            baseY = 10;
            break;
        case 'Foundation':
            baseX = 10 + (cardWidth + horizontalSpacing) * (3 + (cardData.stack_index || 0));
            baseY = 10;
            break;
        case 'Tableau':
            baseX = 10 + (cardWidth + horizontalSpacing) * (cardData.stack_index || 0);
            baseY = 10 + cardHeight + verticalSpacing + (cardData.order * tableauVerticalOffset);
            break;
        default:
            console.warn(`未知の stack_type: ${cardData.stack_type}`);
            break;
    }

    return { x: baseX, y: baseY };
}
*/

// --- ヘルパー関数: スート記号を取得 ---
// 不要になったのでコメントアウト (または後で完全に削除！)
/*
function getSuitSymbol(suitName) {
    switch (suitName) {
        case 'Heart': return '♥';
        case 'Diamond': return '♦';
        case 'Club': return '♣';
        case 'Spade': return '♠';
        default: return '?';
    }
}
*/

// --- ヘルパー関数: ランク文字列を取得 ---
// 不要になったのでコメントアウト (または後で完全に削除！)
/*
function getRankText(rankName) {
    // 基本はそのままだけど、Ace, King, Queen, Jack は A, K, Q, J にしたい
    switch (rankName) {
        case 'Ace': return 'A';
        case 'King': return 'K';
        case 'Queen': return 'Q';
        case 'Jack': return 'J';
        case 'Ten': return '10';
        case 'Nine': return '9';
        case 'Eight': return '8';
        case 'Seven': return '7';
        case 'Six': return '6';
        case 'Five': return '5';
        case 'Four': return '4';
        case 'Three': return '3';
        case 'Two': return '2';
        default: return rankName.charAt(0); // 不明な場合は最初の文字？
    }
}
*/

// --- 実行開始！ ---
main(); 