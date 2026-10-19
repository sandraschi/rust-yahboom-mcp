# UI parity

**Status:** dashboard layout now follows the legacy board. Other pages are still the shared card list.

Rust frames are `docs/parity/rust/*.png` (30 pages, 1280x860, dark theme, kittest wgpu). Same slugs as `docs/parity/legacy/` plus `crossconnect.png` and `raw.png`.

Dashboard has the connection banner, stack-health chips, bring-up copy, Control Center line, server/robot/tools tiles, the offline stream panel, the four telemetry tiles, and the deployment column (PTZ, headlight, Tapo speak, keyboard, lights, voice). PTZ, speak, and lights confirm through the tool bus. Headlight and Tapo listen do not: GPIO and the microphone are separate HTTP paths.

Movement and the other card pages still use one vertical list. They are not the webapp's dense panels.

## Exceptions already known

- Near-identical is the bar. Pixel-nil against Tailwind is not.
- `viz.png` is the 3D shell with the mesh request held. `viz-error.png` is the legacy page after `boomy_00_complete.stl` fails to parse. Both are original-app states.
- Mission Control is a route in `App.tsx` and is in the Rust nav. The legacy sidebar does not list it.
- Movement preset cards use the same titles and descriptions. A confirm click sends one `yahboom_tool` operation. The webapp plays a local step list for square, figure-8, and the dances.
- Crossconnect and Raw JSON are kernel pages past the 28 legacy routes.
- Settings has the fleet LLM card (provider, base URL, model, vault key, test button).
