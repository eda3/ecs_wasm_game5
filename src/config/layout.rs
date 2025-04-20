// src/config/layout.rs
//! ゲーム画面のレイアウトに関する定数を定義するよ！
//! カードやスタックの座標、オフセットなど。

// ★追加: 描画でも使う可能性があるので renderer と合わせる★
// pub const CARD_WIDTH: f32 = 70.0; // カードの幅
// pub const CARD_HEIGHT: f32 = 100.0; // カードの高さ
// ↑ renderer.rs 側で RENDER_CARD_WIDTH/HEIGHT を定義したのでコメントアウト

pub const CARD_SPACING_X: f32 = 20.0; // カード間の水平方向の間隔 (未使用かも)
pub const CARD_SPACING_Y: f32 = 15.0; // カード間の垂直方向の間隔 (未使用かも)
pub const STACK_PADDING: f32 = 10.0; // スタック周囲の余白 (未使用かも)

// --- 各エリアの開始位置 (画面左上から配置するように修正) ---

// ★修正: Stock と Waste の座標を画面左上に調整★
pub const STOCK_POS_X: f32 = 50.0;  // 左端からのオフセット
pub const STOCK_POS_Y: f32 = 50.0; // 上端からのオフセット

// ★修正: Foundation と共通の X方向の間隔を定義 ★
pub const STACK_X_OFFSET: f32 = 100.0; // 各スタック間の X 方向の間隔 (Stock, Waste, Foundation で共通化)

pub const WASTE_POS_X: f32 = STOCK_POS_X + STACK_X_OFFSET; // Stock の右隣
pub const WASTE_POS_Y: f32 = STOCK_POS_Y; // Stock と同じ高さ

// pub const FOUNDATION_START_X: f32 = WASTE_POS_X + STACK_X_OFFSET; // Waste の右隣から Foundation 開始 (★古い定義★)
pub const FOUNDATION_START_X: f32 = 500.0; // ★ 修正: もっと右に配置 ★
pub const FOUNDATION_START_Y: f32 = STOCK_POS_Y; // Stock/Waste と同じ高さに揃える (★変更なし★)
// pub const FOUNDATION_X_OFFSET: f32 = STACK_X_OFFSET; // 上で定義した共通の間隔を使う (★古い定義★)
pub const FOUNDATION_X_OFFSET: f32 = 90.0; // ★ 修正: 少し狭める ★

// ★修正: Tableau の開始位置も調整 (Stock/Waste/Foundation とのバランス)★
pub const TABLEAU_START_X: f32 = STOCK_POS_X; // Stock と同じ X 座標から開始 (7列配置する)
pub const TABLEAU_START_Y: f32 = STOCK_POS_Y + 100.0 + 50.0; // Stock/Waste/Foundation の下に配置 (カード高さ約100 + 余白50)
pub const TABLEAU_X_OFFSET: f32 = STACK_X_OFFSET; // Foundation と同じ間隔で列を配置

// 場札の重なりオフセット (ここは変更なし)
pub const TABLEAU_Y_OFFSET_FACE_DOWN: f32 = 10.0; // 場札の裏向きカードは少しだけ下にずらす
pub const TABLEAU_Y_OFFSET_FACE_UP: f32 = 25.0; // 場札の表向きカードは見えるようにもう少し下にずらす

// Z座標 (重なり順) - 未使用かも
pub const CARD_Z_INDEX_STEP: f32 = 0.1; // カードの重なり順 (Z座標) の増分

// --- 古い座標定義 (コメントアウト) ---
// pub const STOCK_POS_X: f32 = -450.0; 
// pub const STOCK_POS_Y: f32 = 250.0;
// pub const WASTE_POS_X: f32 = -300.0;
// pub const WASTE_POS_Y: f32 = 250.0;
// pub const FOUNDATION_START_X: f32 = 50.0; 
// pub const FOUNDATION_START_Y: f32 = 250.0;
// pub const FOUNDATION_X_OFFSET: f32 = 150.0;
// pub const TABLEAU_START_X: f32 = -450.0;
// pub const TABLEAU_START_Y: f32 = 0.0;
// pub const TABLEAU_X_OFFSET: f32 = 150.0;

// TODO: Foundation と Waste の位置も必要になったら定義しよう！
// pub const FOUNDATION_START_X: f32 = 480.0; // 仮 (Tableau 3列目と同じくらい？)
// pub const FOUNDATION_START_Y: f32 = 100.0;
// pub const FOUNDATION_X_OFFSET: f32 = 110.0;

// pub const WASTE_POS_X: f32 = STOCK_POS_X + TABLEAU_X_OFFSET; // 山札の隣
// pub const WASTE_POS_Y: f32 = STOCK_POS_Y; 