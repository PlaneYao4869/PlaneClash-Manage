# PlaneClash Manage

**A standalone rule manager for Clash-based proxy clients** (FlClash, Clash Verge, mihomo, and friends).

Built with **Tauri 2 + React + TypeScript** — a small (~10 MB) native desktop app
that scans your computer for Clash clients, lets you add/manage domain and
process rules, and writes them back to each client's `config.yaml` with
automatic backup.

> Note: this is a sibling of [PlaneClash](https://github.com/PlaneYao4869/PlaneClash) — a
> full FlClash fork that does everything (proxy core, TUN, UI for every setting).
> PlaneClash Manage is the **opposite** — it's a tiny tool that only does one
> thing: **manage rules**.

## Features (MVP roadmap)

- [x] **Step 1**: scan the computer for installed Clash clients and report
      their `config.yaml` location.
- [ ] **Step 2**: parse a selected `config.yaml` and display its `rules:`
      list (grouped by type).
- [ ] **Step 3**: add a domain rule and save (auto-backup, write back).
- [ ] **Step 4**: add a process rule and save.
- [ ] **Step 5**: IP-CIDR, RULE-SET, MATCH (fallback) types.
- [ ] **Step 6**: per-client format adapters (mihomo yaml, Clash Verge, etc.).

## Why Tauri 2?

CC-Switch ([farion1231/cc-switch](https://github.com/farion1231/cc-switch))
pioneered this kind of "small desktop tool for power users" UX with Tauri 2.
We follow the same style: Web frontend (React) + Rust backend, single 10 MB
executable, native window chrome.

## Building

You don't need a local Rust toolchain — CI builds for all platforms.

### Local dev (optional)

```bash
# Install Rust (https://rustup.rs) and Node 20+
npm install
npm run tauri:dev
```

### Build installers

Push a tag like `v0.1.0`; the GitHub Actions `release.yml` workflow will
build Windows MSI, Linux deb, and macOS dmg, attach them to a draft release.

## License

MIT — see [LICENSE](LICENSE).
