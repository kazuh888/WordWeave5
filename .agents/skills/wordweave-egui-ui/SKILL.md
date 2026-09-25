---
name: wordweave-egui-ui
description: "Build or refine WordWeave5 native egui screens from a reference image, including Japanese text/icon alignment and narrow-window layout. Use for screen prototypes and visual UI fixes; not for backend-only work, CI configuration, or general framework comparisons."
---

# WordWeave egui UI

Use the repository AGENTS.md and the installed egui version. This skill supplies rendering-specific lessons, not another development lifecycle.

## Keep the visual change bounded

Use the user's reference image and stated correction to identify the target controls, alignment, spacing, and required data states. Do not redesign unrelated screens. For a standalone prototype, use synthetic data and separate it from real storage and Codex connections. Explain which controls are simulated. A Git-ignored prototype will not be available to GitHub CI or other clones; disclose this instead of claiming CI covers it.

## Native egui pitfalls

- `Frame::show` inherits its parent's layout. A card inside a horizontal row needs an explicit vertical layout before fixing its width. Otherwise children can expand the row far beyond the window. Assert rendered right bounds with short, long, empty, and large-number content.
- Japanese fonts can have asymmetric space inside the line box. `Align2::CENTER_CENTER` alone does not prove visible characters are centered. For single-line control text, inspect the laid-out Galley's `mesh_bounds` and align its center to the target center; keep ordinary paragraph line spacing unchanged. An empty/nonfinite mesh needs a layout-rect fallback. Do not hard-code a downward offset from one screenshot.
- Icon and text centers should use the same coordinate system. Reserve icon space explicitly; whitespace padding is font-dependent. When repainting a label, keep a real Button (or equivalent focus/keyboard/accessibility semantics), prevent double-painted text, and preserve its accessible label, click area, focus, disabled and hover states.
- eframe can apply a theme after the creation callback. If a headless test differs from the native window, compare the actual font/style setup rather than relaxing assertions. Avoid recreating fonts or textures every frame.
- A small window at an enlarged text scale has much less logical space. Check header/footer height as well as horizontal overflow; collapse secondary navigation when needed and retain a scrollable body. In-app zoom is not a substitute for Windows DPI testing.

## Evidence and stopping

- For ordinary text buttons and selectable tabs, use `src/app/controls.rs` (`Button`, `UiControls`). It preserves native egui interaction and centers visible glyphs in both axes. Collapsing headers remain left aligned and are centered vertically. Existing icon+text home controls have their own group alignment; do not apply the text-only wrapper to image/shortcut groups without extending its layout and tests.
- Search changed UI files for raw `.button(`, `egui::Button::new`, `.selectable_value(` and `CollapsingHeader::new` to catch bypasses. Check enabled/disabled, wrapped Japanese, and multiple zoom factors with the shared control tests. A screenshot of one home button does not establish whole-app alignment.
- Keep the status summary one line. Notification details must be available by click/tap as well as hover; preserve visible fatal warnings and cancellation controls. A settings-repair action should navigate to the exact field, focus/scroll once, explain what to change and how to verify it, and preserve unfinished input.

For a reported alignment bug, first reproduce it with the actual Windows font where available. A useful regression test inspects painted text bounds relative to the button/icon center, not merely the widget rectangle or source text. Test more than one label and scale; preserve focus and keyboard behavior when rendering changes.

Run the focused tests and build. Capture the actual native screen after the change at a representative desktop size and a narrow/enlarged layout. Inspect clipping, Japanese glyphs, centers and control states. Geometry tests do not establish visual acceptance or native IME, audio, pen, screen-reader behavior. Report those separately if not exercised. Do not overwrite a running executable or stop the user's app without consent; place a distinct verification build or request closure.

Reuse the existing isolated preview's screenshot/test hooks when present; do not require that ignored local directory for future tasks. Keep test fixtures in tracked code if they must run in CI. After the requested visual check passes, provide the updated artifact and remaining acceptance question; do not add more decoration or review loops.

API reference for the installed 0.31.1 implementation: [Galley / mesh_bounds](https://docs.rs/epaint/0.31.1/epaint/text/struct.Galley.html). Verify the applicable API before using this technique with a different version.
