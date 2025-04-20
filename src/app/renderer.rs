// src/app/renderer.rs
//! GameApp の描画関連ロジック。

use std::sync::{Arc, Mutex};
use crate::ecs::world::World;
use crate::components::{Position, Card, DraggingInfo, StackInfo, Suit, Rank, StackType};
use crate::ecs::entity::Entity;
use log::warn;
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
    context.begin_path();
    context.rect(STOCK_POS_X as f64, STOCK_POS_Y as f64, RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT);
    context.set_stroke_style(&JsValue::from_str(COLOR_PLACEHOLDER_BORDER));
    context.set_line_width(1.0);
    context.stroke();
    context.set_line_dash(&JsValue::from(js_sys::Array::new())).unwrap();
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

    // --- 2. Collect render data for ALL cards --- 
    let card_entities = world.get_all_entities_with_component::<Card>();
    // ★ Vec に Position, Card, is_dragging, StackInfo(Option) を格納 ★
    let mut card_render_list: Vec<(Entity, Position, Card, bool, Option<StackInfo>)> = Vec::with_capacity(card_entities.len());

    for entity in &card_entities {
        if let (Some(pos), Some(card)) = (
            world.get_component::<Position>(*entity),
            world.get_component::<Card>(*entity)
        ) {
            let is_dragging = world.get_component::<DraggingInfo>(*entity).is_some();
            let stack_info_opt = world.get_component::<StackInfo>(*entity).cloned(); // Clone StackInfo
            // ★ リストにクローンしたデータを追加 ★
            card_render_list.push((*entity, pos.clone(), card.clone(), is_dragging, stack_info_opt));
        } else {
            warn!("Renderer: Entity {:?} missing Pos/Card, skipping.", entity);
        }
    }

    // --- 3. Sort cards for correct rendering order --- 
    // ★ ソート処理を復活＆改良 ★
    // 基本は StackType -> position_in_stack でソートする
    // (Tableau など、重なり順が重要なスタックを意識)
    card_render_list.sort_by(|a, b| {
        let (_, _, _, _, stack_info_a_opt) = a;
        let (_, _, _, _, stack_info_b_opt) = b;

        // StackInfo がないものは最後に描画 (エラーケースなど)
        match (stack_info_a_opt, stack_info_b_opt) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater, // None は大きい (後)
            (Some(_), None) => std::cmp::Ordering::Less,    // Some は小さい (先)
            (Some(info_a), Some(info_b)) => {
                // まず StackType で比較 (描画順: Stock -> Waste -> Foundation -> Tableau -> Hand)
                let type_order_a = stack_type_draw_order(info_a.stack_type);
                let type_order_b = stack_type_draw_order(info_b.stack_type);
                match type_order_a.cmp(&type_order_b) {
                    std::cmp::Ordering::Equal => {
                        // 同じ StackType なら position_in_stack で比較 (小さい方が先)
                        info_a.position_in_stack.cmp(&info_b.position_in_stack)
                    }
                    other => other,
                }
            }
        }
    });

    // --- 4. Draw cards in sorted order (handling dragged card) --- 
    // log(&format!("Renderer: Drawing {} sorted card entities...", card_render_list.len())); // ★ コメントアウト ★
    // ★ ドラッグ中カード描画用変数を初期化 ★
    let mut dragged_card_data: Option<(Position, Card)> = None;

    // ★ ソート済みリストをループ ★
    for (_entity, pos, card, is_dragging, _stack_info_opt) in card_render_list {
        // ★ ドラッグ中のカードは保存してスキップ ★
        if is_dragging {
            // log(&format!("  - Storing dragged card {:?} for later rendering.", _entity));
            dragged_card_data = Some((pos, card)); // pos, card は move される
            continue;
        }

        // --- 通常のカード描画 (ドラッグ中でない場合) --- 
        let card_x = pos.x as f64;
        let card_y = pos.y as f64;

        if card.is_face_up {
            // ... (face-up card drawing logic - unchanged, uses pos and card) ...
            context.save();
            draw_rounded_rect(context, card_x, card_y, RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT, RENDER_CARD_CORNER_RADIUS)?;
            context.set_fill_style(&JsValue::from_str(COLOR_CARD_BG));
            context.fill();
            context.set_stroke_style(&JsValue::from_str(COLOR_CARD_BORDER));
            context.stroke();
            context.restore();

            let (text_color, suit_char) = match card.suit {
                Suit::Heart | Suit::Diamond => (COLOR_TEXT_RED, get_suit_text(card.suit)),
                Suit::Club | Suit::Spade => (COLOR_TEXT_BLACK, get_suit_text(card.suit)),
            };
            let rank_char = get_rank_text(card.rank);

            context.save();
            context.set_fill_style(&JsValue::from_str(text_color));
            context.set_font(&format!("bold {}px {}", FONT_SIZE_RANK, FONT_FAMILY));
            context.fill_text(&format!("{} {}", rank_char, suit_char), card_x + RANK_OFFSET_X, card_y + RANK_OFFSET_Y)?;
            context.restore();
        } else {
            // ... (face-down card drawing logic - unchanged, uses pos and card) ...
            context.save();
            draw_rounded_rect(context, card_x, card_y, RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT, RENDER_CARD_CORNER_RADIUS)?;
            context.set_fill_style(&JsValue::from_str(COLOR_CARD_BACK));
            context.fill();
            context.set_stroke_style(&JsValue::from_str(COLOR_CARD_BORDER));
            context.stroke();
            context.restore();
        }
        // --- 通常のカード描画ここまで ---
    }

    // --- 5. Draw the dragged card last (if any) --- 
    if let Some((pos, card)) = dragged_card_data {
        // log(&format!("Renderer: Drawing dragged card at ({}, {})", pos.x, pos.y));
        let card_x = pos.x as f64;
        let card_y = pos.y as f64;
        // ... (dragged card drawing logic - unchanged) ...
        if card.is_face_up {
            context.save();
            draw_rounded_rect(context, card_x, card_y, RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT, RENDER_CARD_CORNER_RADIUS)?;
            context.set_fill_style(&JsValue::from_str(COLOR_CARD_BG));
            context.fill();
            context.set_stroke_style(&JsValue::from_str(COLOR_CARD_BORDER));
            context.stroke();
            let (text_color, suit_char) = match card.suit {
                Suit::Heart | Suit::Diamond => (COLOR_TEXT_RED, get_suit_text(card.suit)),
                Suit::Club | Suit::Spade => (COLOR_TEXT_BLACK, get_suit_text(card.suit)),
            };
            let rank_char = get_rank_text(card.rank);
            context.save();
            context.set_fill_style(&JsValue::from_str(text_color));
            context.set_font(&format!("bold {}px {}", FONT_SIZE_RANK, FONT_FAMILY));
            context.fill_text(&format!("{} {}", rank_char, suit_char), card_x + RANK_OFFSET_X, card_y + RANK_OFFSET_Y)?; 
            context.restore();
        } else {
            context.save();
            draw_rounded_rect(context, card_x, card_y, RENDER_CARD_WIDTH, RENDER_CARD_HEIGHT, RENDER_CARD_CORNER_RADIUS)?;
            context.set_fill_style(&JsValue::from_str(COLOR_CARD_BACK));
            context.fill();
            context.set_stroke_style(&JsValue::from_str(COLOR_CARD_BORDER));
            context.stroke();
            context.restore();
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

// ★ 追加: StackType の描画順序を決めるヘルパー関数 ★
fn stack_type_draw_order(stack_type: StackType) -> u8 {
    match stack_type {
        StackType::Stock => 0,
        StackType::Waste => 1,
        StackType::Foundation(_) => 2, // Foundation は Tableau より先に描画
        StackType::Tableau(_) => 3,    // Tableau は Foundation の後
        StackType::Hand => 4,         // Hand は最後 (もし使うなら)
    }
} 