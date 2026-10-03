# GlyphOS

> **A Nothing-inspired Linux desktop, built on KDE Plasma.**

GlyphOS is an independent Arch-based Linux desktop project focused on a minimal, expressive, monochrome-first experience. The goal is to heavily customize KDE Plasma into a cohesive desktop workspace rather than building a desktop environment from scratch.

## Status

**Phase: Foundation**

The repository currently contains the first ArchISO profile, live Plasma setup, initial GlyphOS visual system, and build/validation tooling. The Plasma shell itself is still close to stock KDE; deeper workspace changes come next.

## Stack

- Arch Linux
- KDE Plasma
- Wayland / KWin
- SDDM
- PipeWire
- NetworkManager
- ArchISO

## Design direction

GlyphOS takes inspiration from industrial consumer-tech interfaces: strong typography, deep black surfaces, restrained monochrome UI, geometric details, large expressive information blocks, and subtle motion.

GlyphOS is **not an official Nothing product** and does not ship Nothing proprietary artwork, software, fonts, logos, or other protected assets.

## Repository layout

```
.
├── docs/
├── profile/
├── scripts/
├── Makefile
└── README.md
```

## Build

On an Arch Linux host:

```bash
sudo pacman -S --needed archiso git
make build
```

The ISO is written to `out/`.

For a local QEMU test:

```bash
make run
```

See [docs/BUILDING.md](docs/BUILDING.md) for the full workflow.

## Roadmap

- [x] Establish project identity and architecture
- [x] Create the first ArchISO foundation
- [x] Boot into a live KDE Plasma session
- [x] Establish GlyphOS dark color system
- [ ] Create the GlyphOS Plasma shell layout
- [ ] Replace the stock panel/launcher experience
- [ ] Add GlyphOS quick settings and notification surfaces
- [ ] Create the GlyphOS lock screen and login experience
- [ ] Build a first-party GlyphOS settings layer
- [ ] Create a polished installer
- [ ] Add hardware-aware defaults
- [ ] Produce reproducible release ISOs

## License

GlyphOS is under active development. License terms for original GlyphOS code and assets will be added before the first public release.
