# Tablo Demo — Agent Instructions

This repository shows what [Tablo](https://github.com/y-l-g/tablo) does, through Tablo's public
API alone. When a feature needs something Tablo lacks, record it in [`GAPS.md`](GAPS.md) with what
blocks it, rather than working around Tablo.

## Layout

- Each business domain (`shop`, `blog`, `hr`, `projects`) is a module that owns its models,
  resources and dashboard. `access.rs` says what each staff role may do, `panels/` mounts
  everything, and `seed/` fills the in-memory database on every start.
- Every user-facing string is a key in both `locales/en.ftl` and `locales/fr.ftl`, under the same
  `##` section.
- Icons come from the Lucide subset in `assets/iconify/`; an icon outside it fails the build.

## Dependencies

- Tablo comes from its `master` branch, at the commit `Cargo.lock` records. Move it with
  `cargo update -p tablo -p tablo-build`.
- Topcoat is pinned in `[patch.crates-io]` to the commit Tablo builds against. Change it only
  together with Tablo.

## Checks

The demo has no tests, and CI only publishes the image, so run these yourself:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
topcoat asset bundle && cargo run
```

The panels refuse unsafe declarations when they mount, so a change is checked only once the app
starts. Then open `http://127.0.0.1:3000/`, sign in with a demo account, and try the change.
Never pipe a command whose exit code you need. Report what you ran and what you did not.

## Code and prose

- Field access keeps employees' pay hidden from roles without it. Widgets read records through
  `scoped_query` or `list_query`, and a page that reads rows directly gates itself with `access`.
- A comment explains why, never what the next line does.
- Keep the README's "What it covers" and `GAPS.md` current with each feature added or gap closed.

## Commits and pull requests

- Commits use Conventional Commits without a scope, e.g. `feat: tint rejected expenses`.
- One feature per commit: pull requests are merged with a merge commit, so each commit stays in
  `main`.
- A PR description says what changed and which checks ran.
