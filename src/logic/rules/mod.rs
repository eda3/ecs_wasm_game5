// src/logic/rules/mod.rs
//! ソリティアのルール関連モジュールをまとめるよ！

pub mod common;
pub mod foundation;
pub mod tableau;
pub mod stock_waste;
pub mod win_condition;

#[cfg(test)]
mod tests;

// 各モジュールから公開したい関数をここで再エクスポート！
pub use common::*;
pub use foundation::*;
pub use tableau::*;
pub use stock_waste::*;
pub use win_condition::*; 