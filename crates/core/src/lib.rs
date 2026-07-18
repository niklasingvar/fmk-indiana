//! indiana_core — the engine. Core computes; faces render (IN_PRINCIPLES.md).
//!
//! Modules land per milestone: markers + parser (M2), walk + index (M3),
//! id + write chokepoint (M7), scope (M8), compiler (M9).

pub mod compile;
pub mod cos;
pub mod cursor;
pub mod frontmatter;
pub mod id;
pub mod index;
pub mod markers;
pub mod parser;
pub mod scope;
pub mod system_prompt;
pub mod templates;
pub mod walk;
pub mod write;
