// src/config/layout.rs
//! ゲーム画面のレイアウトに関する定数を定義するよ！
//! カードやスタックの座標、オフセットなど。

pub const CARD_WIDTH: f32 = 100.0; // カード画像の幅
pub const CARD_HEIGHT: f32 = 145.0; // カード画像の高さ
pub const CARD_SPACING_X: f32 = 20.0; // カード間の水平方向の間隔
pub const CARD_SPACING_Y: f32 = 15.0; // カード間の垂直方向の間隔 (重なり具合)
pub const STACK_PADDING: f32 = 10.0; // スタック周囲の余白

// --- 各エリアの開始位置 ---
pub const STOCK_POS_X: f32 = -450.0; // 山札のX座標
pub const STOCK_POS_Y: f32 = 250.0; // 山札のY座標

pub const WASTE_POS_X: f32 = -300.0; // 捨て札置き場のX座標
pub const WASTE_POS_Y: f32 = 250.0; // 捨て札置き場のY座標

pub const FOUNDATION_START_X: f32 = 50.0; // 上がり札置き場 (Foundation) の開始X座標
pub const FOUNDATION_START_Y: f32 = 250.0; // 上がり札置き場のY座標
pub const FOUNDATION_X_OFFSET: f32 = 150.0; // 上がり札置き場間のX方向の間隔

pub const TABLEAU_START_X: f32 = -450.0; // 場札 (Tableau) の開始X座標
pub const TABLEAU_START_Y: f32 = 0.0; // 場札の開始Y座標
pub const TABLEAU_X_OFFSET: f32 = 150.0; // 場札の列間のX方向の間隔
pub const TABLEAU_Y_OFFSET_FACE_DOWN: f32 = -15.0; // 場札の裏向きカードのY方向オフセット
pub const TABLEAU_Y_OFFSET_FACE_UP: f32 = -30.0; // 場札の表向きカードのY方向オフセット
pub const CARD_Z_INDEX_STEP: f32 = 0.1; // カードの重なり順 (Z座標) の増分

// TODO: Foundation と Waste の位置も必要になったら定義しよう！
// pub const FOUNDATION_START_X: f32 = 480.0; // 仮 (Tableau 3列目と同じくらい？)
// pub const FOUNDATION_START_Y: f32 = 100.0;
// pub const FOUNDATION_X_OFFSET: f32 = 110.0;

// pub const WASTE_POS_X: f32 = STOCK_POS_X + TABLEAU_X_OFFSET; // 山札の隣
// pub const WASTE_POS_Y: f32 = STOCK_POS_Y; 