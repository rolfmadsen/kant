//! Orthogonal Manhattan edge router and port bus decomposed into modular units under `src/ui/edge_router/`.

pub mod channels;
pub mod geometry;
pub mod router;
pub mod types;

pub use router::EdgeRouter;
pub use types::*;
