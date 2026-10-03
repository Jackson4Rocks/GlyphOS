# GlyphOS Architecture

GlyphOS deliberately uses KDE Plasma as its desktop foundation instead of implementing a desktop environment from scratch.

```
GlyphOS
│
├── Arch Linux
│   ├── Linux
│   ├── systemd
│   ├── pacman
│   └── hardware/userspace infrastructure
│
├── KDE Plasma
│   ├── KWin
│   ├── Plasma Shell
│   ├── System Settings
│   ├── Power management
│   ├── Network integration
│   └── notifications/widgets
│
└── GlyphOS customization
    ├── theme and colors
    ├── Plasma layout
    ├── launcher
    ├── panel
    ├── lock/login experience
    ├── widgets
    └── GlyphOS-specific applications
```

## Why Plasma?

Plasma already solves difficult desktop infrastructure problems: window management, multi-monitor handling, session management, notifications, settings modules, accessibility, power management, and Wayland integration.

GlyphOS can therefore spend development effort on the user experience instead of rebuilding the entire desktop stack.

## Long-term goal

The end state is a desktop that users experience as **GlyphOS**, while KDE remains an implementation foundation underneath it.

The project should progressively replace stock Plasma surfaces instead of accumulating an increasingly large collection of unrelated theme overrides.
