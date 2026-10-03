#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE="$ROOT/profile"

required=(
  "$PROFILE/profiledef.sh"
  "$PROFILE/packages.x86_64"
  "$PROFILE/pacman.conf"
  "$PROFILE/grub/grub.cfg"
  "$PROFILE/grub/loopback.cfg"
  "$PROFILE/syslinux/archiso_sys-linux.cfg"
  "$PROFILE/efiboot/loader/loader.conf"
  "$PROFILE/efiboot/loader/entries/01-glyphos-linux.conf"
)

for file in "${required[@]}"; do
  [[ -f "$file" ]] || { echo "Missing required file: $file" >&2; exit 1; }
done

bash -n "$PROFILE/profiledef.sh"
bash -n "$PROFILE/airootfs/usr/local/libexec/glyphos-live-user"

grep -q '^mkinitcpio-archiso$' "$PROFILE/packages.x86_64"
grep -q 'archisobasedir=%INSTALL_DIR%' "$PROFILE/grub/grub.cfg"
grep -q 'archisobasedir=%INSTALL_DIR%' "$PROFILE/syslinux/archiso_sys-linux.cfg"
grep -q 'archisobasedir=%INSTALL_DIR%' "$PROFILE/efiboot/loader/entries/01-glyphos-linux.conf"

echo "GlyphOS profile validation passed."
