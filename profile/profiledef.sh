#!/usr/bin/env bash
# shellcheck disable=SC2034

iso_name="glyphos"
iso_label="GLYPH_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="GlyphOS Project <https://github.com/Jackson4Rocks/GlyphOS>"
iso_application="GlyphOS Live Environment"
iso_version="$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)"

install_dir="glyph"
arch="x86_64"

pacman_conf="pacman.conf"
airootfs_image_type="squashfs"

bootmodes=('bios.syslinux'
           'uefi.grub')

airootfs_image_tool_options=(
    '-comp' 'xz'
    '-Xbcj' 'x86'
    '-b' '1M'
    '-Xdict-size' '1M'
)

file_permissions=(
    ["/etc/sudoers.d/glyphos-live"]="0:0:440"
    ["/usr/local/libexec/glyphos-live-user"]="0:0:755"
)
