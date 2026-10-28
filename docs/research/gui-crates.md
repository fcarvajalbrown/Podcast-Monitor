# GUI crate for the desktop window

Question: which open-source native Rust GUI crate builds the Spanish desktop window (phase 4 item 1).

Candidates named in the approved item: egui through `eframe` 0.37.0, `iced` 0.14.0, `slint` 1.18.1.

Method: `tools/gui_spike/` holds one crate per candidate (`egui/`, `iced/`, `slint/`) and a shared `common/` crate with the Spanish strings, the show list (Animales Políticos and Nada Personal with their publishers), the progress text and the Open PDF action (`open` 5.4.5 on `work/reports/animales-politicos-ep34-escritorio.pdf`). Each window has a selectable show list, Agregar programa and Quitar buttons that change the list, a progress bar at 45 % that Ejecutar episodio moves by 10 %, Abrir PDF, and a line with every Spanish accent, ¿ ¡ « » and curly quotes. Each crate is its own workspace with its own `Cargo.lock` and target folder. Run with `--screenshot <file.png>` the window captures itself through the crate's own screenshot call, writes the PNG (`png` 0.18.1, in all three) and closes.

Build: `tools/build/cargo-windows.cmd build --release --manifest-path tools/gui_spike/<crate>/Cargo.toml`, each with `CARGO_TARGET_DIR=%USERPROFILE%\.pmt-gui-<crate>`, one after the other on the same machine, crates already downloaded.

| | egui (eframe) | iced | Slint |
|---|---|---|---|
| Licence | MIT or Apache-2.0 | MIT | GPL-3.0-only, or Slint Royalty-free 2.0, or Slint Software 3.0 (commercial) |
| Release exe | 14.6 MB | 11.1 MB | 13.6 MB |
| Clean release build | 136 s | 107 s | 118 s |
| Rebuild after touching `main.rs` | 3 s | 7 s | 8 s |
| Packages in `Cargo.lock` | 430 | 396 | 554 |
| Default renderer | wgpu | wgpu, tiny-skia software fallback | FemtoVG (OpenGL), software renderer also built |
| Code for the test window (plus 54 shared lines) | 94 lines, immediate mode | 99 lines, messages and update | 109 lines, UI in the `slint!` markup language plus Rust callbacks |
| Built on first try | no: 0.37 replaced `App::update` with `App::ui` and changed the screenshot call | yes | yes |
| Selectable list | `selectable_label` per row | none built in; a column of buttons styled as selected or plain | `StandardListView` |
| Screenshot of its own window | `ViewportCommand::Screenshot` with a callback | `window::screenshot` task; taken at start-up it came out blank, so it waits for the third frame through `window::frames` | `Window::take_snapshot` |
| Font | its own bundled font | Windows system font | Windows system font |
| Spanish characters in the window | all correct | all correct | all correct |
| Look with no styling | flat grey, small text | blue filled buttons, larger text; the accents line wraps at 560 px | Windows 11 Fluent style, list leaves empty space below two rows |

Slint's royalty-free licence, from `LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` in the crate: it covers desktop, mobile and web applications, not embedded systems, and asks for the `AboutSlint` widget in an About screen reachable from the top menu (or the splash screen) and the Slint attribution badge on a public web page, preferably where the binaries are downloaded. The other two carry no such condition.

Size against the app: `podcast-monitor.exe` is 103.3 MB with the report built in; the window adds roughly the exe sizes above less the `open` and `png` crates, which are shared. Not yet measured as part of the app, and no macOS build measured.

Screenshots, 560 by 420 logical pixels at 125 % scaling: `gui-crates/egui.png`, `gui-crates/iced.png`, `gui-crates/slint.png`. The Abrir PDF button calls the same `open::that` in all three and was not clicked in this run.

Felipe's pick: pending.
