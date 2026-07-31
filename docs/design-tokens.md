# Design tokens

OpenTranscribe’s visual system lives in `src/styles/tokens.css`. The tokens are
loaded before base, shared-component, and view styles through
`src/styles/main.css`, so every Vue view and app-owned PrimeVue wrapper can use
the same values.

## Token groups

- `--canvas`, `--surface`, `--text`, `--border`, `--divider`, and state colors
  are semantic theme tokens. `--border-width` keeps component outlines and
  internal dividers on the same one-pixel stroke. Light, dark, and system
  themes redefine these values.
- `--font-*` and `--line-height-*` define the type scale and supported weights.
- `--space-*` is a four-pixel spacing scale with half steps for the app’s compact
  desktop density.
- `--control-height-*` keeps inputs, selects, and adjacent buttons on the same
  size tier.
- `--radius-*` defines the shape scale from compact controls through large
  surfaces, plus pill and circular shapes.
- `--shadow-*`, `--duration-*`, `--easing-*`, `--opacity-*`, and `--layer-*`
  centralize interaction and elevation behavior.
- `--layout-*` owns shell-level dimensions that must remain aligned across
  otherwise separate views.

## Usage

Use the smallest semantic token that describes the design decision:

```css
.meeting-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--font-size-md);
}
```

Prefer tokens for any value reused across components or expected to change with
the visual system. Keep intrinsic media dimensions, responsive breakpoints, and
truly component-specific geometry local. Add a new token only when a value has a
clear shared role; do not create aliases for every pixel value.

Theme-dependent colors belong only in `tokens.css`. Components should consume
semantic color tokens or derive a state with `color-mix()` from those tokens.
