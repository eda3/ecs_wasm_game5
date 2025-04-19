// www/bootstrap.js

// まず、wasm-pack が生成した JS ファイルから必要なものをインポートするよ！
// `init` 関数: WASM モジュールを非同期で初期化する関数。
// `GameApp` クラス: Rust 側で #[wasm_bindgen] を付けた構造体が JS ではクラスみたいに見える！
// パスはプロジェクトの構成に合わせてね (普通は `../pkg/` の下にあるはず)
import init, { GameApp } from '../pkg/ecs_wasm_game5.js';

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
// const gameAreaDiv = document.getElementById('game-area'); // ゲーム描画用 (まだ使わないけど)

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
        gameApp.deal_initial_cards(); // Rust 側の deal_initial_cards() を呼び出す！
        // TODO: 配った後に状態を取得して描画するとか？
        // dealButton.disabled = true; // 一回配ったら無効にする？
    });

    // 「状態取得(Console)」ボタン
    getStateButton.addEventListener('click', () => {
        console.log("🖱️ Get State button clicked");
        try {
            const stateJson = gameApp.get_world_state_json(); // Rust 側のメソッド呼び出し
            console.log("--- World State (JSON) ---");
            console.log(JSON.parse(stateJson)); // JSON 文字列をパースしてオブジェクトとして表示
            console.log("-------------------------");
        } catch (e) {
            console.error("状態の取得またはJSONパースに失敗: ", e);
        }
    });
}

// --- 接続状態などを表示する関数 ---
function updateStatusDisplay() {
    if (!gameApp) return; // gameApp がまだなければ何もしない

    try {
        // Rust 側からデバッグ用の接続状態とプレイヤーIDを取得
        const status = gameApp.get_connection_status_debug();
        const playerId = gameApp.get_my_player_id_debug(); // Option<u32> は JS では number | null になる

        connectionStatusSpan.textContent = status;
        playerIdSpan.textContent = playerId !== null ? playerId.toString() : '未参加';

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


// --- 実行開始！ ---
main(); 