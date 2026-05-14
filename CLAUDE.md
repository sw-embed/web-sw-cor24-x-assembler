# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Web UI for the sw-cor24-x-assembler COR24 assembler. Live browser demos using Rust, Yew, and WebAssembly. Currently a scaffold — see [README.md](README.md) for status.

## Mission

Extract the assembler live-demo from [cor24-rs](https://github.com/sw-embed/cor24-rs) (now in deprecation as the COR24 ecosystem splits into single-purpose repos) into this repo. The closest reference is `cor24-rs/src/app.rs` (the multi-tab demo's "Assembler" tab) and `cor24-rs/src/assembler.rs` (the inline assembler) — but the inline assembler in cor24-rs is being replaced by the standalone `cor24-assembler` crate in `sw-cor24-x-assembler`, which this repo depends on via path.

## Related Projects (sibling clones needed for the path deps in Cargo.toml)

- `../sw-cor24-x-assembler` — COR24 assembler crate (the library this UI wraps)
- `../sw-cor24-emulator` — COR24 emulator (execution backend, no-std-compatible via `default-features = false`)
- `../sw-cor24-isa` — COR24 ISA definitions (transitive)

If those sibling clones don't exist in dwxas's working tree yet, ping mike — adding them is an infra step.

## Closest scaffold reference

[web-sw-cor24-x-tinyc](https://github.com/sw-embed/web-sw-cor24-x-tinyc) is the most directly analogous repo: same Yew + Trunk shape, same `dist/` → `pages/` deployment pattern, same compile-then-emulate pipeline structure (just swap `tc24r-*` for `cor24-assembler`). Read its `src/`, `scripts/`, and `pages/` deployment flow before starting.

Other useful conventions to skim:
- `sw-cor24-snobol4` and `sw-cor24-plsw` for the agentrail saga / step protocol — those repos have the most mature CLAUDE.md and have been through many saga cycles. Their workflow patterns generalize.

## Build

```bash
trunk serve                     # dev server with hot reload on port 9102
trunk build --release           # release bundle to dist/
cargo clippy -- -D warnings     # lint
```

Edition 2024. Never suppress warnings.

## Deployment (once pages/ is set up)

Live demo will be at https://sw-embed.github.io/web-sw-cor24-x-assembler/

Follow `web-sw-cor24-x-tinyc`'s pattern: `scripts/build-pages.sh` builds to `dist/` then rsyncs to `pages/` (tracked); `pages/.nojekyll` committed once; GitHub Actions deploys `pages/` on push to main.

## Workflow (devgroup)

This repo is hosted on a devgroup workstation. Run `onboarding` (in `$PATH`) for current state and helpers; full policy at `/disk1/github/softwarewrighter/devgroup/docs/branching-pr-strategy.md`.

- **Never push.** No `git push`, no `gh`. Signaling readiness is done by branch rename only — coordinator (mike) relays `pr/*` into `dev` and pushes.
- **Branches:** `feat/<slug>` for work in progress, based on `origin/dev` (not `origin/main`). Rename to `pr/<slug>` when ready to merge. `fix/<slug>` is the bug-fix flavor.
- **Helpers** (in `$PATH`): `dg-new-feature <slug>`, `dg-new-fix <slug>`, `dg-mark-pr`, `dg-list-pr`, `dg-reap`.
- **No history rewrites** on `dev` or `main`. Rebase is OK on your own `feat/*`.
- **After merge:** `git fetch origin --prune && git switch dev && git branch -D pr/<slug>`.
