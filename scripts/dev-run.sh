#!/usr/bin/env bash
# GlyphOS dev runner — build + launch the live shell on the current session.
# Safe to re-run: single-instance guarded over the IPC socket.
set -euo pipefail
cd "$(dirname "$0")/.."

export PKG_CONFIG_PATH="${HOME}/.local/lib/pkgconfig${PKG_CONFIG_PATH:+:${PKG_CONFIG_PATH}}"
export LD_LIBRARY_PATH="${HOME}/.local/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"

echo "==> building glyph-shell + glyphctl"
cargo build

if [ -e "/run/user/$(id -u)/glyphos/shell.sock" ] && target/debug/glyphctl ping >/dev/null 2>&1; then
    echo "==> glyph-shell already running; use glyphctl to control it."
    exit 0
fi

echo "==> launching glyph-shell"
nohup target/debug/glyph-shell > /tmp/glyph-shell.log 2>&1 &
echo "==> running (pid $!). Log: /tmp/glyph-shell.log"
