//! 山札 (Stock) と捨て札 (Waste) に関するルールを定義するよ。

use crate::ecs::entity::Entity;
use crate::ecs::world::World;
// 他のルール関数 (tableau, foundation) を使うためにインポート
use super::tableau::can_move_to_tableau;
use super::foundation::can_move_to_foundation;

/// ストック（山札）からウェスト（捨て札）にカードを配れるかチェックする。
pub fn can_deal_from_stock(stock_is_empty: bool) -> bool {
    !stock_is_empty
}

/// ストック（山札）が空のときに、ウェスト（捨て札）からストックにカードを戻せるかチェックする。
pub fn can_reset_stock_from_waste(stock_is_empty: bool, waste_is_empty: bool) -> bool {
    stock_is_empty && !waste_is_empty
}

/// ウェスト（捨て札）の一番上のカードが、特定の場札 (Tableau) の一番上に置けるかチェックする。
pub fn can_move_from_waste_to_tableau(
    world: &World,
    waste_top_card_entity: Entity,
    target_tableau_index: u8,
) -> bool {
    can_move_to_tableau(world, waste_top_card_entity, target_tableau_index)
}

/// ウェスト（捨て札）の一番上のカードが、特定の組札 (Foundation) の一番上に置けるかチェックする。
pub fn can_move_from_waste_to_foundation(
    world: &World,
    waste_top_card_entity: Entity,
    target_foundation_index: u8,
) -> bool {
    can_move_to_foundation(world, waste_top_card_entity, target_foundation_index)
} 