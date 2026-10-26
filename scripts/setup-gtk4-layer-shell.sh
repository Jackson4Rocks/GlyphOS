#!/usr/bin/env bash
# Builds gtk4-layer-shell into ~/.local (only needed where the system package
# is unavailable, e.g. this dev machine). Requires meson+ninja (pip fallback).
set -euo pipefail
cd "$(dirname "$0")/.."

PAD=/tmp/opencode/gtk4ls
if [ ! -d "$PAD" ]; then
    mkdir -p /tmp/opencode
    git clone --depth 1 --branch v1.1.1 https://github.com/wmww/gtk4-layer-shell.git "$PAD"
fi
command -v meson >/dev/null || python3 -m pip install --user --break-system-packages meson ninja
export PATH="$HOME/.local/bin:$PATH"
meson setup "$PAD/build" --buildtype=release -Dtests=false -Dexamples=false \
      -Dintrospection=false -Dvapi=false --prefix=$HOME/.local --reconfigure 2>/dev/null \
  || meson setup "$PAD/build" --buildtype=release -Dtests=false -Dexamples=false \
      -Dintrospection=false -Dvapi=false --prefix=$HOME/.local
meson compile -C "$PAD/build"
meson install -C "$PAD/build"
echo "==> gtk4-layer-shell installed to ~/.local (pkg-config + lib present)"
