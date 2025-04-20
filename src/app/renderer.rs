// src/app/renderer.rs
//! GameApp の描画関連ロジック。

use std::sync::{Arc, Mutex};
use crate::ecs::world::World;
use crate::components::{Position, Card, DraggingInfo, StackInfo, Suit, Rank};
use crate::ecs::entity::Entity;
use crate::{log}; // log マクロを使う
use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
// ★修正: config::layout の定数を直接使うためインポート★
use crate::config::layout::*;

// --- 定数定義 (Constants) ---
// カードの見た目に関する設定値をここで決めておくよ！ ✨

// カードのサイズ (ピクセル単位)
// ★注意: layout.rs にも CARD_WIDTH/HEIGHT があるけど、型が違う (f32 vs f64)。
//   描画では f64 が必要なので、ここで定義するか、layout.rs の方を f64 にするか、
//   キャストして使うか。ここでは renderer.rs に f64 で定義する。
// ★修正: event_handler から参照されるため pub にする★
pub const RENDER_CARD_WIDTH: f64 = 70.0;
pub const RENDER_CARD_HEIGHT: f64 = 100.0;
pub const RENDER_CARD_CORNER_RADIUS: f64 = 5.0; // カードの角の丸み

// カードの色
const COLOR_CARD_BG: &str = "#ffffff"; // カードの背景色 (白)
const COLOR_CARD_BORDER: &str = "#cccccc"; // カードの枠線の色 (薄いグレー)
const COLOR_CARD_BACK: &str = "#4682b4"; // カード裏面の色 (スティールブルー)
const COLOR_TEXT_RED: &str = "#d10a0a"; // 赤色の文字 (ハート、ダイヤ)
const COLOR_TEXT_BLACK: &str = "#111111"; // 黒色の文字 (スペード、クラブ)
// ★追加: プレースホルダーの色★
const COLOR_PLACEHOLDER_BORDER: &str = "#a0a0a0"; // 空のスタックの枠線色 (少し濃いグレー)

// カードの文字 (ランクとスート)
const FONT_FAMILY: &str = "sans-serif";
const FONT_SIZE_RANK: f64 = 18.0; // ランク (A, 2-10, J, Q, K) のフォントサイズ
const FONT_SIZE_SUIT: f64 = 14.0; // スート (♥♦♠♣) のフォントサイズ
const RANK_OFFSET_X: f64 = 5.0; // カード左上からのランク文字のXオフセット
const RANK_OFFSET_Y: f64 = 20.0; // カード左上からのランク文字のYオフセット
const SUIT_OFFSET_X: f64 = 5.0; // カード左上からのスート文字のXオフセット
const SUIT_OFFSET_Y: f64 = 38.0; // カード左上からのスート文字のYオフセット

// --- 公開関数 (GameApp から呼び出される) ---

/// Rust側で Canvas にゲーム画面を描画する関数。
/// GameApp::render_game_rust のロジックを移動。
pub fn render_game_rust(
    world_arc: &Arc<Mutex<World>>,
    canvas: &HtmlCanvasElement, // Canvas と Context への参照を受け取る
    context: &CanvasRenderingContext2d
) -> Result<(), JsValue> {
    // ★削除★ ログ不要
    // log("App::Renderer: render_game_rust() called!");

    // --- ステップ1: Canvas 寸法を取得 --- 
    let canvas_width = canvas.width() as f64;
    let canvas_height = canvas.height() as f64;

    // --- ステップ2: Canvas をクリア --- 
    // 毎回描画する前に、前のフレームの絵を全部消すよ！🧹
    context.clear_rect(0.0, 0.0, canvas_width, canvas_height);

    // ★★★ 新しいステップ: 2.5 スタックのプレースホルダーを描画 ★★★
    // ★削除★ ログ不要
    // log("  Drawing stack placeholders...");
    context.set_stroke_style(&JsValue::from_str(COLOR_PLACEHOLDER_BORDER));
    context.set_line_width(1.0);

    // 2.5.1: 山札 (Stock) のプレースホルダーを描画
    // ★削除★ ログ不要
    // log("    Attempting to draw Stock placeholder...");
    draw_rounded_rect(
        context,
        STOCK_POS_X as f64,
        STOCK_POS_Y as f64,
        RENDER_CARD_WIDTH,
        RENDER_CARD_HEIGHT,
        RENDER_CARD_CORNER_RADIUS,
    )?;
    context.stroke();
    // ★削除★ ログ不要
    // log(&format!("    Drew Stock placeholder at ({}, {})", STOCK_POS_X, STOCK_POS_Y));

    // 2.5.2: 捨て札 (Waste) のプレースホルダーを描画
    // ★削除★ ログ不要
    // log("    Attempting to draw Waste placeholder..."); 
    // ★★★ デバッグ: draw_rounded_rect + stroke の代わりに stroke_rect を試す！ ★★★
    // ★削除★ ログ不要
    // log("      Calling context.stroke_rect() for Waste...");
    context.stroke_rect(
        WASTE_POS_X as f64,
        WASTE_POS_Y as f64,
        RENDER_CARD_WIDTH,
        RENDER_CARD_HEIGHT
    );
    // ★削除★ ログ不要
    // log("      context.stroke_rect() for Waste called (assuming success).");
    /* --- 元のコード (コメントアウト) ---
    match draw_rounded_rect(
        context,
        WASTE_POS_X as f64,
        WASTE_POS_Y as f64,
        RENDER_CARD_WIDTH,
        RENDER_CARD_HEIGHT,
        RENDER_CARD_CORNER_RADIUS,
    ) {
        Ok(_) => log("      draw_rounded_rect for Waste succeeded."),
        Err(e) => {
            log(&format!("      draw_rounded_rect for Waste FAILED: {:?}", e));
            return Err(e); // エラーならここで終了
        }
    };
    log("      Calling context.stroke() for Waste...");
    context.stroke();
    log("      context.stroke() for Waste called (assuming success).");
    */
    // ★削除★ ログ不要
    // log(&format!("    Finished drawing Waste placeholder at ({}, {})", WASTE_POS_X, WASTE_POS_Y));

    // 2.5.3: 上がり札 (Foundation) のプレースホルダーを描画 (4つあるからループ！)
    // ★削除★ 念のためログ不要
    // log("    Attempting to draw Foundation placeholders...");
    for i in 0..4 { 
        let foundation_x = FOUNDATION_START_X as f64 + i as f64 * FOUNDATION_X_OFFSET as f64;
        let foundation_y = FOUNDATION_START_Y as f64;
        draw_rounded_rect(
            context,
            foundation_x,
            foundation_y,
            RENDER_CARD_WIDTH,
            RENDER_CARD_HEIGHT,
            RENDER_CARD_CORNER_RADIUS,
        )?;
        context.stroke();
        // log(&format!("    Drew Foundation {} placeholder at ({}, {})", i, foundation_x, foundation_y)); // 毎回のログはうるさいのでコメントアウト
    }
    // ★削除★ 念のためログ不要
    // log("    Finished drawing all 4 Foundation placeholders.");

    // ★削除★ ログ不要
    // log("  Finished drawing placeholders.");

    // --- ステップ3: World からカード情報を取得 & ソート --- 
    let world = world_arc.lock().map_err(|e| JsValue::from_str(&format!("Failed to lock world mutex: {}", e)))?;

    let card_entities = world.get_all_entities_with_component::<Card>();
    let mut cards_to_render: Vec<(Entity, &Position, &Card, Option<&DraggingInfo>, Option<&StackInfo>)> = Vec::with_capacity(card_entities.len());

    for &entity in &card_entities {
        if let (Some(pos), Some(card)) = (
            world.get_component::<Position>(entity),
            world.get_component::<Card>(entity)
        ) {
            let dragging_info = world.get_component::<DraggingInfo>(entity);
            let stack_info = world.get_component::<StackInfo>(entity);
            cards_to_render.push((entity, pos, card, dragging_info, stack_info));
        } else {
            log(&format!("Warning: Skipping entity {:?} in render_game_rust because Card or Position component is missing.", entity));
        }
    }

    cards_to_render.sort_by(|a, b| {
        let (_, _, _, dragging_info_a_opt, stack_info_a_opt) = a;
        let (_, _, _, dragging_info_b_opt, stack_info_b_opt) = b;

        let order_a = dragging_info_a_opt
            .map(|di| di.original_position_in_stack)
            .or_else(|| stack_info_a_opt.map(|si| si.position_in_stack as usize))
            .unwrap_or(0);

        let order_b = dragging_info_b_opt
            .map(|di| di.original_position_in_stack)
            .or_else(|| stack_info_b_opt.map(|si| si.position_in_stack as usize))
            .unwrap_or(0);

        order_a.cmp(&order_b)
    });

    // --- ステップ4: カードの描画処理 --- 
    // ★削除★ ログ不要
    // log(&format!("App::Renderer: Rendering {} sorted cards...", cards_to_render.len()));

    // ソートされたカードリストをループで回して、1枚ずつ描画していくよ！
    for (entity, pos, card, _dragging_info_opt, _stack_info_opt) in cards_to_render {
        // ★削除★ ログが多すぎるのでコメントアウト！
        // log(&format!("  Rendering card {:?} at ({}, {})", entity, pos.x, pos.y));

        // --- 4.1: カードの基本の四角形を描画 --- 
        // 角丸の四角を描画するヘルパー関数を呼ぶ
        draw_rounded_rect(
            context,
            pos.x as f64, // Position の f32 を f64 にキャスト
            pos.y as f64,
            RENDER_CARD_WIDTH, // 描画用の定数を使う
            RENDER_CARD_HEIGHT,
            RENDER_CARD_CORNER_RADIUS,
        )?;
        context.set_fill_style(&JsValue::from_str(COLOR_CARD_BG)); // 背景色 (白)
        context.fill();
        context.set_stroke_style(&JsValue::from_str(COLOR_CARD_BORDER)); // 枠線の色
        context.stroke();

        // --- 4.2: カードの内容を描画 (表向きか裏向きかで処理を分ける) ---
        if card.is_face_up {
            // --- 表向きのカード --- 
            // ★削除★ ログが多すぎるのでコメントアウト！
            // log(&format!("    Card {:?} is face up ({:?}, {:?})", entity, card.rank, card.suit));
            // 4.2.1: スートに基づいて文字色を決定
            let text_color = match card.suit {
                Suit::Heart | Suit::Diamond => COLOR_TEXT_RED,
                Suit::Spade | Suit::Club => COLOR_TEXT_BLACK,
            };
            context.set_fill_style(&JsValue::from_str(text_color));

            // 4.2.2: ランク (数字/文字) を描画
            let rank_text = get_rank_text(card.rank); // ランクを文字列に変換 (ヘルパー関数)
            context.set_font(&format!("bold {}px {}", FONT_SIZE_RANK, FONT_FAMILY));
            context.fill_text(
                rank_text,
                pos.x as f64 + RANK_OFFSET_X,
                pos.y as f64 + RANK_OFFSET_Y,
            )?;

            // 4.2.3: スート (マーク) を描画
            let suit_text = get_suit_text(card.suit); // スートを絵文字に変換 (ヘルパー関数)
            context.set_font(&format!("{}px {}", FONT_SIZE_SUIT, FONT_FAMILY));
            context.fill_text(
                suit_text,
                pos.x as f64 + SUIT_OFFSET_X,
                pos.y as f64 + SUIT_OFFSET_Y,
            )?;

        } else {
            // --- 裏向きのカード --- 
            // log(&format!("    Card {:?} is face down", entity)); // 必要ならこれもコメントアウト
            context.set_fill_style(&JsValue::from_str(COLOR_CARD_BACK));
            context.fill();
        }
    }

    // ★削除★ ログ不要
    // log("App::Renderer: Card rendering finished.");
    Ok(())
}

// --- ヘルパー関数 (Helper Functions) ---

/// 角丸の四角形のパスを作成するヘルパー関数。
/// これ自体は描画せず、パスを作るだけだよ。
/// 呼び出し側で `context.fill()` や `context.stroke()` をする必要がある。
fn draw_rounded_rect(
    context: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) -> Result<(), JsValue> {
    context.begin_path();
    context.move_to(x + radius, y);
    context.line_to(x + width - radius, y);
    context.arc_to(x + width, y, x + width, y + radius, radius)?;
    context.line_to(x + width, y + height - radius);
    context.arc_to(x + width, y + height, x + width - radius, y + height, radius)?;
    context.line_to(x + radius, y + height);
    context.arc_to(x, y + height, x, y + height - radius, radius)?;
    context.line_to(x, y + radius);
    context.arc_to(x, y, x + radius, y, radius)?;
    context.close_path();
    Ok(())
}

/// ランク (Rank enum) を表示用の文字列に変換するヘルパー関数。
fn get_rank_text(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
    }
}

/// スート (Suit enum) を表示用の絵文字 (文字列) に変換するヘルパー関数。
fn get_suit_text(suit: Suit) -> &'static str {
    match suit {
        Suit::Heart => "♥",   // ハート
        Suit::Diamond => "♦", // ダイヤ
        Suit::Spade => "♠",   // スペード
        Suit::Club => "♣",    // クラブ
    }
} 