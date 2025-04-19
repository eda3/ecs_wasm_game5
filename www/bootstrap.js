// www/bootstrap.js

// まず、wasm-pack が生成した JS ファイルから必要なものをインポートするよ！
// `init` 関数: WASM モジュールを非同期で初期化する関数。
// `GameApp` クラス: Rust 側で #[wasm_bindgen] を付けた構造体が JS ではクラスみたいに見える！
// パスはプロジェクトの構成に合わせてね (http-server がルートを配信するので、ルートからの絶対パス /pkg/ になる)
import init, { GameApp } from '/pkg/ecs_wasm_game5.js';

// グローバルスコープ (どこからでもアクセスできる場所) に gameApp インスタンスを保持する変数を用意するよ。
// 最初は null (まだ無い状態) にしておく。
let gameApp = null;

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
        console.log("🖱️ Connect button clicked");
        gameApp.connect(); // Rust 側の connect() を呼び出す！
        // TODO: 接続試行中の表示とか？
    });

    // 「ゲームに参加」ボタン
    joinButton.addEventListener('click', () => {
        console.log("🖱️ Join button clicked");
        // とりあえず仮のプレイヤー名で参加！ 本当は入力させるべきだね。
        const playerName = prompt("プレイヤー名を入力してください:", "ギャルゲーマー");
        if (playerName) { // prompt でキャンセルされなかったら
            gameApp.send_join_game(playerName); // Rust 側の send_join_game() を呼び出す！
            // TODO: 参加後のボタン状態変更など
        }
    });

    // 「カードを配る」ボタン
    dealButton.addEventListener('click', () => {
        console.log("🖱️ Deal button clicked");
        try {
            gameApp.deal_initial_cards(); // Rust 側の deal_initial_cards() を呼び出す！
            console.log("🃏 Cards dealt on Rust side.");
            renderGame(); // カードを配った後に画面を再描画！✨
        } catch (e) {
            console.error("カード配布または描画中にエラー:", e);
        }
    });

    // 「状態取得(Console)」ボタン (描画も行うように変更！)
    getStateButton.addEventListener('click', () => {
        console.log("🖱️ Get State button clicked");
        try {
            const stateJson = gameApp.get_world_state_json(); // Rust 側のメソッド呼び出し
            console.log("--- World State (JSON) ---");
            console.log(JSON.parse(stateJson)); // JSON 文字列をパースしてオブジェクトとして表示
            console.log("-------------------------");
            renderGame(); // 状態取得後にも画面を描画！✨
        } catch (e) {
            console.error("状態の取得、JSONパース、または描画中にエラー: ", e);
        }
    });
}

// --- 接続状態などを表示する関数 ---
function updateStatusDisplay() {
    if (!gameApp) return; // gameApp がまだなければ何もしない

    try {
        // Rust 側からデバッグ用の接続状態とプレイヤーIDを取得
        const status = gameApp.get_connection_status_debug();
        const playerId = gameApp.get_my_player_id_debug(); // Option<u32> は JS では number | undefined になる

        connectionStatusSpan.textContent = status;
        playerIdSpan.textContent = playerId !== undefined ? playerId.toString() : '未参加';

        // --- 接続状態に応じてボタンの有効/無効を切り替え ---
        if (status === 'Connected') {
            connectButton.disabled = true; // 接続済みなら接続ボタンは無効
            joinButton.disabled = false;   // 参加ボタンは有効
            dealButton.disabled = false;   // カード配布ボタンも有効 (仮。本当はゲーム状態による)
        } else if (status === 'Connecting') {
            connectButton.disabled = true;
            joinButton.disabled = true;
            dealButton.disabled = true;
        } else { // Disconnected or Error
            connectButton.disabled = false; // 未接続なら接続ボタンは有効
            joinButton.disabled = true;
            dealButton.disabled = true;
        }
        // TODO: ゲーム参加後 (playerId が設定された後) の状態制御も追加！

    } catch (e) {
        console.error("ステータス更新中にエラー:", e);
        connectionStatusSpan.textContent = "エラー";
        playerIdSpan.textContent = "-";
        // エラー時はボタンを無効化
        connectButton.disabled = true;
        joinButton.disabled = true;
        dealButton.disabled = true;
    }

    // 受信メッセージを処理する (これも定期的に呼ぶのが簡単かな？)
    try {
        gameApp.process_received_messages(); // Rust側のメッセージ処理を呼び出す
    } catch (e) {
        console.error("メッセージ処理中にエラー:", e);
    }
}

// --- ★ 新しい関数: ゲーム状態を描画する！ --- ★
function renderGame() {
    console.log("🎨 Rendering game state...");
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
            console.log(`  Rendering ${gameState.cards.length} cards...`);
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

                // --- ★ ここから追加: クリックイベントリスナーを設定 ★ ---
                cardElement.addEventListener('click', () => {
                    handleCardClick(cardData, cardElement);
                });
                // --- ★ 追加ここまで ★ ---

                // 作成したカード要素をゲームエリアに追加
                gameAreaDiv.appendChild(cardElement);
            });
            console.log("  Card elements added to game area.");
        } else {
            console.warn("gameState に cards 配列が含まれていません。");
            gameAreaDiv.innerHTML = '<p>カード情報がありません。</p>';
        }

    } catch (e) {
        console.error("ゲーム状態の描画中にエラーが発生しました:", e);
        gameAreaDiv.innerHTML = '<p style="color: red;">ゲーム画面の描画中にエラーが発生しました。</p>';
    }
}

// --- ★ 新しい関数: カードクリック処理 ★ ---
function handleCardClick(cardData, cardElement) {
    console.log(`🖱️ Card clicked! Entity ID: ${cardData.entity_id}`, cardData);

    // TODO: クリックされたカードに応じたゲームロジックを呼び出す
    // 例: gameApp.card_clicked(cardData.entity_id);

    // --- 見た目の選択状態を切り替える (簡易版) ---
    // 他のカードから selected クラスを削除
    document.querySelectorAll('#game-area .card.selected').forEach(el => {
        el.classList.remove('selected');
    });
    // クリックされたカードに selected クラスを追加
    cardElement.classList.add('selected');
    console.log('  Added .selected class to clicked card.');
}

// --- ヘルパー関数: カードの表示位置を計算 --- (超簡易版！)
function calculateCardPosition(cardData) {
    const cardWidth = 72; // カードの幅 (CSSと合わせる必要あり)
    const cardHeight = 96; // カードの高さ
    const horizontalSpacing = 10; // 横の間隔
    const verticalSpacing = 15;   // 縦の間隔 (重ねる場合)
    const tableauVerticalOffset = 25; // 場札の重なり具合

    let baseX = 10;
    let baseY = 10;

    switch (cardData.stack_type) {
        case 'Stock':
            // 山札は左上に固めておく (雑)
            baseX = 10;
            baseY = 10; // order で少しずらす？今回は固定
            break;
        case 'Waste':
            // 捨て札は山札の右隣 (雑)
            baseX = 10 + cardWidth + horizontalSpacing;
            baseY = 10; // order で少しずらす？今回は固定
            break;
        case 'Foundation':
            // 上がり札は右上に4つ並べる (雑)
            baseX = 10 + (cardWidth + horizontalSpacing) * (3 + (cardData.stack_index || 0)); // 3番目以降に配置
            baseY = 10;
            break;
        case 'Tableau':
            // 場札は7列、下に重ねていく (雑)
            baseX = 10 + (cardWidth + horizontalSpacing) * (cardData.stack_index || 0);
            baseY = 10 + cardHeight + verticalSpacing + (cardData.order * tableauVerticalOffset);
            break;
        default:
            console.warn(`未知の stack_type: ${cardData.stack_type}`);
            break;
    }

    return { x: baseX, y: baseY };
}

// --- ヘルパー関数: スート記号を取得 ---
function getSuitSymbol(suitName) {
    switch (suitName) {
        case 'Heart': return '♥';
        case 'Diamond': return '♦';
        case 'Club': return '♣';
        case 'Spade': return '♠';
        default: return '?';
    }
}

// --- ヘルパー関数: ランク文字列を取得 ---
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

// --- 実行開始！ ---
main(); 