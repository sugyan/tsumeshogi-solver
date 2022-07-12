mod node;
pub mod search;
mod searcher;
mod traits;

pub use node::Node;
pub use searcher::DefaultSearcher;
pub use traits::*;

pub type U = u32;
pub const INF: U = U::MAX;
