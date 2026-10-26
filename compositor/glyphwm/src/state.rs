//! glyphwm compositor state: window tree (floating + tiling), workspaces,
//! stacking. Compiled only with `--features run`.

/// Placeholder structure for the milestone-3 implementation surfaced here so
/// reviewers can see the target data model without pretending it runs yet.
///
/// - `windows` keeps insertion/focus order per workspace
/// - `tiling` describes the layout algorithm chosen per workspace split
/// - all fields are real state once `run` is enabled; nothing is simulated

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Floating,
    Columns,
    Rows,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workspace {
    pub id: u32,
    pub layout: Layout,
}

pub fn run() {
    eprintln!("glyphwm: compositor state module compiled; backend bring-up pending (phase 3).");
}
