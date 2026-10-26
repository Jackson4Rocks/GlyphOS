//! Glyph Command — the central command bar (Super+Space): real app search,
//! real window switching, real workspace jumps, graceful errors.

use crate::apps;
use crate::compositor;
use crate::state::ShellState;
use crate::ui::widget_helpers::*;
use gtk::prelude::*;
use gtk::{glib, Box as VBox, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::rc::Rc;
use std::sync::Arc;

/// A single scored search result.
#[derive(Debug, Clone)]
enum Hit {
    Launch { name: String, exec: String },
    SwitchWindow { title: String, class: String, addr: String },
    Workspace(u32),
}

impl Hit {
    fn label(&self) -> String {
        match self {
            Hit::Launch { name, .. } => format!("▸ {name}"),
            Hit::SwitchWindow { title, class, .. } => {
                format!("▣ {class} — {}", trunc(&title, 40))
            }
            Hit::Workspace(n) => format!("⌗ workspace {n}"),
        }
    }
}

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n { s.to_string() } else {
        format!("{}…", s.chars().take(n).collect::<String>())
    }
}

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("Glyph Command").build();
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::OnDemand,
        &[Edge::Top],
        &[(Edge::Top, 60)],
        0,
        "glyphos-command",
    );
    win.set_default_size(560, 360);
    win.add_css_class("glyph-root");
    let cmdbar = VBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(cls(&["cmdbar"]))
        .build();
    let outer = gtk::Box::builder().css_classes(cls(&["glyph-root"])).build();
    outer.append(&cmdbar);
    win.set_child(Some(&outer));

    // ——— prompt ———
    let prompt = Entry::builder()
        .placeholder_text("Search apps, windows, workspaces…")
        .css_classes(cls(&["sensitive-entry","body"]))
        .build();
    margin_all(&prompt, 8);
    cmdbar.append(&prompt);

    // ——— results ———
    let list = ListBox::builder().build();
    let store: Rc<std::cell::RefCell<Vec<Hit>>> = Rc::new(std::cell::RefCell::new(Vec::new()));
    let scroll = ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .min_content_height(140)
        .build();
    margin_all(&scroll, 4);
    cmdbar.append(&scroll);

    // ——— hint ———
    let hint = ui_label("↟↟ enter launch · ↑↓ navigate · esc close · try “workspace 2”", "small dim");
    cmdbar.append(&hint);

    // ——— query → results ———
    {
        let store = store.clone();
        let list = list.clone();
        let prompt = prompt.clone();
        let wc = win.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
            let query = prompt.text().to_string();
            let hits = compute_hits(&query);
            if query != *last_query(&store).borrow() {
                *last_query(&store).borrow_mut() = query.clone();
                rebuild_results(&list, &mut *store.borrow_mut(), hits.clone());
            }
            let _ = &wc;
            glib::ControlFlow::Continue
        });
    }
    // ——— keyboard behavior on the command window ———
    {
        let wc = win.clone();
        let list = list.clone();
        let store = store.clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                gtk::gdk::Key::Escape => {
                    wc.hide();
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Return => {
                    if let Some(row) = list.selected_row() {
                        let idx = row.index() as usize;
                        if let Some(hit) = store.borrow().get(idx).cloned() {
                            run_hit(&hit, &wc);
                        }
                    }
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Up | gtk::gdk::Key::Down => {
                    let delta = if key == gtk::gdk::Key::Down { 1 } else { -1 };
                    move_selection(&list, delta);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        win.add_controller(controller);
    }

    // Focus whenever shown; register for IPC toggling.
    let prompt_focus = prompt.clone();
    win.connect_map(move |_| { prompt_focus.grab_focus(); });
    crate::dispatch::register("command", win.clone());
}

thread_local! {
    // Sentinel ≠ any user text so the first open always rebuilds results.
    static LAST_QUERY: Rc<std::cell::RefCell<String>> = Rc::new(std::cell::RefCell::new(String::from("\u{1}glyphos-init")));
}

fn last_query(_store: &Rc<std::cell::RefCell<Vec<Hit>>>) -> Rc<std::cell::RefCell<String>> {
    LAST_QUERY.with(|q| q.clone())
}

fn rebuild_results(list: &ListBox, store: &mut Vec<Hit>, hits: Vec<Hit>) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    *store = hits.clone();
    let mut first_row: Option<ListBoxRow> = None;
    for hit in hits.iter() {
        let row = ListBoxRow::builder().css_classes(cls(&["list-row","body"])).build();
        let lbl = ui_label(&hit.label(), "body dim");
        row.set_child(Some(&lbl));
        if first_row.is_none() { first_row = Some(row.clone()); }
        list.append(&row);
    }
    if let Some(r) = first_row {
        list.select_row(Some(&r));
    }
}

fn move_selection(list: &ListBox, delta: i32) {
    let n = list.observe_children().n_items() as i32;
    if n == 0 { return; }
    let cur = list.selected_row().map(|r| r.index()).unwrap_or(-1);
    let next = (cur + delta).clamp(0, n - 1);
    let row = list.row_at_index(next);
    if let Some(row) = row.and_downcast::<ListBoxRow>() {
        list.select_row(Some(&row));
    }
}

fn run_hit(hit: &Hit, win: &Window) {
    match hit {
        Hit::Launch { exec, .. } => {
            crate::dispatch::launch(exec);
            win.hide();
        }
        Hit::SwitchWindow { addr, .. } => {
            compositor::focus(addr);
            win.hide();
        }
        Hit::Workspace(n) => {
            compositor::switch_workspace(*n);
            win.hide();
        }
    }
}

/// Score and build the result list from real data sources
/// (installed desktop apps, actual compositor windows, workspace commands).
fn compute_hits(query: &str) -> Vec<Hit> {
    let q = query.to_lowercase();
    let mut hits = Vec::new();

    // Workspace jump command.
    if let Some(rest) = q.strip_prefix("workspace") {
        let n = rest.trim().chars().next().and_then(|c| c.to_digit(10));
        if let Some(n) = n {
            hits.push(Hit::Workspace(n as u32));
        }
    }

    // Apps from .desktop registry (empty query = top few).
    for a in apps::desktop_apps() {
        if hits.len() >= 6 { break; }
        if a.no_display { continue; }
        if q.is_empty() || a.name.to_lowercase().contains(&q) {
            hits.push(Hit::Launch { name: a.name, exec: a.exec });
        }
    }

    // Running windows from the compositor cache.
    for w in compositor::windows() {
        if hits.len() >= 10 { break; }
        if q.is_empty() || w.title.to_lowercase().contains(&q) || w.class.to_lowercase().contains(&q) {
            hits.push(Hit::SwitchWindow { title: w.title, class: w.class, addr: w.address });
        }
    }
    hits
}

// Hidden deps: state kept for future config-aware search (recent commands).
pub fn _keep_state_alive(_st: &Arc<ShellState>) {}
