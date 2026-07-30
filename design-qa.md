# OpenTranscribe Full-Width Main Pane Design QA

## Comparison target

- Source visual truth:
  `/var/folders/th/_kjfxnjx6gq9570f318lmwnc0000gn/T/codex-clipboard-6f5c1cc5-18dc-4ff0-8657-5d73c99a0688.png`
- Requested change: replace the fixed-width route workspace with content that
  fills the available main pane.
- Implementation screenshot: `/tmp/opentranscribe-full-width-final.png`
- Combined comparison: `/tmp/opentranscribe-layout-comparison.png`
- Viewport: 1508 × 841 CSS pixels.
- Source pixels: 3016 × 1682 at 2× density, normalized to 1508 × 841.
- Implementation pixels: 1508 × 841 at 1× density.
- State: light theme, Home, first-run recording setup.

## Full-view comparison

The source shows the original fixed-width page and 880px first-run surface,
which leave a large unused column at wider desktop sizes. The revised
implementation fills the route pane and retains adaptive horizontal gutters:

- main pane: 1264px
- route page: 1262px
- first-run surface: 1172px
- first-run left and right gutters: 45px

All `.page` routes were checked at 1508px: Home, Inbox, Project, Processing,
Settings, and Trash fill the main pane without horizontal overflow. Settings
was also checked at the 900 × 640 minimum window size and did not overflow.

A focused-region comparison was unnecessary because the requested change is
limited to the major page-width relationship. The typography, controls, icons,
colors, borders, and spacing within the setup surface remain unchanged and are
clearly readable in the normalized full-view comparison.

## Required fidelity surfaces

- Fonts and typography: unchanged; hierarchy, weights, wrapping, and line
  lengths remain consistent.
- Spacing and layout rhythm: fixed outer max-widths were removed; adaptive
  24–48px route gutters preserve edge breathing room.
- Colors and visual tokens: unchanged and still sourced from semantic tokens.
- Image quality and asset fidelity: no raster assets are present; existing
  application logo and Lucide controls remain sharp.
- Copy and content: unchanged. Preview fixture names and paths differ from the
  native screenshot by design and do not affect layout fidelity.

## Findings and comparison history

- Initial P2 — fixed-width utility workspace.
  - Evidence: the source page and setup surface stop growing while the main
    pane continues, producing a wide empty right column.
  - Fix: changed `.page` to `width: 100%` with adaptive gutters and
    `.first-run` to `width: 100%`.
  - Post-fix evidence: `/tmp/opentranscribe-full-width-final.png` and the
    measured dimensions above show full pane utilization with no overflow.

No actionable P0, P1, or P2 findings remain. No browser warnings or errors were
reported.

## Follow-up polish

No P3 changes are required for this scoped layout correction.

final result: passed
