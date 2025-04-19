// server/ws_server.js

// ws パッケージをインポート
const WebSocket = require('ws');

// サーバーをポート 8101 で起動
const wss = new WebSocket.Server({ port: 8101 });

console.log('🚀 WebSocket Server is running on ws://localhost:8101');

// --- ゲーム状態管理 (簡易版) --- ★ここから追加★
// サーバー側で保持するゲーム状態オブジェクト
// protocol.rs の GameStateData に対応する感じ！
let gameState = {
    players: [], // { id: PlayerId, name: String }
    cards: [],   // { entity: Entity(usize), suit: Suit, rank: Rank, is_face_up: bool, stack_type: StackType, position_in_stack: usize, position: {x:f32, y:f32} }
    // TODO: 他に必要な状態 (例: Waste の枚数、ゲームのステータスなど)
};

// カードの初期配置を行う関数 (今は使わないけど、サーバー主導にするならここに書く)
function initializeCards() {
    // Rust側の deal_system.rs に相当するロジックをここに実装？
    // gameState.cards = ...;
    console.log("🃏 (Server-side card dealing not implemented yet)");
    // とりあえず空のまま
    gameState.cards = [];
}

initializeCards(); // サーバー起動時にカード状態を初期化 (今は空だけど)
// --- ★追加ここまで★

const clients = new Map(); // 変更: クライアントを PlayerId と WebSocket オブジェクトのペアで管理！
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