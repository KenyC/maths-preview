pub mod error;
mod geometry;
mod render;
mod svg;
mod glyph_to_character;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(target_arch = "wasm32")]
pub mod web;
