# Commercial workspace implementation plan

> Execute inline using executing-plans, test-driven-development and verification-before-completion. User approved the design and autonomous execution, including Git commits and GitHub pushes for each verified stage.

**Goal:** Complete the serial debugging workspace while preserving its visual style and allowing users to choose visible panels.

**Architecture:** Keep GPUI views; extract tested communication, parsing and preferences models. Each connection owns its cancellation state. UI state observes data without owning blocking device operations.

**Tech Stack:** Rust 2024, GPUI, gpui-component, Tokio, serialport, serde_json; existing locked dependencies.

## Global constraints

- Never delete multiple files or directories. Preserve the user's existing `info.rs` edits.
- Each verified stage gets a commit with `AI-Assisted: OpenAI Codex` and a push to `origin/feat/commercial-workspace`.
- No automatic connection or timer start when preferences are restored.
- Bound queues, text history, parser buffers and chart history. Hidden panels continue collecting data.
- Report hardware and graphical verification limits honestly.

## Stage 1: Communication correctness

Files: `crates/ww-protocol/src/{lib,transfer}.rs`, `crates/ww-ui/src/{model/port_model,ui/port_panel,ui/io_panel,ui/main_view}.rs`.

Interface: `transfer::write_bytes(writer: &mut impl Write, data: &[u8], cancelled: impl Fn() -> bool) -> TransferResult`, with `bytes_written: usize` and `error: Option<io::Error>`; `transfer::parse_hex(&str) -> Result<Vec<u8>, String>`.

- [ ] Add failing regression tests: a writer accepting 3 bytes each call must receive all 140 bytes exactly once; zero writes must return WriteZero; odd or invalid Hex must fail; cancelled writes must stop and preserve the exact successful count.
- [ ] Run `cargo test -p ww-protocol --offline`, observe regressions, implement the independent helpers and rerun.
- [ ] Use a background writer with connection generation checks; count successful bytes only and report failures. Use bounded read channels, per-connection cancellation and nonblocking close. Publish connected state only when both handles exist.
- [ ] Run `cargo check --workspace --offline` and targeted tests; inspect diff; commit and push.

## Stage 2: Preferences and configurable workspace

Files: `crates/ww-ui/src/{preferences,ui/main_view,ui/io_panel,ui/config_panel,ui/port_panel,ui_config}.rs`.

Interface: serialized `Preferences` contains `Layout`, connection parameters, encoding, font, timer and chart choices; `Layout::preset(Preset)` returns an independent visibility map. Global GPUI `PreferenceStore` owns path, settings and visible save/load errors.

- [ ] Add tests for preset visibility, all-hidden recovery, preference defaults, round-trip serialization, malformed settings and range validation; observe expected failing assertions before implementing.
- [ ] Store settings under local application data; save atomically through a temporary sibling file; reject unsupported versions. Keep original corrupt settings until the user changes settings.
- [ ] Build an always accessible View toolbar with presets and individual checkboxes; reflow configuration and work areas when hidden or narrow. Use explicit panel labels and nonzero minimum dimensions.
- [ ] Restore parameters and fonts without connecting; fix default config panic when no baud list exists.
- [ ] Verify tests and workspace check; commit and push.

## Stage 3: Complete debugging controls

Files: `crates/ww-ui/src/{ui/io_panel,ui/port_panel,model/port_model,export}.rs`, `crates/ww-protocol/src/model.rs`.

- [ ] Test interval validation (10–3600000 ms), raw Hex suffix append, exclusive export creation and flow-control parameter mapping.
- [ ] Add None/RTS-CTS/XON-XOFF selector and apply through serialport builder.
- [ ] Add editable timer interval and start/stop buttons; snapshot the payload, prevent overlap, cancel on disconnect/write error. Add clear/copy/export actions to log headers, reset statistics separately, persist log display options.
- [ ] Export via save-file dialog and exclusive file creation; an existing file requires explicit overwrite confirmation. Do disk writes in a background task and surface result.
- [ ] Verify tests and workspace check; commit and push.

## Stage 4: Incremental data analysis and waveforms

Files: `crates/ww-protocol/src/telemetry.rs`, `crates/ww-ui/src/ui/telemetry_panel.rs`, `crates/ww-ui/src/ui/main_view.rs`.

Interface: `TelemetryParser::new(Protocol)`, `push(&[u8]) -> Vec<Vec<f32>>`; limit 16 channels and 8192 buffered bytes. Chart history holds at most 2048 frames.

- [ ] Test split/coalesced CSV and JustFloat frames, malformed/oversized frames, nonfinite values and recovery. JustFloat tail is `00 00 80 7f`; floats are little endian. Verify protocol against primary documentation.
- [ ] Add protocol selector, custom GPUI canvas plot, channel legend and visibility, pause/resume, clear and CSV export. Keep parsing and caching active while hidden; pause freezes only displayed data. Throttle notifications to a maximum 30 Hz.
- [ ] Reset partial frames on connection changes and protocol changes. Show parser rejection count and buffer/history limits.
- [ ] Verify tests and workspace check; commit and push.

## Stage 5: Release validation and documentation

Files: `README.md`, `docs/COMMERCIAL_READINESS.md`, `.github/workflows/ci.yml`, this plan.

- [ ] Review all changed paths for lifecycle races, data loss, layout constraints and write errors.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace --offline`, `cargo check --workspace --offline` and a release build when available.
- [ ] Document usage, protocols, limits, troubleshooting, AI assistance and the user request; create Windows CI checks.
- [ ] Record actual validation evidence and a device acceptance checklist without claiming unperformed hardware tests passed.
- [ ] Commit and push the final verified stage, confirm upstream head equals the local head.
