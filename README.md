# OriginOS

A Debian-based distribution. The distro rule: **Debian is the foundation,
Origin is the shell layer.** We never rewrite what the host already does.

```text
Debian trixie (kernel, drivers, packages — untouched)
        │
Origin layer (ours: desktop, keybinds, shell config, kernel tweaks)
        │
       you
```

## Layout

- `debian-live/` — the ISO recipe: package list, installer defaults,
  first-boot files, build hooks. Edit here, build on Debian.
- `packages/origin-desktop/` — metapackage skeleton: the shopping list
  that pulls your whole Origin layer in one `apt install`.
- `desktop/` — YOUR desktop + the keybind schema contract.
- `kernel/` — kernel config *fragment* (overrides only) + merge script.
- `legacy/` — frozen reference app. Read it, never patch it.
- `PLAN.md` — local session plan (gitignored, not committed).

## Build (on a Debian host — VM, bare metal, or container)

```sh
cd debian-live
sudo ./auto/config   # or: sudo lb config (reads auto/config)
sudo ./auto/build    # produces originos-*.hybrid.iso
```

Your own packages first:

```sh
cd packages/origin-desktop
dpkg-buildpackage -b -us -uc
cp ../origin-desktop_*.deb ../../debian-live/config/packages/
```

Kernel tweaks (research each option before enabling):

```sh
./kernel/apply-fragment.sh /path/to/kernel-source
```

## On Windows

Edit here. Build there. This machine is the workshop, not the factory —
`lb build` needs root on Debian, so use a VM or Debian container.

## First three jobs (your parts)

1. `desktop/` — write the desktop against the keybind contract.
2. `desktop/keybinds.example.toml` — finalize the schema, then honor it.
3. `kernel/config-fragment/origin.cfg` — uncomment overrides one at a time.
