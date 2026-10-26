# Configuration — `~/.config/glyphos/glyphos.toml`

Created automatically with defaults on first launch of `glyph-shell`.
Every value below is honored live (live-appearance values reload the
stylesheet without restarting the shell).

```toml
[theme]
appearance = "dark"            # dark | light | auto
accent    = "#D71920"          # GlyphOS red; used sparingly by design
glass     = "frosted"          # frosted | reduced (no blur) | opaque (a11y)
glass_opacity = 82             # 0..100, surface tint strength
blur_quality  = 70             # 0..100 (compositor-dependent)
scale_percent = 100
animation_intensity = 80       # 0..100; 0 = reduced motion
corner_radius = 14
# User-supplied licensed NDot font path (from Nothing-adjacent licensing);
# when absent the OS falls back to Doto (OFL):
# display_font = "/home/me/.fonts/NDot-55-New.ttf"
ui_font   = "Inter"
mono_font = "Space Mono"

[dock]
pinned = ["glyphos-terminal", "glyphos-files", "glyphos-settings"]
position = "bottom"            # bottom | left | right
icon_size = 44
autohide = false

[apps]
# Optional custom entries surfaced in Glyph Command and the dock.
# { name = "Obsidian", exec = "obsidian --no-sandbox %F", description = "Notes", icon = "obsidian" }

# wallpaper = "/path/to/wall.png"   # honored by glyphwm once it owns the desktop
```

Notes
- `pinned` dock entries resolve through the desktop-entry registry
  (`/usr/share/applications`, `~/.local/share/applications`) before falling
  back to raw exec.
- All colors are hex strings; alpha form `#RRGGBBAA` is accepted.
- The config file is written atomically by Settings surfaces; hand edits
  while the shell runs are picked up on next theme reload.
