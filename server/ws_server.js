const WebSocket = require('ws');

const wss = new WebSocket.Server({ port: 8101 });
console.log('WebSocket server started on port 8101 🚀');

const clients = new Set();
let nextPlayerId = 1;

// ゲーム全体の現在の状態を保持するオブジェクト。
const gameState = {
    players: {}, // プレイヤー情報の連想配列 (キーはplayerId)
    // ★修正点1★: カード情報は最初は空配列。クライアントからの ProvideInitialState を待つ！
    cards: [],
};

// WebSocketサーバーに誰かが接続してきた時の処理を定義。
wss.on('connection', (ws) => {
    const playerId = nextPlayerId++; // 新しいプレイヤーにIDを割り当て
    const playerName = `Player ${playerId}`;
    gameState.players[playerId] = { id: playerId, name: playerName }; // プレイヤー情報を保存
    console.log(`Client connected: ${playerName} (ID: ${playerId})`);

    ws.playerId = playerId; // WebSocket接続オブジェクトにplayerIdを紐付け
    clients.add(ws); // クライアントリストに追加

    // 接続してきたクライアントに、ゲーム参加完了メッセージを送信。
    const joinMessage = {
        type: 'GameJoined', // メッセージタイプ
        payload: {
            playerId: playerId, // あなたのプレイヤーID
            initialGameState: { // ゲームの初期状態
                players: Object.values(gameState.players), // 現在参加中の全プレイヤーリスト
                // ★修正点3★: サーバーが保持している現在のカード状態 (gameState.cards) を送る！
                // (最初のプレイヤー接続時は空配列 [] が送られる想定)
                cards: gameState.cards,
            }
        }
    };
    // オブジェクトをJSON文字列に変換して送信
    ws.send(JSON.stringify(joinMessage));

    // 他の全クライアントに、新しいプレイヤーが参加したことを通知。
    const playerJoinedMessage = {
        type: 'PlayerJoined', // メッセージタイプ
        payload: { player: gameState.players[playerId] } // 参加したプレイヤーの情報
    };
    // 自分以外のクライアントにブロードキャスト（一斉送信）
    broadcast(JSON.stringify(playerJoinedMessage), ws);

    // クライアントが接続を切断した時の処理
    ws.on('close', () => {
        console.log(`Client disconnected: ${playerName} (ID: ${playerId})`);
        // gameStateからプレイヤー情報を削除
        delete gameState.players[playerId];
        // クライアントリストから削除
        clients.delete(ws);

        // 他の全クライアントに、プレイヤーが退出したことを通知。
        const playerLeftMessage = {
            type: 'PlayerLeft', // メッセージタイプ
            payload: { playerId: playerId } // 退出したプレイヤーのID
        };
        broadcast(JSON.stringify(playerLeftMessage)); // 全員にブロードキャスト
    });

    // クライアントからメッセージを受信した時の処理
    ws.on('message', (message) => {
        try {
            const messageString = message.toString();
            const parsedMessage = JSON.parse(messageString);
            console.log('Received message from player', ws.playerId, ':', parsedMessage.type);

            // メッセージのタイプに応じて処理を分岐
            switch (parsedMessage.type) {
                // ★修正点2★: ProvideInitialState メッセージを処理するケースを追加！
                case 'ProvideInitialState':
                    // クライアントから送られてきた初期状態を受け取る
                    // 注意: 最初に deal_initial_cards を実行したクライアントだけがこれを送る想定。
                    // もし複数のクライアントが送ってきた場合の処理は今は考慮していない。
                    if (parsedMessage.payload && parsedMessage.payload.initial_state && parsedMessage.payload.initial_state.cards) {
                        // gameState.cards を受け取ったデータで上書き！
                        // TODO: 既に gameState.cards が設定されている場合の処理を追加した方が安全かも？
                        //       (例: 最初の ProvideInitialState のみ受け付ける、など)
                        if (gameState.cards.length === 0) { // まだカードが設定されていなければ設定
                            gameState.cards = parsedMessage.payload.initial_state.cards;
                            console.log(`  Received and stored initial card state (${gameState.cards.length} cards) from player ${ws.playerId}.`);
                            // 最初の状態が設定されたら、全クライアントに通知する
                            broadcastGameStateUpdate();
                        } else {
                            console.log(`  Initial card state already exists. Ignoring ProvideInitialState from player ${ws.playerId}.`);
                        }
                    } else {
                        console.error('  Invalid ProvideInitialState payload received.');
                    }
                    break;

                case 'MakeMove':
                    // TODO: カード移動のリクエストを処理するロジック
                    console.log(`  Player ${ws.playerId} requested a move:`, parsedMessage.payload);
                    // 現状は受け取ったログを出すだけ
                    // 将来的には、ここで gameState.cards を更新し、
                    // broadcastGameStateUpdate(); を呼ぶことになる。
                    break;

                case 'RequestGameState':
                    // (任意) クライアントが明示的に最新の状態を要求してきた場合の処理
                    console.log(`  Player ${ws.playerId} requested game state.`);
                    const currentStateMessage = {
                        type: 'GameStateUpdate',
                        payload: {
                            current_game_state: {
                                players: Object.values(gameState.players),
                                cards: gameState.cards,
                            }
                        }
                    };
                    // 要求してきたクライアントにだけ送る
                    ws.send(JSON.stringify(currentStateMessage));
                    break;

                case 'Ping':
                    // Pong メッセージを送り返す
                    console.log(`  Received Ping from player ${ws.playerId}, sending Pong.`);
                    ws.send(JSON.stringify({ type: 'Pong' }));
                    break;

                // 他のメッセージタイプがあればここに追加
                default:
                    console.log('  Received unknown message type:', parsedMessage.type);
            }
        } catch (error) {
            console.error('Failed to process message or invalid message format:', error);
        }
    });
});

// メッセージを全員 (または指定した人以外) に送る便利関数
function broadcast(message, sender) {
    clients.forEach((client) => {
        if (client !== sender && client.readyState === WebSocket.OPEN) {
            client.send(message);
        }
    });
}

// (任意) 現在のゲーム状態を全クライアントにブロードキャストするヘルパー関数
function broadcastGameStateUpdate() {
    console.log("Broadcasting game state update to all clients...");
    const updateMessage = {
        type: 'GameStateUpdate',
        payload: {
            current_game_state: {
                players: Object.values(gameState.players),
                cards: gameState.cards,
            }
        }
    };
    broadcast(JSON.stringify(updateMessage));
}

console.log("WebSocket server setup complete. Waiting for connections...👂"); 