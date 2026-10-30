# HANDOFF (capant lane)

State 2026-10-30: R0-R5 done; 0.0.16 published and tagged. PR "supported versions + KISS verification" (0.0.17) in flight from branch `kiss-verify`, worktree `C:\Projects\capant-wt-kiss-verify`; remove the worktree after merge. The PM publishes 0.0.17 and tags.

## Next task
After merge: nothing queued. Ideas for rows if the PM wants them: none. Open thread: PersistAnt lane's `Evaluation::explain`-based fn (answered A, additive).

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`).
- CHANGELOG.md, Cargo.*, README.md, ci.yml are CRLF: scripted edits keep the file's newline; read the result back. In Rust test source never put a literal CR in a string.
- `cargo test --test ci_check` alone does not rebuild the example; use `cargo test`.
- Untracked `capant-seq.sh` and `supertool` in the primary checkout are not mine.
