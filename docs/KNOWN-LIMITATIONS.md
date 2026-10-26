# Known limitations (honest list)

1. **glyphwm does not exist yet.** The desktop currently runs on the
   documented development-fallback compositor backend (Hyprland). Window
   tiling/snapping behavior is therefore the fallback's, not GlyphOS's.
2. **Notifications surface is built but no daemon yet.** The D-Bus
   notification daemon lands in phase 5; the surface is real, the events
   are not flowing yet.
3. **Lock screen is a labeled placeholder.**it does not pretend to lock;
   it prints a clear diagnostic until the session controller exists.
4. **Dock running-indicators** read real windows from the compositor cache
   but only refresh every 2 s (fine at 60 fps? no; acceptable for now).
5. **Glass blur** on the dev compositor relies on its blur pipeline;
   `reduced`/`opaque` modes are implemented CSS-side and always available.
6. **Wallpaper management** is not in the shell yet (compositor-owned).
7. **NDot** is Nothing's proprietary face; GlyphOS bundles Doto (OFL) and
   supports pointing `theme.display_font` at a licensed NDot path.
8. **Audio per-app mixing** shows global/endpoint state; per-stream UI
   arrives with the Sound settings page.
9. The shell currently expects English locale strings; i18n comes later.

Verified on: Arch Linux · Hyprland 0.56 · Wayland · Intel iGPU.
