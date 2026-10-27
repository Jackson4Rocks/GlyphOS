//! Window-management policy shared by the future Smithay runtime.
//!
//! This module deliberately has no Wayland or graphics dependencies, so the
//! layout rules can be tested independently of a compositor session.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Floating,
    Columns,
    Rows,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub id: WindowId,
    pub geometry: Rect,
    pub floating: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: u32,
    pub layout: Layout,
    pub gap: u32,
    windows: Vec<Window>,
    focused: Option<WindowId>,
}

impl Workspace {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            layout: Layout::Floating,
            gap: 8,
            windows: Vec::new(),
            focused: None,
        }
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focused
    }

    pub fn add_window(&mut self, window: Window) {
        if let Some(existing) = self.windows.iter_mut().find(|item| item.id == window.id) {
            *existing = window;
        } else {
            self.windows.push(window);
        }
        self.focused = Some(window.id);
    }

    pub fn focus(&mut self, id: WindowId) -> bool {
        if self.windows.iter().any(|window| window.id == id) {
            self.focused = Some(id);
            true
        } else {
            false
        }
    }

    pub fn remove_window(&mut self, id: WindowId) -> Option<Window> {
        let index = self.windows.iter().position(|window| window.id == id)?;
        let removed = self.windows.remove(index);
        if self.focused == Some(id) {
            self.focused = self.windows.last().map(|window| window.id);
        }
        Some(removed)
    }

    /// Return window positions in stacking order for the requested output area.
    /// Floating windows retain their saved geometry in every tiling layout.
    pub fn arrange(&self, area: Rect) -> Vec<(WindowId, Rect)> {
        let mut arranged = self
            .windows
            .iter()
            .filter(|window| window.floating || self.layout == Layout::Floating)
            .map(|window| (window.id, window.geometry))
            .collect::<Vec<_>>();
        let tiled = self
            .windows
            .iter()
            .filter(|window| !window.floating && self.layout != Layout::Floating)
            .map(|window| window.id)
            .collect::<Vec<_>>();

        if tiled.is_empty() {
            return arranged;
        }

        let gap = self.gap.min(i32::MAX as u32);
        let inner = inset(area, gap);
        let tiles = match self.layout {
            Layout::Floating => Vec::new(),
            Layout::Columns => split_rects(inner, tiled.len(), gap, Axis::Horizontal),
            Layout::Rows => split_rects(inner, tiled.len(), gap, Axis::Vertical),
            Layout::Split => split_master_stack(inner, tiled.len(), gap),
        };
        arranged.extend(tiled.into_iter().zip(tiles));
        arranged
    }
}

#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

fn inset(area: Rect, gap: u32) -> Rect {
    let gap_i32 = gap as i32;
    let outer = gap.saturating_mul(2);
    Rect {
        x: area.x.saturating_add(gap_i32),
        y: area.y.saturating_add(gap_i32),
        width: area.width.saturating_sub(outer),
        height: area.height.saturating_sub(outer),
    }
}

fn split_rects(area: Rect, count: usize, gap: u32, axis: Axis) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }
    let gap = gap.min(i32::MAX as u32);
    let count_u32 = count.min(u32::MAX as usize) as u32;
    let gaps = gap.saturating_mul(count_u32.saturating_sub(1));
    let total = match axis {
        Axis::Horizontal => area.width,
        Axis::Vertical => area.height,
    };
    let available = total.saturating_sub(gaps);
    let base = available / count_u32;
    let remainder = available % count_u32;
    let mut offset = 0u32;

    (0..count_u32)
        .map(|index| {
            let size = base + u32::from(index < remainder);
            let rect = match axis {
                Axis::Horizontal => Rect {
                    x: area.x.saturating_add(offset.min(i32::MAX as u32) as i32),
                    y: area.y,
                    width: size,
                    height: area.height,
                },
                Axis::Vertical => Rect {
                    x: area.x,
                    y: area.y.saturating_add(offset.min(i32::MAX as u32) as i32),
                    width: area.width,
                    height: size,
                },
            };
            offset = offset.saturating_add(size).saturating_add(gap);
            rect
        })
        .collect()
}

fn split_master_stack(area: Rect, count: usize, gap: u32) -> Vec<Rect> {
    if count < 2 {
        return vec![area];
    }
    let gap = gap.min(i32::MAX as u32);
    let available_width = area.width.saturating_sub(gap);
    let master_width = ((u64::from(available_width) * 3) / 5) as u32;
    let stack_width = available_width.saturating_sub(master_width);
    let stack_x = area
        .x
        .saturating_add(master_width.min(i32::MAX as u32) as i32)
        .saturating_add(gap as i32);
    let mut rects = vec![Rect {
        width: master_width,
        ..area
    }];
    rects.extend(split_rects(
        Rect {
            x: stack_x,
            width: stack_width,
            ..area
        },
        count - 1,
        gap,
        Axis::Vertical,
    ));
    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(id: u64, floating: bool) -> Window {
        Window {
            id: WindowId(id),
            geometry: Rect {
                x: 40 + id as i32,
                y: 30,
                width: 400,
                height: 300,
            },
            floating,
        }
    }

    #[test]
    fn columns_fill_area_with_even_tiles_and_gaps() {
        let mut workspace = Workspace::new(1);
        workspace.layout = Layout::Columns;
        workspace.gap = 10;
        for id in 1..=3 {
            workspace.add_window(window(id, false));
        }

        let tiles = workspace.arrange(Rect {
            x: 0,
            y: 0,
            width: 1000,
            height: 600,
        });

        assert_eq!(tiles.len(), 3);
        assert_eq!(
            tiles[0].1,
            Rect {
                x: 10,
                y: 10,
                width: 320,
                height: 580
            }
        );
        assert_eq!(tiles[1].1.x, 340);
        assert_eq!(tiles[2].1.x, 670);
        assert_eq!(tiles[2].1.x + tiles[2].1.width as i32, 990);
    }

    #[test]
    fn rows_and_split_produce_distinct_layouts() {
        let mut workspace = Workspace::new(1);
        for id in 1..=3 {
            workspace.add_window(window(id, false));
        }
        let output = Rect {
            x: 0,
            y: 0,
            width: 1200,
            height: 800,
        };

        workspace.layout = Layout::Rows;
        let rows = workspace.arrange(output);
        workspace.layout = Layout::Split;
        let split = workspace.arrange(output);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].1.width, 1184);
        assert!(rows[1].1.y > rows[0].1.y);
        assert_eq!(split.len(), 3);
        assert!(split[1].1.x > split[0].1.x);
        assert!(split[2].1.y > split[1].1.y);
    }

    #[test]
    fn floating_windows_keep_geometry_when_other_windows_tile() {
        let mut workspace = Workspace::new(1);
        workspace.layout = Layout::Columns;
        let floating = window(1, true);
        workspace.add_window(floating);
        workspace.add_window(window(2, false));

        let arranged = workspace.arrange(Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        });

        assert_eq!(arranged[0], (floating.id, floating.geometry));
        assert_eq!(arranged[1].0, WindowId(2));
        assert_ne!(arranged[1].1, floating.geometry);
    }

    #[test]
    fn focus_and_removal_keep_workspace_state_consistent() {
        let mut workspace = Workspace::new(2);
        workspace.add_window(window(1, false));
        workspace.add_window(window(2, false));
        assert!(workspace.focus(WindowId(1)));
        assert_eq!(workspace.focused(), Some(WindowId(1)));

        assert_eq!(workspace.remove_window(WindowId(1)), Some(window(1, false)));
        assert_eq!(workspace.focused(), Some(WindowId(2)));
        assert!(!workspace.focus(WindowId(99)));
    }

    #[test]
    fn tiny_outputs_and_large_gaps_never_overflow() {
        let mut workspace = Workspace::new(1);
        workspace.layout = Layout::Columns;
        workspace.gap = u32::MAX;
        workspace.add_window(window(1, false));
        workspace.add_window(window(2, false));

        let tiles = workspace.arrange(Rect {
            x: i32::MAX - 2,
            y: 0,
            width: 1,
            height: 1,
        });
        assert_eq!(tiles.len(), 2);
        assert!(tiles
            .iter()
            .all(|(_, rect)| rect.width == 0 && rect.height == 0));
    }
}
