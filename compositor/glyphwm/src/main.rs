//! glyphwm — the GlyphOS Wayland compositor (Smithay foundation, phase 3).
//!
//! Status: **window policy prototype**. The crate has tested workspace and
//! layout rules, but does not yet create a Wayland display or manage surfaces.
//! The `run` feature enables Smithay for compositor development; runtime
//! backend bring-up is still pending.
//!
//! Implementation plan (order follows docs/ROADMAP.md phase 3):
//! 1. Backend selection: libinput/udev seat session + gles2 renderer
//! 2. Window map: floating-first with tiling layouts (columns/rows/split)
//! 3. wlr-layer-shell + xdg-shell + xwayland (via smithay's X11 stack)
//! 4. Workspace strip + overview protocol shared with glyph-shell
//! 5. Screenshot txn (wlroots protocol pipelines reused where licensed)

fn main() {
    #[cfg(feature = "run")]
    run_glyphwm();
    #[cfg(not(feature = "run"))]
    println!("glyphwm policy prototype (no Wayland compositor runtime yet).");
}

#[cfg(test)]
mod state; // compositor window state and deterministic layout policy

#[cfg(feature = "run")]
mod backends; // Smithay backend bring-up is pending

#[cfg(feature = "run")]
fn run_glyphwm() {
    eprintln!(
        "glyphwm: Smithay is enabled, but the Wayland backend and event loop are not implemented yet"
    );
}
