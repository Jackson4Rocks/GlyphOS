# GlyphOS

> **Every detail. By design.** — a Nothing-inspired desktop operating system built on Arch Linux + Wayland.

GlyphOS is a real Linux desktop OS with its own integrated shell, compositor
adapter architecture, command bar, control center, and design-token theme
engine. It is not a theme, not a mockup, and not a web page that pretends to
be a computer. Every value you see in the shell (Wi-Fi, battery, audio,
brightness, workspaces, windows) comes from real system state.

---

## What is working right now (verified on this machine)

| Component | Status | Backend |
|---|---|---|
| `glyph-shell` (panel, dock, command, control center, notifications) | **running & verified** | GTK4 + gtk4-layer-shell (wlr-layer-shell) |
| Glyph Control Center | **running & verified** | NetworkManager, BlueZ, PipeWire/WirePlumber, backlight sysfs |
| Glyph Command (Super+Space) | **running** | desktop-entry registry + compositor IPC |
| Workspace indicators / switching | **running** | compositor adapter (hyprctl JSON today, glyphwm tomorrow) |
| IPC bus + `glyphctl` CLI | **working** | Unix socket, JSON lines |
| Design tokens → live CSS | **working** | `glyph-core` (dark/light, glass levels, live reload) |
| Doto (dot-matrix) font pipeline | **working** | OFL font auto-installed to `~/.local/share/fonts` |
| persistent config `glyphos.toml` | **working** | `~/.config/glyphos/glyphos.toml` |
| glyphwm (custom Smithay compositor) | **planned — phase 2/3** | see `compositor/` |
| live ISO / installer | **planned — phase 7** | see `dist/` |

## Quick start (development)

```sh
scripts/dev-run.sh            # builds + launches the shell on the current Wayland session
glyphctl ping                 # wants the socket: /run/user/<uid>/glyphos/shell.sock
glyphctl command              # toggle Glyph Command bar
glyphctl control              # toggle Control Center
glyphctl workspace 3          # jump to workspace 3 (compositor adapter)
glyphctl launch kitty         # desktop-entry aware launch
```

Requires: Rust, GTK4, gtk4-layer-shell (built locally into `~/.local` by
`scripts/setup-gtk4-layer-shell.sh` when pacman cannot be used).

## Repository layout

```
crates/
  glyph-core/     design tokens, config (glyphos.toml), IPC protocol
  glyph-system/   honest adapters: NetworkManager, BlueZ, PipeWire, backlight, battery
  glyph-shell/    the desktop shell (panel, dock, command bar, control center)
  glyphctl/       CLI client for the shell IPC bus
compositor/       glyphwm — the custom Wayland compositor (Smithay), phase 2+
assets/           fonts (Doto, Space Mono), wallpapers, icon sources
docs/             architecture, roadmap, keybindings, limitations
dist/             packaging, ISO tooling, session .desktop registration
```

## Configuration

Everything lives in `~/.config/glyphos/glyphos.toml` — see
`docs/ARCHITECTURE.md` for the token reference and `docs/CONFIGURATION.md`
for every option (dock pins, wallpaper, glass opacity, theme mode, fonts).

## Honesty markers

The docs mark every subsystem as **implemented / partial / planned**.
Nothing in this tree fakes a success it cannot demonstrate. Section 13 of
the architecture doc lists known limitations in plain language.
