pub mod search;
mod searcher;
pub mod types;

pub use searcher::*;
pub use types::*;

pub type U = u32;
pub const INF: U = U::MAX;
