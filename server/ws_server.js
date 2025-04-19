// server/ws_server.js

// ws パッケージをインポート
const WebSocket = require('ws');

// サーバーをポート 8101 で起動
const wss = new WebSocket.Server({ port: 8101 });

console.log('🚀 WebSocket Server is running on ws://localhost:8101');

// 接続してきたクライアントを管理する Set (同じクライアントが複数入らないように)
const clients = new Set();
// 次に割り当てる Player ID (仮)
let nextPlayerId = 1;

// 新しいクライアント接続があった時の処理
wss.on('connection', (ws) => {
    console.log('✅ Client connected');
    clients.add(ws);

    // クライアントからメッセージを受信した時の処理
    ws.on('message', (message) => {
        try {
            // メッセージはバイナリかもしれないので String に変換
            const messageString = message.toString();
            console.log('📥 Received:', messageString);

            // 受信したメッセージを JSON としてパース
            const clientMessage = JSON.parse(messageString);

            // メッセージタイプに応じて処理を分岐 (今は JoinGame だけ)
            if (clientMessage.JoinGame) {
                const playerName = clientMessage.JoinGame.player_name || '名無しさん';
                console.log(`👋 Player '${playerName}' trying to join...`);

                // --- GameJoined メッセージを作成 --- 
                const playerId = nextPlayerId++;
                const serverMessage = {
                    GameJoined: {
                        your_player_id: playerId,
                        // とりあえず初期状態は空っぽで返す！
                        initial_game_state: {
                            players: [
                                { id: playerId, name: playerName } // 自分自身の情報だけ返す
                            ],
                            cards: [] // カード情報は空
                        }
                    }
                };

                // --- メッセージを送信 --- 
                const responseJson = JSON.stringify(serverMessage);
                console.log('📤 Sending GameJoined:', responseJson);
                ws.send(responseJson);
                console.log(`👍 Sent GameJoined to Player ${playerId} ('${playerName}')`);

            } else if (clientMessage.MakeMove) {
                // MakeMove はとりあえず無視するか、MoveRejected を返す
                console.log('🔄 Received MakeMove (Ignoring for now).');
                // const rejectMessage = { MoveRejected: { reason: "Not implemented yet" } };
                // ws.send(JSON.stringify(rejectMessage));

            } else if (clientMessage.Ping) {
                // Ping が来たら Pong を返す (接続維持のため)
                console.log('🏓 Received Ping');
                const pongMessage = { Pong: {} }; // Pongメッセージは `{}` だけでOKのはず
                ws.send(JSON.stringify(pongMessage));
                console.log('📤 Sent Pong');
            } else {
                console.warn('❓ Received unknown message type:', clientMessage);
            }

        } catch (error) {
            console.error('❌ Failed to process message or error occurred:', error);
            // エラーメッセージをクライアントに返す？
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
        clients.delete(ws);
        // TODO: PlayerLeft メッセージを他のクライアントにブロードキャストする？
    });

    // エラー発生時の処理
    ws.on('error', (error) => {
        console.error('WebSocket error:', error);
        clients.delete(ws); // エラーが発生した接続も Set から削除
    });
});

// TODO: 定期的に全クライアントに Ping を送る？ (タイムアウト検出のため)
// TODO: ゲーム状態をサーバー側で保持・更新するロジック
// TODO: メッセージを特定のクライアントや全クライアントにブロードキャストする機能 