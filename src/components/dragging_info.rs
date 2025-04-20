// src/components/dragging_info.rs

use serde::{Deserialize, Serialize};
use crate::ecs::entity::Entity;
use crate::ecs::component::Component; // Component トレイトを使うためにインポート
use crate::components::stack::StackType; // StackType を使うためにインポート

/// ドラッグ中のカードに関する情報を表すコンポーネントだよ！🖱️➡️🃏
/// これは内部的な状態管理に使うもので、Wasm 公開は不要かも？ (一旦 #[wasm_bindgen] は付けない)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DraggingInfo {
    /// グループ全体の元のスタック情報
    pub original_stack_type: StackType,
    /// グループ全員の元のスタック内位置 (ソート済み)
    /// (Entity, original_position_in_stack: u8)
    pub original_group_positions: Vec<(Entity, u8)>,
    /// ドラッグ開始地点の X 座標 (f64 より f32 の方が一般的かも？要検討)
    pub original_x: f64,
    /// ドラッグ開始地点の Y 座標
    pub original_y: f64,
    /// ドラッグ開始時のマウスとカード左上のオフセット X
    pub offset_x: f64,
    /// ドラッグ開始時のマウスとカード左上のオフセット Y
    pub offset_y: f64,
    /// グループドラッグ用のフィールド (一緒にドラッグされているエンティティのリスト)
    pub dragged_group: Vec<Entity>,
}

// この構造体が Component であることを示すマーカー実装
impl Component for DraggingInfo {} 