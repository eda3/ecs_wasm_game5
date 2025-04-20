// src/components/position.rs

// serde を使う宣言！位置情報をネットワークで送ったり保存したりするかも！
use serde::{Serialize, Deserialize};
// Component トレイトを使う宣言！Position がコンポーネントであることを示す！
use crate::ecs::component::Component;

/// 2D空間での位置を表すコンポーネントだよ！ (x, y) 座標を持つよ。📍
///
/// カードだったり、カードを置く場所（場札、山札、組札）だったり、
/// いろんなエンティティがこのコンポーネントを持つことになると思う！汎用性高い！✨
///
/// #[derive(...)] のおまじない！
/// - Debug: デバッグ表示用
/// - Clone: コピー可能に (位置情報はコピーして使う場面も多いかも？)
/// - PartialEq: 等しいか比較できるように (同じ位置にあるかチェックする時に使うかも！)
/// - Serialize, Deserialize: JSON などに変換できるように
///
/// 座標の型は `f32` (32ビット浮動小数点数) にしてみようかな？
/// 整数 (`i32`) でもいいけど、将来的にアニメーションとかで滑らかに動かしたい時に
/// 小数点以下も扱えると便利だからね！😉
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Position 構造体が Component であることを示すマーカー！ これ大事！✅
impl Component for Position {}

// --- テスト ---
#[cfg(test)]
mod tests {
    use super::*; // 上で定義した Position を使う
    use crate::ecs::component::Component; // Component トレイトもテストで使う

    #[test]
    fn create_position_component() {
        let pos = Position { x: 100.5, y: -50.0 };

        // 値がちゃんと設定されてるか確認
        assert_eq!(pos.x, 100.5);
        assert_eq!(pos.y, -50.0);

        // 比較がちゃんとできるか確認
        let pos_same = Position { x: 100.5, y: -50.0 };
        let pos_different = Position { x: 0.0, y: 0.0 };
        assert_eq!(pos, pos_same);
        assert_ne!(pos, pos_different);

        // デバッグ表示も確認
        println!("作成した位置: {:?}", pos);

        // Component トレイトが実装されているかチェック
        fn needs_component<T: Component>(_: T) {}
        needs_component(pos.clone());

        println!("Position コンポーネント作成テスト、成功！🎉");
    }
} 