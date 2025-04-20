// src/ecs/mod.rs
//! ECS (Entity-Component-System) core implementation.

pub mod component;
pub mod entity;
pub mod system;
pub mod world;

// Re-export key types for easier use via `crate::ecs::X`
pub use component::Component;
pub use entity::Entity;
pub use system::System;
pub use world::World; 