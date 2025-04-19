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
                    const { moved_entity, target_stack } = parsedMessage.payload || {};
                    // ペイロードに必要な情報があるかチェック
                    if (moved_entity === undefined || target_stack === undefined || moved_entity.id === undefined) { // moved_entity.id もチェック
                        console.error('  Invalid MakeMove payload received:', parsedMessage.payload);
                        // (任意) エラーメッセージをクライアントに送り返す
                        // ws.send(JSON.stringify({ type: 'MoveRejected', payload: { reason: 'Invalid payload' } }));
                        break;
                    }

                    // --- 1. 動かすカードを探す ---                    
                    const movedCardIndex = gameState.cards.findIndex(card => card.entity && card.entity.id === moved_entity.id); // Rust側のEntityは { id: usize } だったはず

                    if (movedCardIndex === -1) {
                        console.error(`  MakeMove Error: Moved card with entity ID ${moved_entity.id} not found!`);
                        // ws.send(JSON.stringify({ type: 'MoveRejected', payload: { reason: 'Card not found' } }));
                        break;
                    }

                    const movedCard = gameState.cards[movedCardIndex];
                    // 元の情報をディープコピーしておく（移動元判定のため）
                    const oldStackType = movedCard.stack_type;
                    const oldStackIndex = movedCard.stack_index;
                    const oldPositionInStack = movedCard.position_in_stack;
                    console.log(`  Processing move for Card ID ${movedCard.entity.id} (${movedCard.rank} of ${movedCard.suit}) from ${oldStackType}${oldStackIndex !== null ? '[' + oldStackIndex + ']' : ''} pos ${oldPositionInStack}`);
                    console.log(`  Target Stack: ${target_stack.stack_type}${target_stack.stack_index !== null ? '[' + target_stack.stack_index + ']' : ''}`);

                    // --- 2. 新しい StackInfo を計算 ---                    
                    const newStackType = target_stack.stack_type; // stack_type を取得
                    const newStackIndex = target_stack.stack_index; // stack_index を取得 (Tableau/Foundation の場合に値が入る)

                    // 新しい position_in_stack を計算
                    let maxPosInTarget = -1;
                    gameState.cards.forEach(card => {
                        // 自分自身は除外して計算
                        if (card.entity.id !== movedCard.entity.id &&
                            card.stack_type === newStackType &&
                            card.stack_index === newStackIndex) // stack_index も比較 (null 同士もOK)
                        {
                            if (card.position_in_stack > maxPosInTarget) {
                                maxPosInTarget = card.position_in_stack;
                            }
                        }
                    });
                    const newPositionInStack = maxPosInTarget + 1;

                    // --- 3. gameState.cards を更新 ---                    
                    movedCard.stack_type = newStackType;
                    movedCard.stack_index = newStackIndex; // null か 数値
                    movedCard.position_in_stack = newPositionInStack;
                    // 表向きにするかどうか？ (例: Foundation に置いたら必ず表)
                    // movedCard.is_face_up = true; // 必要に応じて追加

                    console.log(`  Updated Card ID ${movedCard.entity.id} stack to ${newStackType}${newStackIndex !== null ? '[' + newStackIndex + ']' : ''} pos ${newPositionInStack}`);

                    // --- 4. 移動元の山に残ったカードを表にする処理 ---                    
                    if (oldStackType === 'Tableau' && oldPositionInStack > 0) {
                        const positionToReveal = oldPositionInStack - 1;
                        // 同じ Tableau の山 (oldStackIndex) の、一つ下 (positionToReveal) のカードを探す
                        const cardToRevealIndex = gameState.cards.findIndex(card =>
                            card.stack_type === oldStackType &&
                            card.stack_index === oldStackIndex &&
                            card.position_in_stack === positionToReveal
                        );

                        if (cardToRevealIndex !== -1) {
                            const cardToReveal = gameState.cards[cardToRevealIndex];
                            if (!cardToReveal.is_face_up) {
                                cardToReveal.is_face_up = true;
                                console.log(`  Revealed card ID ${cardToReveal.entity.id} (${cardToReveal.rank} of ${cardToReveal.suit}) at old position.`);
                            }
                        } else {
                            console.log(`  No card found to reveal at ${oldStackType}[${oldStackIndex}] pos ${positionToReveal}.`);
                        }
                    }

                    // --- 5. 全員に更新されたゲーム状態をブロードキャスト ---                    
                    broadcastGameStateUpdate();
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
                players: Object.values(gameState.players), // Object.values で配列に変換
                cards: gameState.cards,
            }
        }
    };
    broadcast(JSON.stringify(updateMessage)); // ブロードキャストヘルパーを使う
}

console.log("WebSocket server setup complete. Waiting for connections...👂"); 