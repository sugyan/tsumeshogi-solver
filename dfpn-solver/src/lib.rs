pub mod generate_moves;
pub mod impl_default_hash;
pub mod impl_hashmap_table;
pub mod impl_vec_table;
pub mod impl_zobrist_hash;
pub mod solver;

pub use generate_moves::*;
pub use solver::*;

type U = u32;
pub const INF: U = U::MAX;
