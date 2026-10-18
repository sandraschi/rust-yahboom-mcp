# HANDOVER -- rust-yahboom-mcp

**Last updated:** 2026-10-18
**Phase:** local gates green. Visual diff and installer are still open.

## What was done

Legacy frames are in `docs/parity/legacy/`. The Rust workspace builds. `status --json` reads the live gateway on port 10892 and reports the robot offline at `192.168.1.11`. The desktop shell has one page per legacy route. Drive buttons ask for a confirm click and stay disabled until ROS reports connected.

## Next

Open `yahboom-app`, screenshot every page into `docs/parity/rust/`, and fix the layout against the legacy frames. Then lock kittest goldens. Installer and a GitHub release are separate and were not published.

## Deliberately not done

No chassis command was sent. `health_check` through `POST /api/v1/control/tool` returns HTTP 500 from the Python gateway while the link is down; `GET /api/v1/health` is the read that works. Multi-step movement cards (square, figure-8, dances) are labeled like the webapp and send one gateway operation after confirm, not the webapp's local step list.
