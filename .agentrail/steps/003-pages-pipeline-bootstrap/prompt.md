Set up the pages-deploy infrastructure for web-sw-cor24-x-assembler,
modelled on the working pattern in
~/github/sw-embed/web-sw-cor24-basic. Mike has already configured
this repo's GitHub Pages settings to deploy via GH Actions.

Scope:

1. scripts/build-pages.sh -- Adapted from basic's build-pages.sh.
   Run `trunk build --release --public-url /web-sw-cor24-x-assembler/`,
   then `rsync -a --delete --exclude='.nojekyll' dist/ pages/`.
   Carry over the dist/-lock trick so a running `trunk serve` cannot
   race the build.
2. scripts/serve.sh -- Adapted from basic's serve.sh; port 9102.
   Same dist/ lock; passes `--port 9102` to `trunk serve`.
3. .github/workflows/pages.yml -- Same shape as basic: on push to
   main, upload `./pages` as the pages artifact and deploy via
   actions/deploy-pages@v4. `pages: write`, `id-token: write`,
   concurrency group "pages".
4. pages/.nojekyll committed once.
5. Seed pages/ from a release build of the current scaffold so the
   workflow has something to deploy on first run.
6. README.md: add a "Live demo" link near the top pointing at
   https://sw-embed.github.io/web-sw-cor24-x-assembler/. Drop the
   "Scaffold" status hedge accordingly (replace with "Initial
   scaffold deployed; assembler tab port is next").
7. .gitignore: keep pages/* tracked (the existing line
   `!pages/*.wasm` already handles the wasm; verify the rest of the
   directory is not excluded).

Exit criteria:
- `./scripts/build-pages.sh` runs clean from a clean tree and
  populates pages/ with index.html, .nojekyll, the wasm + JS bundle.
- `./scripts/serve.sh` starts trunk on port 9102.
- pages/.nojekyll present.
- .github/workflows/pages.yml in place; YAML parses (a syntax-only
  check is enough; the actual deploy verifies on push).
- README.md live-demo link present.
- One commit lands all of this plus the saga record.
