// src/logic/rules/mod.rs
//! ソリティアのルール関連モジュールをまとめるよ！

pub mod common;
pub mod foundation;
pub mod tableau;
pub mod stock_waste;
pub mod win_condition;
pub mod move_validation;

#[cfg(test)]
mod tests;

// 各モジュールから公開したい関数をここで再エクスポート！
pub use common::*;
pub use foundation::*;
pub use stock_waste::*;
pub use move_validation::is_move_valid;

// サブモジュール内の主要な関数を、このモジュール (rules) の直下から使えるように re-export！
pub use tableau::can_move_to_tableau;
pub use win_condition::check_win_condition; 