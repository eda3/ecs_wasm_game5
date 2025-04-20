// src/components/dragging_info.rs

use serde::{Deserialize, Serialize};
use crate::ecs::entity::Entity;
use crate::ecs::component::Component; // Component トレイトを使うためにインポート

/// ドラッグ中のカードに関する情報を表すコンポーネントだよ！🖱️➡️🃏
/// これは内部的な状態管理に使うもので、Wasm 公開は不要かも？ (一旦 #[wasm_bindgen] は付けない)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DraggingInfo {
    /// ドラッグ開始前の、スタック内での順番 (usize より u8 の方が適切かも？要検討)
    pub original_position_in_stack: usize,
    /// ドラッグ開始前のスタックの種類とインデックスを特定するための情報？
    /// Entity で持つのが適切かは要検討。StackType と stack_index を持つ方が良いかも？
    pub original_stack_entity: Entity, // Entity ID を直接持つ？ u32 がいい？
    /// ドラッグ開始地点の X 座標 (f64 より f32 の方が一般的かも？要検討)
    pub original_x: f64,
    /// ドラッグ開始地点の Y 座標
    pub original_y: f64,
    /// ドラッグ開始時のマウスとカード左上のオフセット X
    pub offset_x: f64,
    /// ドラッグ開始時のマウスとカード左上のオフセット Y
    pub offset_y: f64,
}

// この構造体が Component であることを示すマーカー実装
impl Component for DraggingInfo {} 