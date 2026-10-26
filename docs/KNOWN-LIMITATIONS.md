# Known limitations (honest list)

1. **glyphwm does not exist yet.** The desktop currently runs on the
   documented development-fallback compositor backend (Hyprland). Window
   tiling/snapping behavior is therefore the fallback's, not GlyphOS's.
2. **Notification daemon gaps (daemon is real, features are partial):**
   capabilities advertise `body` only — no action buttons, no sounds, no
   `ActionInvoked` emissions, no body markup. The internal 6 s banner timeout
   does not emit `NotificationClosed(id, 1)` (expiry) on the bus; closes via
   user click or `CloseNotification` do emit (reason 2/3, bus-verified).
   `Notify`'s `expire_timeout` hint is accepted but not honored precisely.
   Takeover of the well-known name follows D-Bus rules: an existing owner
   that did not allow replacement (e.g. a running quickshell) keeps the
   name; glyph-notifd logs this and stays unclaimed until that daemon exits.
3. **Lock screen is a labeled placeholder**; it does not pretend to lock;
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
