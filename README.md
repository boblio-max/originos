# OriginOS

what if there was a Linux distro where the Origin language *is* the shell layer? that's OriginOS. and before you ask — no, I'm not writing a kernel. the distro rule is: **Debian is the foundation, Origin is the shell layer.** never rewrite what the host already does.

## how it's laid out

- `debian-live/auto` + `config` — `live-build` recipe that cooks the ISO on top of Debian trixie
- `packages/origin-desktop/` — metapackage (Debian packaging via `dpkg-buildpackage`) pulling in the desktop layer
- `desktop/` — keybind/TOML contract for the Origin-driven desktop experience
- `kernel/` — config fragment (tweaks, not a fork)
- `legacy/` — frozen Rust app carried over from earlier experiments

```bash
# needs a Debian host (Windows = edit-only)
lb config && lb build
```

## status

live-build skeleton + architecture, no ISO yet. aspirational but grounded — every piece is a config/recipe, not a rewrite. first-3-jobs roadmap in the original docs.
