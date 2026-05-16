# web-sw-cor24-x-assembler

Web UI for the [sw-cor24-x-assembler](https://github.com/sw-embed/sw-cor24-x-assembler) COR24 assembler. Assemble and run COR24 programs entirely in the browser using Rust, Yew, and WebAssembly.

**Live demo:** [sw-embed.github.io/web-sw-cor24-x-assembler](https://sw-embed.github.io/web-sw-cor24-x-assembler/)

Part of the [Software Wrighter COR24 Tools Project](https://sw-embed.github.io/web-sw-cor24-demos/#/).

## Status

**Working demo deployed.** Three-pane layout (assembly source on the left, listing in the middle, emulator I/O panel on the right) with a Load-demo dropdown of bundled examples from [sw-cor24-x-assembler](https://github.com/sw-embed/sw-cor24-x-assembler). LED, switch S2, UART, and register view live; press *Assemble & Run* to run a program. I2C-device panels (starting with TMP101 simulation, then RTC / LCD / additional sensors) are the next saga steps.

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
