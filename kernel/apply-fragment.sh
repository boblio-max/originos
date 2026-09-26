#!/bin/sh
# Merge origin.cfg into a kernel source tree on a Debian build host.
# Usage: ./apply-fragment.sh /path/to/kernel-source
# The kernel's own merge_config.sh validates every option; read its
# output — a typo'd CONFIG_ name fails loudly, which is what you want.
set -e
SRC="${1:?usage: apply-fragment.sh /path/to/kernel-source}"
FRAG="$(dirname "$0")/config-fragment/origin.cfg"
cd "$SRC" && scripts/kconfig/merge_config.sh .config "$FRAG"
