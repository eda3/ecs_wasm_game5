// server/ws_server.js

// ws パッケージをインポート
const WebSocket = require('ws');

// サーバーをポート 8101 で起動
const wss = new WebSocket.Server({ port: 8101 });

console.log('🚀 WebSocket Server is running on ws://localhost:8101');

// --- ゲーム状態管理 (簡易版) ---
let gameState = {
    players: [], // { id: PlayerId, name: String }
    cards: [],   // CardData オブジェクトの配列
    // TODO: 他に必要な状態
};

// --- カード初期化ロジック --- ★ここから実装★
const SUITS = ['Heart', 'Diamond', 'Club', 'Spade'];
const RANKS = ['Ace', 'Two', 'Three', 'Four', 'Five', 'Six', 'Seven', 'Eight', 'Nine', 'Ten', 'Jack', 'Queen', 'King'];

// Fisher-Yates (aka Knuth) Shuffle アルゴリズム
function shuffle(array) {
    let currentIndex = array.length, randomIndex;
    // While there remain elements to shuffle.
    while (currentIndex !== 0) {
        // Pick a remaining element.
        randomIndex = Math.floor(Math.random() * currentIndex);
        currentIndex--;
        // And swap it with the current element.
        [array[currentIndex], array[randomIndex]] = [
            array[randomIndex], array[currentIndex]];
    }
    return array;
}

function initializeCards() {
    console.log("🃏 Initializing card deck on server...");
    let deck = [];
    let entityIdCounter = 1; // エンティティIDは1から始める

    // 1. デッキ作成
    for (const suit of SUITS) {
        for (const rank of RANKS) {
            deck.push({ suit, rank });
        }
    }

    // 2. デッキシャッフル
    deck = shuffle(deck);
    console.log(`  Deck shuffled (${deck.length} cards).`);

    // 3. カードデータ配列を初期化
    gameState.cards = [];

    // 4. 場札 (Tableau) に配る
    let cardIndex = 0;
    for (let i = 0; i < 7; i++) { // 7つの場札の山
        for (let j = 0; j <= i; j++) { // 各山に i+1 枚配る
            if (cardIndex >= deck.length) break;
            const cardInfo = deck[cardIndex++];
            gameState.cards.push({
                entity: entityIdCounter++, // Rust側のEntity型に合わせる (usizeだけどJSではnumber)
                suit: cardInfo.suit,
                rank: cardInfo.rank,
                is_face_up: (j === i), // 各山の最後のカードだけ表向き
                stack_type: 'Tableau',
                stack_index: i,
                position_in_stack: j,
                position: { x: 0, y: 0 } // 位置はクライアントで計算するけど初期値設定
            });
        }
    }
    console.log(`  Dealt cards to Tableau piles. ${cardIndex} cards used.`);

    // 5. 残りを山札 (Stock) に配る
    let stockPosition = 0;
    while (cardIndex < deck.length) {
        const cardInfo = deck[cardIndex++];
        gameState.cards.push({
            entity: entityIdCounter++,
            suit: cardInfo.suit,
            rank: cardInfo.rank,
            is_face_up: false, // 山札は裏向き
            stack_type: 'Stock',
            stack_index: null, // Stock には index はない
            position_in_stack: stockPosition++,
            position: { x: 0, y: 0 }
        });
    }
    console.log(`  Dealt remaining ${stockPosition} cards to Stock pile.`);
    console.log(`  Total cards in gameState: ${gameState.cards.length}`);
    // console.log("Initial GameState Cards:", JSON.stringify(gameState.cards, null, 2)); // デバッグ用に詳細表示が必要なら
}

initializeCards(); // サーバー起動時にカード状態を初期化
// --- ★実装ここまで★

const clients = new Map();
let nextPlayerId = 1;

// --- ヘルパー関数: 全クライアントにメッセージを送信 (ブロードキャスト) --- ★追加★
function broadcast(message, senderWs = null) {
    const messageJson = JSON.stringify(message);
    console.log(`📤 Broadcasting: ${messageJson}`);
    clients.forEach((ws, playerId) => {
        // 送信元クライアントには送らない場合 (オプション)
        if (senderWs && ws === senderWs) {
            return;
        }
        // 接続が確立しているクライアントにのみ送信
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(messageJson);
        } else {
            // 接続が切れていたら Map から削除？ (close イベントで処理する方が良いかも)
            console.warn(`Client ${playerId} is not open, removing from broadcast list.`);
            clients.delete(playerId);
        }
    });
}

// 新しいクライアント接続があった時の処理
wss.on('connection', (ws) => {
    // ★注意★: この時点ではまだ Player ID は不明！ JoinGame を待つ。
    console.log('✅ Client connected (pending JoinGame)');

    // クライアントからメッセージを受信した時の処理
    ws.on('message', (message) => {
        try {
            const messageString = message.toString();
            console.log('📥 Received:', messageString);
            const clientMessage = JSON.parse(messageString);

            // --- JoinGame 処理 --- 
            if (clientMessage.JoinGame) {
                const playerName = clientMessage.JoinGame.player_name || '名無しさん';
                console.log(`👋 Player '${playerName}' trying to join...`);

                // 新しい Player ID を発行
                const newPlayerId = nextPlayerId++;
                const newPlayer = { id: newPlayerId, name: playerName };

                // クライアントを Map に追加 (ID と WebSocket オブジェクトを紐付け)
                clients.set(newPlayerId, ws);
                // ws オブジェクト自体にも playerId を持たせると便利かも？
                ws.playerId = newPlayerId;

                // ゲーム状態にプレイヤーを追加 ★変更★
                gameState.players.push(newPlayer);
                console.log(`  Added player ${newPlayerId} ('${playerName}') to gameState. Total players: ${gameState.players.length}`);

                // --- GameJoined メッセージを作成 (現在の gameState を使う！) ★変更★ ---
                const gameJoinedMessage = {
                    GameJoined: {
                        your_player_id: newPlayerId,
                        // 現在のゲーム状態全体を送る！
                        initial_game_state: gameState
                    }
                };
                const responseJson = JSON.stringify(gameJoinedMessage);
                console.log('📤 Sending GameJoined:', responseJson);
                ws.send(responseJson);
                console.log(`👍 Sent GameJoined to Player ${newPlayerId} ('${playerName}')`);

                // --- 他のクライアントに PlayerJoined をブロードキャスト --- ★追加★
                const playerJoinedMessage = {
                    PlayerJoined: {
                        player_id: newPlayerId,
                        player_name: playerName
                    }
                };
                // 送信元 (ws) 以外にブロードキャスト
                broadcast(playerJoinedMessage, ws);

                // --- MakeMove 処理 --- (まだ実装しない)
            } else if (clientMessage.MakeMove) {
                console.log('🔄 Received MakeMove (Ignoring for now).');
                // TODO: ゲーム状態を更新して、GameStateUpdate をブロードキャストする

                // --- Ping 処理 --- 
            } else if (clientMessage.Ping) {
                console.log('🏓 Received Ping');
                const pongMessage = { Pong: {} };
                ws.send(JSON.stringify(pongMessage));
                console.log('📤 Sent Pong');

                // --- その他のメッセージ --- 
            } else {
                console.warn('❓ Received unknown message type:', clientMessage);
            }

        } catch (error) {
            console.error('❌ Failed to process message or error occurred:', error);
            try {
                ws.send(JSON.stringify({ Error: { message: 'Failed to process message on server' } }));
            } catch (sendError) {
                console.error('Failed to send error message to client:', sendError);
            }
        }
    });

    // クライアント切断時の処理
    ws.on('close', () => {
        console.log('❌ Client disconnected');
        // どのプレイヤーが切断したか特定して処理 ★変更★
        if (ws.playerId) {
            console.log(`  Player ${ws.playerId} disconnected.`);
            // ゲーム状態からプレイヤーを削除
            gameState.players = gameState.players.filter(p => p.id !== ws.playerId);
            // クライアント管理 Map から削除
            clients.delete(ws.playerId);
            console.log(`  Removed player ${ws.playerId} from gameState and clients map. Total players: ${gameState.players.length}`);

            // --- 他のクライアントに PlayerLeft をブロードキャスト --- ★追加★
            const playerLeftMessage = { PlayerLeft: { player_id: ws.playerId } };
            broadcast(playerLeftMessage); // 全員に送信 (切断した本人には送られない)
        } else {
            console.warn('  Disconnected client did not have a playerId (never joined?).');
        }
    });

    // エラー発生時の処理
    ws.on('error', (error) => {
        console.error('WebSocket error:', error);
        // エラー発生時も close イベントが呼ばれることが多いので、そちらで処理。
        // 必要であればここで Map から削除などのクリーンアップを行う。
        if (ws.playerId && clients.has(ws.playerId)) {
            clients.delete(ws.playerId);
            console.log(`  Removed player ${ws.playerId} from clients map due to error.`);
            // gameState からも削除すべき？ close で処理されるはずだけど念のため？
            gameState.players = gameState.players.filter(p => p.id !== ws.playerId);
            // PlayerLeft も broadcast すべき？
        }
    });
});

// TODO: ゲーム状態を永続化する方法 (ファイル保存、DBなど)
// TODO: MakeMove をちゃんと処理して gameState.cards を更新する
// TODO: GameStateUpdate メッセージを実装してブロードキャストする 