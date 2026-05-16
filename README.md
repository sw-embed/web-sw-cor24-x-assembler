# web-sw-cor24-x-assembler

Web UI for the [sw-cor24-x-assembler](https://github.com/sw-embed/sw-cor24-x-assembler) COR24 assembler. Assemble and run COR24 programs entirely in the browser using Rust, Yew, and WebAssembly.

**Live demo:** [sw-embed.github.io/web-sw-cor24-x-assembler](https://sw-embed.github.io/web-sw-cor24-x-assembler/)

Part of the [Software Wrighter COR24 Tools Project](https://sw-embed.github.io/web-sw-cor24-demos/#/).

## Status

**Initial scaffold deployed.** Pages pipeline (build + GH Actions) is wired up; the page boots a placeholder Yew app. Assembler-tab port from [cor24-rs](https://github.com/sw-embed/cor24-rs) (in deprecation in favor of the split `sw-cor24-emulator` + `sw-cor24-x-tinyc` + `sw-cor24-x-assembler` repos) is the next saga step, followed by I/O panels including I2C-device simulation (starting with TMP101).

## Related

- [sw-cor24-x-assembler](https://github.com/sw-embed/sw-cor24-x-assembler) — COR24 assembler library (the crate this UI wraps)
- [sw-cor24-emulator](https://github.com/sw-embed/sw-cor24-emulator) — COR24 emulator (execution backend)
- [sw-cor24-x-tinyc](https://github.com/sw-embed/sw-cor24-x-tinyc) — Tiny C compiler for COR24
- [web-sw-cor24-x-tinyc](https://github.com/sw-embed/web-sw-cor24-x-tinyc) — sibling web demo (closest scaffold template)
- [sw-cor24-project](https://github.com/sw-embed/sw-cor24-project) — COR24 ecosystem hub

## Development

```bash
./scripts/serve.sh              # dev server with hot reload on port 9102
./scripts/build-pages.sh        # release build, rsynced to pages/
cargo clippy -- -D warnings     # lint
```

Pushing to `main` deploys `pages/` to GitHub Pages via `.github/workflows/pages.yml`. Build `pages/` locally and commit it as part of any PR that changes the UI; the workflow does not re-build — it just uploads the tracked `pages/` tree.

## Copyright

Copyright (c) 2026 Michael A. Wright

## License

MIT
