#!/bin/bash

# エラーが発生したらスクリプトを停止する
set -e

echo "🚀 デプロイ準備を開始します..."

# 1. Rust (WASM) コードをリリースモードでビルド
# --target web は、バンドラーなしで直接ブラウザで動かすのに適した形式で出力するよ
# これにより、プロジェクトルートに `pkg` ディレクトリが作成・更新される
echo "📦 WASM モジュールをビルド中... (リリースモード)"
wasm-pack build --release --target web

echo "✅ WASM モジュールのビルドが完了しました！ (./pkg)"
echo ""
echo "デプロイに必要なファイルは以下の場所にあります:"
echo "  - フロントエンド (HTML/CSS/JS): ./www/"
echo "  - WASM/JS グルーコード: ./pkg/"
echo "  - WebSocket サーバー: ./server/ws_server.js"
echo "  - サーバー依存関係: ./package.json, ./package-lock.json"
echo ""
echo "これらのファイルを本番サーバーの適切な場所に配置してください。"
echo "サーバーでのセットアップ例:"
echo "1. プロジェクト全体 (または必要なファイル) をサーバーにアップロード。"
echo "2. サーバーのプロジェクトルートに移動。"
echo "3. サーバーの依存関係をインストール: npm install --production"
echo "4. WebSocket サーバーを起動: node server/ws_server.js &"
echo "5. 静的ファイル配信サーバー (例: nginx) を設定し、./www と ./pkg の内容を配信するようにする。"
echo "   (例: nginx で / を ./www に、/pkg を ./pkg にマッピング)"
echo "�� デプロイが成功しますように！ 💖" 