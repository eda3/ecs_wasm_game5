// src/app/stock_handler.rs
//! Handles logic related to clicking the Stock pile (dealing to Waste, resetting Waste).

use crate::ecs::world::World;
use crate::ecs::entity::Entity;
use crate::components::{Card, Position, StackInfo, StackType};
use crate::logic::rules::stock_waste; // Use the rule checks
use crate::app::layout_calculator;
use crate::log;
use log::{warn, info}; // Import specific log levels

/// Deals one card from the Stock pile to the Waste pile.
/// Returns true if a card was dealt, false otherwise.
pub fn deal_one_card_from_stock(world: &mut World) -> bool {
    info!("Attempting to deal card from Stock to Waste...");

    // Find the top card in the Stock pile
    let stock_cards = find_cards_in_stack(world, StackType::Stock);
    if stock_cards.is_empty() {
        info!("  Stock is empty. Cannot deal.");
        return false; // Nothing to deal
    }

    // Check rule (optional, but good practice)
    if !stock_waste::can_deal_from_stock(false /* stock_is_empty = false */) {
         warn!("  Rule check failed: Cannot deal from stock (logic error?).");
         return false;
    }

    // Find the highest position_in_stack in Stock
    let top_card_entity_opt = stock_cards.iter()
        .max_by_key(|(_, stack_info)| stack_info.position_in_stack)
        .map(|(entity, _)| *entity); // Get the Entity

    if let Some(top_card_entity) = top_card_entity_opt {
        info!("  Dealing card: {:?}", top_card_entity);

        // Calculate new position in Waste
        let waste_cards = find_cards_in_stack(world, StackType::Waste);
        let new_pos_in_waste = waste_cards.len() as u8;

        // Calculate new physical position
        let new_position = layout_calculator::calculate_card_position(
            StackType::Waste,
            new_pos_in_waste,
            world,
        );

        // Update the card's components
        if let Some(stack_info) = world.get_component_mut::<StackInfo>(top_card_entity) {
            stack_info.stack_type = StackType::Waste;
            stack_info.position_in_stack = new_pos_in_waste;
        } else { warn!("  Failed to get StackInfo for {:?}", top_card_entity); }

        if let Some(position) = world.get_component_mut::<Position>(top_card_entity) {
            *position = new_position;
        } else { warn!("  Failed to get Position for {:?}", top_card_entity); }

        if let Some(card) = world.get_component_mut::<Card>(top_card_entity) {
            card.is_face_up = true; // Card dealt to Waste is face up
        } else { warn!("  Failed to get Card for {:?}", top_card_entity); }

        info!("  Card {:?} moved to Waste.", top_card_entity);
        true // Card was dealt
    } else {
        warn!("  Could not find top card in Stock, even though it's not empty.");
        false
    }
}

/// Resets the Waste pile back to the Stock pile when Stock is empty.
/// Returns true if the reset was performed, false otherwise.
pub fn reset_waste_to_stock(world: &mut World) -> bool {
    info!("Attempting to reset Waste to Stock...");

    let stock_cards = find_cards_in_stack(world, StackType::Stock);
    let waste_cards = find_cards_in_stack(world, StackType::Waste);

    // Check rules
    if !stock_waste::can_reset_stock_from_waste(stock_cards.is_empty(), waste_cards.is_empty()) {
        info!("  Cannot reset Waste to Stock (Stock not empty or Waste empty).");
        return false;
    }

    info!("  Resetting {} cards from Waste to Stock.", waste_cards.len());

    // Sort waste cards by their position in waste (lowest first)
    let mut sorted_waste_cards = waste_cards;
    sorted_waste_cards.sort_by_key(|(_, stack_info)| stack_info.position_in_stack);

    // Move each card back to Stock
    for (i, (entity, _)) in sorted_waste_cards.iter().enumerate() {
        let new_pos_in_stock = i as u8;
        let new_position = layout_calculator::calculate_card_position(
            StackType::Stock,
            new_pos_in_stock,
            world,
        );

        if let Some(stack_info) = world.get_component_mut::<StackInfo>(*entity) {
            stack_info.stack_type = StackType::Stock;
            stack_info.position_in_stack = new_pos_in_stock;
        } else { warn!("  Failed to get StackInfo for waste card {:?}", entity); }

        if let Some(position) = world.get_component_mut::<Position>(*entity) {
            *position = new_position;
        } else { warn!("  Failed to get Position for waste card {:?}", entity); }

        if let Some(card) = world.get_component_mut::<Card>(*entity) {
            card.is_face_up = false; // Cards in Stock are face down
        } else { warn!("  Failed to get Card for waste card {:?}", entity); }
    }

    info!("  Waste pile reset to Stock complete.");
    true
}

/// Helper function to find all entities with Card and StackInfo in a specific stack.
fn find_cards_in_stack(world: &World, stack_type: StackType) -> Vec<(Entity, StackInfo)> {
    world.get_all_entities_with_component::<Card>()
        .iter()
        .filter_map(|&entity| {
            world.get_component::<StackInfo>(entity)
                .filter(|si| si.stack_type == stack_type)
                .map(|si| (entity, si.clone())) // Clone StackInfo to return ownership
        })
        .collect()
} 