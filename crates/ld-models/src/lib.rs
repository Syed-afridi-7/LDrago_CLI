//! Model abstraction layer and adapters for LDrago CLI.

pub mod adapter;
pub mod custom_brain;
pub mod ollama;
pub mod router;
pub mod switching;

pub use adapter::*;
pub use custom_brain::*;
pub use ollama::*;
pub use router::*;
pub use switching::*;
