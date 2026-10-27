# Roadmap

Phases map 1:1 to the product plan; statuses updated only when verified.

## Phase 1 — Foundation ✅ (this commit)
- [x] Cargo workspace: glyph-core / glyph-system / glyph-shell / glyphctl
- [x] Design tokens → single CSS generator (dark, light, glass levels)
- [x] Config at `~/.config/glyphos/glyphos.toml` with persistence
- [x] IPC bus + `glyphctl` (ping/command/control/…)
- [x] Fonts: Doto (OFL) auto-install; user-supplied NDot supported
- [x] dev scripts + gtk4-layer-shell bootstrap

## Phase 2 — Working session (running on the dev compositor fallback)
- [x] Shell surfaces on wlr-layer-shell: desktop (wallpaper layer), panel,
      dock, command, control, notify, quick flyout
- [x] GlyphOS owns the desktop image (config `wallpaper`, honest fallback art)
- [x] Real workspace indicators + switching via compositor adapter
- [x] Glyph Command: desktop-entry search + window switch (compositor cache)
- [x] Control Center: real Wi-Fi/Bluetooth/volume/brightness/mode switching
- [x] Panel: app-grid launcher mark, centered Doto clock → quick flyout
      (calendar / media transport / notification history), stat cluster
- [x] glyphctl reachable from keybinds (`dist/hyprland/` example conf)
- [ ] Settings surface replaces Control Center "big" page (phase 6 app)

## Phase 3 — Window management
- [ ] tiling layouts and snapping live **in glyphwm**, not in the shell
- [ ] glyphwm: floating + tiling, workspaces, monitor hotplug (Smithay)
- [ ] window rules; keyboard-driven management parity with the plan
- [ ] overview surface in the shell reading glyphwm's real state

## Phase 4 — Glass engine & identity polish
- [x] tokens + material struct + fallback levels (done, live-tested)
- [x] Wallpaper: shell bottom layer serves the configured image; glyphwm
      per-workspace wallpapers remain a phase-3 follow-up
- [x] motion: token-driven hover/press transitions + flyout drop-in animation
- [ ] light theme photography pass (tokens exist; visual QA pending)

## Phase 5 — System interfaces
- [x] D-Bus notification daemon (real `org.freedesktop.Notifications` on the
      session bus; verified with `notify-send` + `busctl` close loop)
- [x] lock screen placeholder (marked "phase 5" in UI; honest)
- [x] Notification history view (quick flyout shows the live store; clear-all
      echoes real close notifications)
- [ ] notification actions/sounds (close reason 1 expiry not emitted)
- [ ] screenshot UI (grim plumbing exists in tests)
- [ ] greetd greeter (layer-shell root; reuses tokens)

## Phase 6 — First-party applications
- files / terminal / settings / store / monitor — each a crate consuming
  glyph-core tokens; Store runs pacman via polkit (no root GUI shortcuts)

## Phase 7 — Distribution
- archiso live image, installer, bootloader, first-boot wizard,
  update path, VM validation. **Nothing before phase 7 calls itself an ISO.**

## Phase 8 — Refinement
- accessibility audit (contrast, reduced-motion already tokenized),
  multi-monitor QA, theme fallback tests, crash-hardening drills.
