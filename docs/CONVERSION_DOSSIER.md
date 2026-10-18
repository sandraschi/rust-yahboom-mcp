# CONVERSION DOSSIER -- rust-yahboom-mcp

**Phase:** 0. Screens captured 2026-10-18. No `.rs` written.
**Legacy:** `D:\Dev\repos\yahboom-mcp` `main` @ `58378e3` (untouched).
**Oracle:** `http://127.0.0.1:10893`, viewport 1280x860, dark.
**Robot:** did not move. Gateway reported LINK LOST, target 192.168.1.11, ROS disconnected. Movement presets stayed disabled. No drive command was sent.

## Frames

`docs/parity/legacy/<route>.png` is the first viewport of each route.

`<route>-scroll-N.png` is the same page after scrolling the main pane, only where that pane was taller than the viewport.

| File | Route | Scroll frames |
|---|---|---|
| dashboard | /dashboard | 2 |
| mission-control | /mission-control | 1 |
| status | /status | 1 |
| analytics | /analytics | 0 |
| viz | /viz | 0 |
| map | /map | 0 |
| slam | /slam | 1 |
| lidar-map | /lidar-map | 0 |
| movement | /movement | 1 |
| sensors | /sensors | 0 |
| peripherals | /peripherals | 1 |
| audio | /audio | 1 |
| voice | /voice | 1 |
| voice-upgrade | /voice-upgrade | 2 |
| lidar-addon | /lidar-addon | 0 |
| chat | /chat | 0 |
| missions | /missions | 0 |
| llm | /llm | 0 |
| skills | /skills | 0 |
| workflows | /workflows | 0 |
| diagnostics | /diagnostics | 2 |
| inbox | /inbox | 0 |
| logs | /logs | 1 |
| tools | /tools | 1 |
| apps | /apps | 0 |
| settings | /settings | 1 |
| onboarding | /onboarding | 1 |
| help | /help | 1 |

28 routes. Mission Control is in the router and not in the sidebar. Sidebar Dashboard points at `/`, which redirects to `/dashboard`.

## States worth keeping

- Dashboard opens on a red "No connection to robot" stack-health card, then CONTROL CENTER. Scroll 2 reaches Tapo Audio, keyboard WASD, light presets, Voice and Media Hub.
- Movement is three groups: Preset movements, Patrolling, Dancing. The scroll frame shows the full card grid.
- Visualization shell is `viz.png`: title "3D Visualization", pre-alpha line, Simulated pill, orbit hint, heading/battery/velocity strip. `viz-error.png` is the page a moment later: SUBSYSTEM FAILURE, `Could not load /assets/meshes/boomy_00_complete.stl`, invalid typed array length. The shell frame was taken with that mesh request held so the error boundary did not replace the page.
- Lidar Map was still on "Loading map...". Apps Hub showed discovery failed (Not Found).

## Pass bar

Same groups, order, labels, and cards as these frames. A 2-to-4 page shell fails. Pixel-nil against Tailwind is not required.

## Not done

Tool and portmanteau table. Any Rust.
