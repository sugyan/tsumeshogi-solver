mod dfpn;
pub mod generate_moves;
pub mod impl_default_hash;
pub mod impl_extended_solver;
pub mod impl_hashmap_table;
pub mod impl_normal_solver;
pub mod impl_vec_table;
pub mod impl_zobrist_hash;
pub mod solver;

pub use generate_moves::*;
pub use impl_extended_solver::*;
pub use impl_normal_solver::*;
pub use solver::*;

type U = u32;
pub const INF: U = U::MAX;
