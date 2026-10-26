//! glyphwm — the GlyphOS Wayland compositor (Smithay foundation, phase 3).
//!
//! Status: **scaffold**. This crate exists to keep the compositor project
//! structurally first-class in the repository. Building without `run` costs
//! almost nothing and yields a binary that prints its honest status; the
//! `run` feature compiles the actual Smithay backend.
//!
//! Implementation plan (order follows docs/ROADMAP.md phase 3):
//! 1. Backend selection: libinput/udev seat session + gles2 renderer
//! 2. Window map: floating-first with tiling layouts (columns/rows/split)
//! 3. wlr-layer-shell + xdg-shell + xwayland (via smithay's X11 stack)
//! 4. Workspace strip + overview protocol shared with glyph-shell
//! 5. Screenshot txn (wlroots protocol pipelines reused where licensed)

#[cfg(not(feature = "run"))]
fn main() {
    println!("glyphwm scaffold (no `run` feature).");
    println!("The GlyphOS desktop currently rides the documented dev fallback");
    println!("compositor adapter in glyph-shell. Build with:");
    println!("  cargo run -p glyphwm --features run   # once Smithay lands");
    println!("Nothing here pretends to be a finished compositor.");
}

#[cfg(feature = "run")]
fn main() {
    run_glyphwm();
}

#[cfg(feature = "run")]
mod state; // compositor window state: floating, tiling, workspaces

#[cfg(feature = "run")]
mod backends; // libinput/udev + x11 fallback; renderer init

#[cfg(feature = "run")]
fn run_glyphwm() {
    // Skeleton: bring up the Wayland listener, seat + graphics, then the
    // window/tree manager, mirroring smithay's layered examples.
    state::run();
}
