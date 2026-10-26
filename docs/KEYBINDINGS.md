# Keybindings

Global bindings live with the compositor for now (panels never own input
except overlays). `dist/hyprland/glyphos-dev.conf` is a ready-made dev
config; the same commands work with any compositor that can spawn commands.

## The plan (targets for glyphwm)

| Binding | Action |
|---|---|
| `Super+Space` | Toggle Glyph Command |
| `Super` (tap) | App launcher / overview (configurable) |
| `Super+Enter` | Terminal |
| `Super+E` | Files |
| `Super+,` | Settings |
| `Super+Tab` | App switcher |
| `Super+Arrows` | Snap / tile |
| `Super+Shift+Arrows` | Move window between tiles/workspaces |
| `Super+1..9` | Switch workspace |
| `Super+Shift+1..9` | Move window to workspace N |
| `Print` | Screenshot UI |
| `Super+L` | Lock screen (phase 5) |
| `Super+Q` | Close focused window |

Today's working subset via glyphctl:

```sh
glyphctl command      # Super+Space
glyphctl control      # quick settings
glyphctl notify       # notifications
glyphctl workspace N  # Super+N behavior
glyphctl launch kitty # app launching
```

Full window-management bindings arrive with glyphwm (phase 3); they will be
defined in `~/.config/glyphos/keymap.toml` with conflict detection in
Settings.
