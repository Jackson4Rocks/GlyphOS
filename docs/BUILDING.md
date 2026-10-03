# Building GlyphOS

GlyphOS currently targets x86_64 Arch Linux systems.

## Host requirements

```bash
sudo pacman -S --needed archiso git qemu-desktop edk2-ovmf
```

## Build

From the repository root:

```bash
make build
```

Equivalent command:

```bash
mkarchiso -v -w work -o out profile
```

The generated ISO is placed in `out/`.

## Clean build

```bash
make clean
make build
```

## Test in QEMU

```bash
make run
```

The test target uses `run_archiso` when available.

## Validation

```bash
make validate
```

Validation checks the profile structure, shell syntax, required package file, and key boot configuration placeholders.

Do not hand-edit generated content under `work/` or `out/`.
