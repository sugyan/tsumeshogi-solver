mod generate_moves;
pub mod impl_hashmap_table;
pub mod impl_vec_table;
mod solver;
mod types;

pub use generate_moves::*;
pub use solver::*;
pub use types::*;

type U = u32;
pub const INF: U = U::MAX;
