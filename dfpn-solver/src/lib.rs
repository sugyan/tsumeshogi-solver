mod default_solver;
pub mod impl_hashmap_table;
pub mod impl_vec_table;
pub mod solve;
mod types;

pub use default_solver::*;
pub use types::*;

type U = u32;
pub const INF: U = U::MAX;
