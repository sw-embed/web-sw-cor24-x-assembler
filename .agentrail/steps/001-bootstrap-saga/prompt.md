Bootstrap the agentrail-tracked workflow for web-sw-cor24-x-assembler:

1. Run `agentrail gen-agents-doc` and rename `AGENTS.example.md` to `AGENTS.md`.
2. Consolidate AGENTS.md and CLAUDE.md so they contain the same information
   (project section + mission + build/deploy + devgroup workflow + the
   agentrail process). Make CLAUDE.md a symlink to AGENTS.md.
3. Stage sibling clones of `sw-cor24-x-assembler` and `sw-cor24-emulator`
   under `../` (relative to this repo's worktree) so the path-deps in
   Cargo.toml resolve. `sw-cor24-isa` is transitive but expected by the
   emulator's path-dep, so confirm it exists too.

Exit criteria:
- `AGENTS.md` exists, `CLAUDE.md -> AGENTS.md` symlink in place.
- Sibling repos `../sw-cor24-x-assembler` and `../sw-cor24-emulator`
  cloned and at a sensible base commit.
- `agentrail status` shows the saga active with this step in-progress.
- Commit includes AGENTS.md, CLAUDE.md symlink, `.agentrail/*` artifacts.
