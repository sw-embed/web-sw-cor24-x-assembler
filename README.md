# web-sw-cor24-x-assembler

Web UI for the [sw-cor24-x-assembler](https://github.com/sw-embed/sw-cor24-x-assembler) COR24 assembler. Assemble and run COR24 programs entirely in the browser using Rust, Yew, and WebAssembly.

Part of the [Software Wrighter COR24 Tools Project](https://sw-embed.github.io/web-sw-cor24-demos/#/).

## Status

**Scaffold.** This repo was just initialized. The plan is to extract the assembler live-demo from [cor24-rs](https://github.com/sw-embed/cor24-rs) (which is being phased out in favor of the split `sw-cor24-emulator` + `sw-cor24-x-tinyc` + `sw-cor24-x-assembler` repos) and host it here.

## Related

- [sw-cor24-x-assembler](https://github.com/sw-embed/sw-cor24-x-assembler) — COR24 assembler library (the crate this UI wraps)
- [sw-cor24-emulator](https://github.com/sw-embed/sw-cor24-emulator) — COR24 emulator (execution backend)
- [sw-cor24-x-tinyc](https://github.com/sw-embed/sw-cor24-x-tinyc) — Tiny C compiler for COR24
- [web-sw-cor24-x-tinyc](https://github.com/sw-embed/web-sw-cor24-x-tinyc) — sibling web demo (closest scaffold template)
- [sw-cor24-project](https://github.com/sw-embed/sw-cor24-project) — COR24 ecosystem hub

## Development

Build the bundled demo:

```bash
./scripts/build-pages.sh        # release build to pages/ (once it exists)
trunk serve                     # dev server with hot reload on port 9102
cargo clippy -- -D warnings     # lint
```

## Copyright

Copyright (c) 2026 Michael A. Wright

## License

MIT
