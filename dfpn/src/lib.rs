mod default_searcher;
pub mod impl_hashmap_table;
pub mod impl_vec_table;
pub mod search;
mod types;

pub use default_searcher::*;
pub use types::*;

pub type U = u32;
pub const INF: U = U::MAX;
