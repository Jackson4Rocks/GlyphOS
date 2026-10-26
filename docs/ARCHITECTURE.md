# Architecture

## 1. Layered view

```
┌────────────────────────────────────────────────────────────────┐
│  GlyphOS Shell (crates/glyph-shell, GTK4 + gtk4-layer-shell)   │
│   panel · dock · Glyph Command · Control Center · notifications │
├────────────────────────────┬───────────────────────────────────┤
│ glyph-core (tokens, config │ glyph-system (real adapters:         │
│ protocol, IPC bus)         │ nmcli, BlueZ, PipeWire/wl-pulse,    │
│                            │ sysfs backlight + battery)          │
├────────────────────────────┴───────────────────────────────────┤
│ compositor adapter (compositor.rs)                              │
│   backend: "glyphwm" (planned) / "hyprland" (dev fallback)      │
├──────────────────────────────────────────────────────────────────┤
│ Wayland + wlr-layer-shell   │  kernel: KMS, DRM, backlight,    │
│                             │  power_supply, networkd stack    │
└──────────────────────────────────────────────────────────────────┘
```

### Compositor adapter — the honesty seam

`crates/glyph-shell/src/compositor.rs` is the *only* place that knows which
compositor is in charge. The shell works against a narrow surface:
workspaces, windows, focus, close. Two backends exist:

- **hyprland** — today's development fallback; a complete, running
  compositor. Real JSON IPC; never simulated.
- **glyphwm** — the custom Smithay compositor in `compositor/`. It will
  speak the same interface; until it exists as a working compositor, this
  file says so in plain terms.

The fallback is *documented, labeled as fallback*, and carries no branding:
that is the deliberate rule — modularity over disguise.

## 2. Design tokens (`glyph-core::tokens`)

One stylesheet function renders the *entire* shell CSS from tokens:

- `appearance`: dark / light / auto (auto resolves by time of day)
- `accent`: signature `#D71920` used sparingly
- `glass`: frosted / reduced / opaque (accessibility + weak-GPU fallback)
- `glass_opacity` (0..100), `blur_quality` (0..100)
- typography: display font (Dot matrix; user-supplied NDot supported via
  `display_font` path with fallback `Doto`), UI font `Inter`, mono
  `Space Mono`
- `corner_radius`, `animation_intensity` → generated `--anim` durations

Tokens → one CSS string → `install_css()` at startup plus live
`ReloadTheme` IPC (Control Center mode switch writes `glyphos.toml` and
hot-relaunches the stylesheet). No surface hardcodes colors.

### Glass materials

`GlassMaterial { blur_sigma, tint_opacity, border, background_dim }` —
large shells vs popovers get different defaults. Blur execution belongs to
the compositor backend (`hyprland` has blur; when `glyphwm` lands, its own
blur pipeline is wired). CSS `backdrop-filter` stays per-material so
`reduced` / `opaque` modes degrade gracefully by construction.

## 3. IPC bus

Unix socket `$XDG_RUNTIME_DIR/glyphos/shell.sock` (mode 0600), JSON-per-line.
`glyphctl` is the reference client; the same channel serves desktop
keybinds (e.g. Hyprland binds map to `glyphctl command` etc.).
Protocol: `{t:"command"|"control"|"notify"|"launch"|"workspace"|"reload-theme"|"ping"}` →
`{t:"ok"}` / `{t:"err",message}` / `{t:"pong",version,uptime_secs}`.

## 4. Process model & failure policy

- One shell process, several layer surfaces. If a *component* crashes the
  shell's restart is cheap (sub-second) and every surface re-registers on
  its own thread closures.
- Polling threads run at 500–700 ms into lock-guarded caches (`snap_cache`,
  `ws_cache`, `window_cache`); UI ticks read caches only — the GTK main
  thread never blocks on subprocesses or disk.
- System adapters return typed results; unavailable hardware surfaces an
  honest reason string ("no backlight device...") instead of hiding the UI.

## 5. Future phases (tracked in docs/ROADMAP.md)

- glyphwm (Smithay): floating + tiling, layer-shell, rootwm protocol.
- First-party applications (Files, Terminal, Settings, Store, Monitor…)
  consume `glyph-core` tokens and `glyph-system` adapters directly.
- D-Bus notification daemon resident in `glyph-shell`.
- dist/: archiso-based ISO, installer, polkit rules for backlight/store,
  greetd greeter integration.
