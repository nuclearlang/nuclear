//! Language server for NuclearLang (`.nuc` files): diagnostics, semantic highlighting, hover, completion,
//! signature help, go to definition, references, rename, outline, folding, quick fixes and code metrics
//! (cyclomatic complexity as code lenses).
//!
//! The server speaks LSP over stdin/stdout (`src/main.rs`). The active dialect comes from the
//! `nuclear.dialect` setting (`flutter` by default; `base` checks only the base language).

pub mod dialects;
pub mod features;
pub mod index;
pub mod metrics;
pub mod project;
pub mod rpc;
pub mod server;
pub mod text;
pub mod typedefs;
