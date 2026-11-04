# Chat 12: release 0.1.0

## Done

- Phase 9 item 3: `CHANGELOG.md` (Spanish, 0.1.0, Añadido), linked from the README; Inno Setup installer `tools/build/installer.iss` built by `tools/build/release-windows.sh`; tag v0.1.0.
- Installer carries Microsoft's vc_redist.x64.exe (run only when the runtime is missing, Felipe), vulkan-1.dll built from Khronos's Vulkan-Loader tag vulkan-sdk-1.4.363.0 by `tools/build/vulkan-loader.cmd` (Felipe), the icon embedded in the exe (winresource) and set on the window (Felipe).
- Installed and uninstalled on this PC; title "Podcast-Monitor 0.1.0" and the logo in title bar and taskbar seen by Felipe.
- CPU fallback checked with no Vulkan driver visible; Ep. 34 on the CPU in 4,295 s (1.1x real time). Integrated-GPU speed estimated at 18 to 24 min for Ep. 34 on a 680M or Arc 155H class PC (`docs/research/igpu-speed-estimate.md`), not measured.
- Felipe: readers mostly have integrated graphics; 0.1.0 goes to beta testers, and without use in about two months the project is released as open source and closed.
- Cuña-Cutter session notes: the party proposal was sent by Felipe.

## Flags

- Opened: VC++ runtime step not seen on a PC without it; integrated-GPU timing needs a tester's PC; whether the page shows the CHANGELOG, set in item 4.
- Closed: phase 9 item 3; CPU-only speed measured.
