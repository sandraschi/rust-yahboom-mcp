# UI parity

**Status:** legacy frames and Rust stills are both on disk. Layout is not near-identical yet.

Rust frames are `docs/parity/rust/*.png` (30 pages, 1280x860, dark theme, kittest wgpu). Same slugs as `docs/parity/legacy/` plus `crossconnect.png` and `raw.png`.

What matches: page names, group order, indigo selection, dark ground, readable body text.

What does not: the legacy dashboard is a stack-health board with bring-up copy and shortcut buttons. The Rust dashboard is a short status column plus the PTZ, headlight, and speak controls. Card pages list the same preset titles. They are not the webapp's dense panels.

## Exceptions already known

- Near-identical is the bar. Pixel-nil against Tailwind is not.
- `viz.png` is the 3D shell with the mesh request held. `viz-error.png` is the legacy page after `boomy_00_complete.stl` fails to parse. Both are original-app states.
- Mission Control is a route in `App.tsx` and is in the Rust nav. The legacy sidebar does not list it.
- Movement preset cards use the same titles and descriptions. A confirm click sends one `yahboom_tool` operation. The webapp plays a local step list for square, figure-8, and the dances.
- Crossconnect and Raw JSON are kernel pages past the 28 legacy routes.
- Settings has the fleet LLM card (provider, base URL, model, vault key, test button).
