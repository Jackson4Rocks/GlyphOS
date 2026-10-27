//! The clock flyout ("quick panel"): calendar, media transport, notifications.
//! Real data only — chrono for the month, playerctl for MPRIS, the daemon
//! store for notification history. The centered clock opens this.

use crate::state::{Notif, ShellState};
use crate::ui::widget_helpers::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use gtk::glib;
use gtk::prelude::*;
use gtk::Window;
use std::sync::Arc;

struct Hold {
    holder: gtk::Box,
    st: Arc<ShellState>,
}

thread_local! {
    static HOLD: std::cell::RefCell<Option<Hold>> = const { std::cell::RefCell::new(None) };
}

pub fn build(st: &Arc<ShellState>, app: &gtk::Application) {
    let win = Window::builder().application(app).title("Glyph Quick").build();
    // Full-width strip below the panel; the flyout itself is centered inside.
    glass_layer(
        &win,
        Layer::Top,
        KeyboardMode::OnDemand,
        &[Edge::Top, Edge::Left, Edge::Right],
        &[(Edge::Top, 96)],
        0,
        "glyphos-quick",
    );
    win.add_css_class("glyph-root");

    let holder = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    win.set_child(Some(&holder));

    render_into(&holder, st);
    HOLD.with(|h| {
        *h.borrow_mut() = Some(Hold { holder: holder.clone(), st: Arc::clone(st) })
    });
    let wref = win.clone();
    crate::dispatch::register("quick", win);

    // A click that lands outside the flyout card (on the strip background)
    // closes it, and so does Escape — both standard popover behavior.
    let outside = gtk::GestureClick::new();
    {
        let holder = holder;
        let wref = wref.clone();
        outside.connect_pressed(move |_, _, x, y| {
            let picked = wref.pick(x, y, gtk::PickFlags::DEFAULT.union(gtk::PickFlags::NON_TARGETABLE));
            let inside = picked.map(|w| w.is_ancestor(&holder)).unwrap_or(false);
            if !inside { crate::dispatch::hide_overlays(); }
        });
    }
    wref.add_controller(outside);

    let esc = gtk::EventControllerKey::new();
    esc.set_propagation_phase(gtk::PropagationPhase::Capture);
    esc.connect_key_pressed(|_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            crate::dispatch::hide_overlays();
        }
        gtk::glib::Propagation::Proceed
    });
    wref.add_controller(esc);

    // Keep the data honest while shown: re-read media state and the
    // notification store on a short tick.
    glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
        if wref.get_visible() {
            refresh();
        }
        glib::ControlFlow::Continue
    });
}

/// Re-render contents (open + visible ticks + store mutations).
pub fn refresh() {
    HOLD.with(|h| {
        let holder = h.borrow_mut().as_mut().map(|hold| (hold.holder.clone(), hold.st.clone()));
        let Some((holder, st)) = holder else { return };
        render_into(&holder, &st);
    });
}

fn render_into(holder: &gtk::Box, st: &Arc<ShellState>) {
    while let Some(child) = holder.first_child() {
        child.unparent();
    }

    let shell = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .css_classes(cls(&["glass-pop", "quick-shell"]))
        .build();
    // Explicit geometry: layer surfaces in this GTK combo mis-measure natural
    // size on first map (content collapsed to a sliver); fixed pixels are safe.
    shell.set_size_request(720, 400);

    let cal = calendar_card();
    cal.set_valign(gtk::Align::Fill);
    cal.set_vexpand(true);
    shell.append(&cal);

    let right = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .css_classes(cls(&["quick-col"]))
        .build();
    let media = media_card();
    right.append(&media);
    let notif = notif_card(st);
    notif.set_vexpand(true);
    right.append(&notif);
    right.set_valign(gtk::Align::Fill);
    right.set_vexpand(true);
    shell.append(&right);

    holder.append(&shell);
}

// ---------------------------------------------------------------- calendar

fn month_name(m: u32) -> &'static str {
    const MONTHS: [&str; 13] = [
        "Unknown", "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    MONTHS[m.min(12) as usize]
}

fn calendar_card() -> gtk::Box {
    use chrono::Datelike;

    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .css_classes(cls(&["quick-card"]))
        .build();

    let now = chrono::Local::now();
    let head = format!("{} {}", month_name(now.month()).to_uppercase(), now.year());
    card.append(&ndot_label(&head, "ndot-sm"));

    let grid = gtk::Grid::builder().column_spacing(12).row_spacing(6).build();

    for (i, d) in ["MO","TU","WE","TH","FR","SA","SU"].iter().enumerate() {
        let l = ui_label(d, "ndot-xs dim");
        l.set_halign(gtk::Align::Fill);
        l.set_hexpand(true);
        l.set_justify(gtk::Justification::Center);
        grid.attach(&l, i as i32, 0, 1, 1);
    }

    let first = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or(now.date_naive());
    let offset = first.weekday().num_days_from_monday() as i32;
    let days = days_in_month(now.year(), now.month());
    for day in 1..=days {
        let idx = offset + day as i32 - 1;
        let col = idx % 7;
        let row = idx / 7 + 1;
        let cls = if day == now.day() { "cal-day today" } else { "cal-day" };
        let cell = ui_label(&day.to_string(), cls);
        cell.set_hexpand(true);
        cell.set_halign(gtk::Align::Fill);
        cell.set_justify(gtk::Justification::Center);
        grid.attach(&cell, col, row, 1, 1);
    }

    card.append(&grid);
    card
}

fn days_in_month(y: i32, m: u32) -> u32 {
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    match m {
        1|3|5|7|8|10|12 => 31,
        4|6|9|11 => 30,
        2 => if leap { 29 } else { 28 },
        _ => 30,
    }
}

// ---------------------------------------------------------------- media

fn media_card() -> gtk::Box {
    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .css_classes(cls(&["quick-card"]))
        .build();

    card.append(&ndot_label("MEDIA", "ndot-xs"));

    let snap = glyph_system::events::snap_cache().read().map(|s| s.clone()).unwrap_or_default();
    let m = snap.media;
    if !m.available {
        card.append(&ui_label("Nothing is playing.", "body dim"));
        card.append(&caption("No media players on the session bus."));
        return card;
    }

    let title = ui_label(&m.title, "ink");
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let artist = ui_label(&m.artist, "small dim");
    artist.set_ellipsize(gtk::pango::EllipsizeMode::End);
    card.append(&title);
    card.append(&artist);
    card.append(&caption(&format!("via {}", m.player)));

    let bar = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .build();
    bar.append(&transport("⏮", "Previous", "previous"));
    bar.append(&transport(if m.playing { "⏸" } else { "▶" }, "Play / Pause", if m.playing { "pause" } else { "play" }));
    bar.append(&transport("⏭", "Next", "next"));
    // Push the transport toward the bottom of the card.
    let filler = gtk::Box::new(gtk::Orientation::Vertical, 0);
    filler.set_vexpand(true);
    card.append(&filler);
    card.append(&bar);
    card
}

fn transport(label: &str, tooltip: &str, cmd: &'static str) -> gtk::Button {
    let b = gtk::Button::builder()
        .label(label)
        .css_classes(cls(&["pill", "transport", "ndot-md"]))
        .tooltip_text(tooltip)
        .build();
    b.connect_clicked(move |_| {
        // Subprocess fire-and-forget on a worker thread (UI stays smooth).
        std::thread::spawn(move || { let _ = glyph_system::media::media_cmd(cmd); });
        glib::timeout_add_local(std::time::Duration::from_millis(500), || {
            refresh();
            glib::ControlFlow::Break
        });
    });
    b
}

fn caption(text: &str) -> gtk::Label {
    ui_label(text, "ndot-xs dim")
}

// ---------------------------------------------------------------- notifications

fn notif_card(st: &Arc<ShellState>) -> gtk::Box {
    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .css_classes(cls(&["quick-card"]))
        .build();

    card.append(&ndot_label("NOTIFICATIONS", "ndot-xs"));

    let list: Vec<Notif> = st.notifs.lock().unwrap().iter().rev().take(4).cloned().collect();
    if list.is_empty() {
        card.append(&ui_label("No notifications.", "small dim"));
    } else {
        for n in &list {
            let row = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(2)
                .css_classes(cls(&["list-row", "notif-row"]))
                .build();
            let head = ui_label(&n.summary, "ink");
            head.set_ellipsize(gtk::pango::EllipsizeMode::End);
            let app = ui_label(&n.app, "ndot-xs dim");
            row.append(&head);
            row.append(&app);
            card.append(&row);
        }
    }

    // The footer always exists: empty history shows a disabled clear button
    // instead of popping the button in and out on every re-render.
    let clear = gtk::Button::builder()
        .label("CLEAR ALL")
        .sensitive(!list.is_empty())
        .css_classes(cls(&["pill", "ndot-xs", "dim"]))
        .build();
    let st2 = Arc::clone(st);
    clear.connect_clicked(move |_| {
        let drained = st2.notifs.lock().unwrap().drain(..).collect::<Vec<_>>();
        for n in drained {
            st2.notif_close_tx.send((n.id, 2)).ok(); // 2 = dismissed by user
        }
        refresh();
    });
    // Pin the footer to the card's bottom edge.
    let filler = gtk::Box::new(gtk::Orientation::Vertical, 0);
    filler.set_vexpand(true);
    card.append(&filler);
    card.append(&clear);
    card
}
