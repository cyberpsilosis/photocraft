# Working in photocraft

This file is a guide. It says how to do each kind of change here well and
what the separate reviewer checks. Every kind of change is allowed. When
Elijah's own words in a brief or an ask differ from this file, his words win.
`AGENTS.md` is the project's own guide and applies here too: read it first,
then `docs/`. This file adds what is particular to Elijah's copy.

## What it is

PhotoCraft is an open-source, clean-room reimplementation of Photoshop in
Rust (README): a native desktop app (egui and eframe on wgpu), a headless
command line, `photocraft-cli` (`convert`, `info`, `run`, `batch`,
`commands`, `mcp`), and the same app compiled to WebAssembly. It is a Cargo
workspace: 24 crates in `crates/`, three apps in `apps/` and `xtask/`.
`docs/` and the book in `book/` describe it.

This repository, cyberpsilosis/photocraft, is Elijah's public fork of
storytold/photocraft, made by the ArtCraft team. Its `main` is still the
commit it was forked at, d836e34 (2026-10-09). Elijah calls it his Photoshop
clone. AETHERNET's graphic designer uses it: it builds the command line from
the checkout beside aethernet-ops (`cargo build --release -p photocraft-cli`)
and runs its commands to make cut-outs and stills from Elijah's photos and the
Design Syndrome assets (aethernet-ops `docs/doctrine/agents/graphic-designer.md`).

What is live: nothing is hosted from this fork. It has no releases, no GitHub
Pages and no `release` branch. The installers, the GitHub releases and
getartcraft.com that the README names belong to the upstream. A merge to
`main` reaches the harness at the next job that builds from its checkout.

## How to run it

CI is `.github/workflows/ci.yml`, written for GitHub's hosted runners:
`ubuntu-latest`, plus `macos-latest` and `windows-latest` on `main` and on a
pull request that touches `apps/`, `ui-egui`, `text`, `gpu`, `tablet`,
`heif`, a `build.rs` or `Cargo.lock`. **GitHub runs none of it on this fork.**
Its Actions list no workflows and no runs (checked 2026-11-21), and pull
request #1 shows no checks. So the author runs the Linux job's steps and
pastes their output in the pull request, and the reviewer runs them again on
the head commit. Rust stable 1.95 or newer (`rust-version` in `Cargo.toml`).

    sudo apt-get install -y --no-install-recommends libxkbcommon-dev libwayland-dev \
      libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev
    export CARGO_PROFILE_DEV_DEBUG=line-tables-only    # as CI sets it
    cargo fmt --all -- --check
    cargo test --workspace --locked --features heif
    cargo test -p photocraft-heif -p photocraft-codecs -p photocraft-io --locked
    cargo clippy --workspace --all-targets --locked --features heif -- -D warnings
    cargo xtask layers
    cargo xtask scorecard --check
    rustup target add wasm32-unknown-unknown && cargo xtask wasm

- **The corpus job.** `cargo xtask corpus --all` fetches the pinned real-file
  corpora into the gitignored `corpus/` and checks their sha256
  (`xtask/src/corpus_pins.rs`); `cargo xtask test-corpus` runs the tests
  behind the `corpus` feature. CI runs it with
  `CARGO_PROFILE_RELEASE_LTO=false` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`.
  On a harness machine (2026-11-21) the fetch got PngSuite and then stopped:
  `codeload.github.com` answered 403 for the psd-tools tarball. Where it
  cannot fetch, say so in the pull request.
- **The craft-fonts pass.** CI's Linux job runs the workspace tests once more
  with storytold/craft-fonts checked out at `abb83316` into `craft-fonts/`,
  `CRAFT_FONTS_DIR` pointing at it and `CRAFT_FONTS_REQUIRED=1`.
- **Clippy on main.** With Rust 1.97.0, clippy stops on `main` (d836e34) at
  two `manual_range_contains` hits in `photocraft-engine`'s test code
  (`crates/engine/src/fill_cmds.rs:416`, `crates/engine/src/image_cmds.rs:625`)
  and checks nothing after them. Run it on `main` and on the branch with the
  same toolchain and compare: the branch adds no new hit.
- `cargo xtask ci` runs fmt, clippy, the tests, layers and wasm in one go and
  stops at the first failure. `AGENTS.md` section 5 lists the extra checks by
  kind of change (`parity`, `panic_hunt`, `perf`, `scorecard`).
- **Disk.** On a harness machine (2026-11-21) the workspace test build with
  CI's `line-tables-only` debug info and Cargo's incremental cache passed
  27 GB and filled the disk. With `CARGO_PROFILE_DEV_DEBUG=0
  CARGO_INCREMENTAL=0` the target dir stayed at 6.2 GB and the tests passed in
  12.5 minutes on 4 cores. Say in the pull request which settings you ran.
  Parallel agents each use their own `CARGO_TARGET_DIR` (AGENTS.md section 6).

## How changes ship

- `main` is the only main line. Start each change from a fresh `origin/main`,
  give it one branch and one pull request against `main` of
  cyberpsilosis/photocraft (not the upstream storytold/photocraft), and merge
  `main` into the branch again, then re-run the steps above, before asking for
  review.
- A separate reviewer reads the whole diff against this file and ends its
  comment with `VERDICT: PASS` or `VERDICT: FAIL`, after the extra checks in
  the table below. A passed review puts the pull request in front of Elijah.
- Elijah merges, by hand. No agent merges a pull request here.
- Say in the pull request what the harness will see change: which commands,
  renders or file outputs differ, and that only Linux was run (GitHub runs no
  macOS or Windows job here).

## Hard constraints

Each says how to meet it.

1. **No secret in the repository or its history.** The secrets this code
   knows are the control channel's bearer token and the release signing
   keys. The token is 64 hex characters, made by `generate_token`
   (`crates/automation/src/security.rs`), and is passed as a file with
   `--control-token-file` or in `PHOTOCRAFT_CONTROL_TOKEN`
   (`docs/development.md`). `.gitignore` keeps `/secrets` out but not
   `.private/`, the folder the example in `docs/development.md` uses, so keep a
   token file under `secrets/` or outside the checkout. The Apple, Windows and
   Azure signing secrets live in a GitHub `release` environment, read only by
   `release.yml` on the `release` branch (`docs/releasing.md`, Secrets). Tests
   make their own tokens. A request that fetches anything sends a generic
   `Photocraft-dev` User-Agent and no one's name or email (AGENTS.md rule 6).
2. **People's photos stay out of git.** The harness opens Elijah's and his
   clients' photos in PhotoCraft. None of them goes into a commit, a test, a
   doc image or an issue. Demo and README images are public-domain art with
   their sources in `docs/images/SOURCES.md` (AGENTS.md rule 6); test files
   are synthetic or minimized and hold no personal data (`SECURITY.md`); real
   files stay in the gitignored `corpus/` and new oracles go to
   storytold/photocraft-corpus; fonts stay in storytold/craft-fonts.
3. **The automation channel checks who is asking.** There are no accounts
   and no sign-in. The one door is automation. The desktop app's
   `--control <port>` server binds to 127.0.0.1 only
   (`apps/photocraft/src/control_server.rs`), the headless TCP server refuses
   any address that is not loopback (`serve_tcp` in
   `crates/automation/src/rpc.rs`), and both dispatch nothing until
   the first request, `auth`, carries the token (`authentication_reply` and
   `token_matches` in `crates/automation/src/security.rs`). The MCP bridge to
   the running app accepts only loopback addresses and sends the same token
   (`crates/automation/src/bridge.rs`). Automation reads and writes files
   only inside the launch-time roots `--automation-read-root` and
   `--automation-write-root`, and fails closed without them (`SECURITY.md`,
   Current limitations). A new method or transport keeps all of this and adds
   a test like `authentication_does_not_dispatch_without_the_token`.
4. **Never crash.** A panic loses the document someone is working on
   (AGENTS.md, Never crash). Non-test code returns errors and never uses
   `unwrap`, `expect`, `panic!` or unchecked indexing on numbers that come
   from input; `clippy.toml` allows those in tests only. `unsafe` is
   forbidden outside `crates/tablet/src/macos.rs`. A crash fix comes with a
   test that panicked before it, and a new or changed command passes
   `cargo test -p photocraft-engine --test panic_hunt -- --ignored`.
5. **Clean-room and licences.** Code is MIT OR Apache-2.0. Nothing is copied
   from Photoshop or other proprietary software (code, shaders, profiles,
   icons); behaviour comes from public specs and observation. A third-party
   or original asset keeps its licence file beside it and gets a row in
   `ATTRIBUTION.md` in the same change. The ArtCraft logos in `docs/brand/`
   are not open source, and the README (License and credits) says forks and
   modified versions must remove them, so a build or page this fork publishes
   leaves them out.

No code here takes a payment, sends email or keeps a list. The README calls
the app offline, and its only network listeners are the two loopback servers
above.

## Conventions

- **Language.** Rust only, edition 2024. No JavaScript or TypeScript, and no
  Tauri, Electron or webview shell (AGENTS.md).
- **Style.** `cargo fmt` with `rustfmt.toml` (`max_width = 160`), `cargo
  clippy -D warnings`, comments that say why, and the surrounding code's
  style. The product is written PhotoCraft in anything a person reads; crate,
  binary and file names stay lowercase.
- **Layers.** Crates sit in layers L0 to L6 and depend only downward,
  enforced by `cargo xtask layers`; a new crate is registered in
  `xtask/src/layers.rs`. Nothing below `ui-egui` uses egui, eframe, winit or
  rfd, and L0 to L6 must build for `wasm32-unknown-unknown`.
- **Everything is a command.** User-visible behaviour is an engine command in
  `crates/engine/src/<area>_cmds.rs` with `specs()`, registered in
  `commands.rs`, using the id from `crates/ui-egui/src/menu_catalog.rs`
  (`docs/contributing.md`, Adding a command). Bit depth and colour model are
  runtime data; colour goes through `photocraft-cms`.
- **Tests.** Unit tests in `#[cfg(test)]` modules beside the code, integration
  tests in `crates/*/tests/` and `apps/*/tests/`, helpers in `crates/testkit`,
  proptest for tiles, regions and codecs. Pixel code is tested at 8, 16 and
  32 bits. `panic_hunt` is `#[ignore]` and runs with `--ignored`.
- **Outside this repository.** AGENTS.md points to `../craftrules`, which is
  not public and not checked out here, and to `log/devlog.md` and `plan/`,
  which `.gitignore` keeps local. The pull request body is the record of what
  landed, its numbers and what is still open.

## How to check each kind of change

**On every row** the reviewer runs the steps under "How to run it" on the
head commit (the corpus job and the craft-fonts pass where the row says) and
pastes the test counts, greps the diff for tokens, keys, email addresses,
phone numbers and photos of people, and confirms the pull request targets
`main` of cyberpsilosis/photocraft.

| Kind of change | Doing it well here | The reviewer also checks, before VERDICT: PASS |
| --- | --- | --- |
| Copy and text (UI labels, messages, docs, the book) | UI strings are English keys translated in `crates/ui-egui/src/i18n/*.tsv` (docs/localization.md); a new label gets its catalog lines. Docs say what the code does today | The `photocraft-ui-egui` tests pass (they enforce catalog coverage); a snapshot of each changed screen; every command and flag named exists (`photocraft-cli commands`, `--help`) |
| A page's look | There are no web pages. What people look at is the app's window: colours and radii from `theme::Tokens`, widgets from `widgets::*` (docs/ui-design.md) | Before and after snapshots, opened at full size: `cargo run --release -p photocraft-ui-egui --example snapshot -- --out ui.png --size 1440x900 --scale 2 --open <public-domain image>`; a change to the web shell also built with `trunk build --release` in `apps/photocraft-web` and opened in a browser |
| Payments or checkout | None. Nothing here sells or charges; a change that adds it names its provider and where its keys live | The provider and the keys' home named in the pull request and absent from the diff |
| Sign-in, admin or door access | No accounts. The door is the automation channel in hard constraint 3 | A test per new or changed method or transport: nothing answered before `auth` with the token, and a path outside its root refused; `cargo test -p photocraft-automation -p photocraft` green; `SECURITY.md` and `book/src/automation/security-model.md` still match the code |
| Email to the lists | None: no email, no list, no account | Nothing in the diff sends mail; a change that adds it names the provider and where its key lives |
| Contracts or anything on chain | Nothing on chain | The keys' home named in the pull request and absent from the diff |
| Commands and tools (`crates/engine`, `menu_catalog.rs`) | The "Adding a command" checklist: params doc, `enabled`, one history step, bad-param tests, several depths | A test that fails without the change; `panic_hunt -- --ignored` passes; after new commands, `cargo xtask parity` run, `docs/parity.md` committed and `FLOOR` in `crates/ui-egui/src/parity.rs` not lowered |
| Files and formats (`psd`, `codecs`, `io`, `format`, `heif`) | Round-trip, generator and malformed-input tests; unknown PSD blocks kept verbatim; a new document field added to `crates/format/src/manifest.rs` and `convert.rs` with `#[serde(default)]` (AGENTS.md section 8) | The corpus job's output (`cargo xtask test-corpus`), no corpus result or floor lowered |
| Rendering and effects (`compose`, `gpu`, `algo`) | The CPU compositor is the oracle and the GPU path matches it; work per tile, cache per revision | A test that fails without the change; a public-domain image rendered with `PHOTOCRAFT_CPU_COMPOSE=1` and without, opened side by side; the corpus job; `cargo xtask perf --quick` before and after on a hot path |
| Dependencies and config | `Cargo.lock` updated and used with `--locked`; no JavaScript, Tauri or webview; why the crate is needed and its licence in the pull request (book/src/development/dependencies.md) | All steps above, including `cargo xtask wasm` and the craft-fonts pass; that only Linux ran is stated |
| Taking newer upstream commits | Merge storytold/photocraft's `main` into a branch, never a rebase, and keep Elijah's commits | Every step above plus the corpus job; each of Elijah's commits still in the tree |
