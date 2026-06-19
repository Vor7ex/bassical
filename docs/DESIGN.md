# Design

## Theme

Deep dark with green accent. DAW/workstation aesthetic. Near-black background, muted chrome, high-contrast data areas.

## Color Palette

All values in OKLCH.

### Surface Ramp

| Token | OKLCH | Usage |
|-------|-------|-------|
| `--bg-root` | `oklch(0.13 0.01 160)` | App background — near-black with faint green tint |
| `--bg-surface` | `oklch(0.16 0.012 160)` | Elevated surfaces (header, toolbar, panels) |
| `--bg-raised` | `oklch(0.20 0.015 160)` | Cards, dialogs, dropdowns |
| `--bg-input` | `oklch(0.11 0.01 160)` | Input fields, code blocks |
| `--border-subtle` | `oklch(0.25 0.015 160)` | Default borders |
| `--border-strong` | `oklch(0.35 0.02 160)` | Emphasized borders, focus rings |

### Accent

| Token | OKLCH | Usage |
|-------|-------|-------|
| `--accent` | `oklch(0.72 0.18 155)` | Primary actions, active states, links — light green |
| `--accent-hover` | `oklch(0.78 0.20 155)` | Hover state |
| `--accent-muted` | `oklch(0.45 0.10 155)` | Subtle accent for badges, indicators |
| `--accent-bg` | `oklch(0.20 0.04 155)` | Accent background tints |

### Text

| Token | OKLCH | Usage |
|-------|-------|-------|
| `--text-primary` | `oklch(0.93 0.01 160)` | Primary text — near-white |
| `--text-secondary` | `oklch(0.65 0.015 160)` | Secondary text, labels |
| `--text-tertiary` | `oklch(0.50 0.015 160)` | Placeholder, disabled, metadata |
| `--text-danger` | `oklch(0.62 0.22 25)` | Errors, missing files — muted red |
| `--text-success` | `oklch(0.72 0.18 155)` | Success states — matches accent |

### Semantic

| Token | OKLCH | Usage |
|-------|-------|-------|
| `--danger-bg` | `oklch(0.18 0.04 25)` | Error backgrounds |
| `--danger-border` | `oklch(0.30 0.08 25)` | Error borders |

## Typography

**One family:** Inter (system fallback: -apple-system, Segoe UI, sans-serif). No second font.

| Scale | Size | Weight | Line Height | Letter Spacing | Usage |
|-------|------|--------|-------------|----------------|-------|
| Display | 24px | 600 | 1.2 | -0.01em | View titles |
| Heading | 16px | 600 | 1.3 | 0 | Section headings |
| Body | 13px | 400 | 1.5 | 0 | Default text |
| Caption | 11px | 500 | 1.4 | 0.02em | Column headers, labels, status |
| Mono | 12px | 400 | 1.4 | 0 | File paths, code, technical data |

- Body text max width: 75ch (not enforced in data-dense views)
- `text-wrap: balance` on headings
- No uppercase except column headers (caption scale, 0.04em tracking)

## Spacing

4px base unit.

| Token | Value | Usage |
|-------|-------|-------|
| `--space-1` | 4px | Tight gaps (icon-to-text) |
| `--space-2` | 8px | Component internal padding |
| `--space-3` | 12px | Standard gaps |
| `--space-4` | 16px | Section gaps |
| `--space-6` | 24px | Major section dividers |
| `--space-8` | 32px | View padding |

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 4px | Inputs, buttons, badges |
| `--radius-md` | 6px | Cards, dialogs |
| `--radius-lg` | 8px | Modals (max) |

No pill shapes. No 16px+ radii.

## Shadows

Minimal. The dark theme relies on surface color elevation, not shadows.

| Token | Value | Usage |
|-------|-------|-------|
| `--shadow-modal` | `0 8px 32px oklch(0 0 0 / 0.5)` | Dialog overlay |

## Components

### Buttons

- **Primary:** `bg-accent text-bg-root`. Hover: `bg-accent-hover`. No border.
- **Secondary:** `bg-surface text-secondary`. Border: `border-subtle`. Hover: `bg-raised`.
- **Danger:** `bg-danger-bg text-danger`. Border: `danger-border`. Hover: brighter bg.
- All: `radius-sm`, 4px 12px padding, 13px body text, no uppercase.

### Inputs

- `bg-input border-subtle text-primary`. Focus: `border-accent` + `outline: 2px solid accent/20%`.
- Placeholder: `text-tertiary`.
- Radius: `radius-sm`.

### Dialogs

- `bg-raised border-subtle radius-md`. No drop shadow on dark theme (use overlay dimming).
- Overlay: `bg-black/60`.
- Title bar: `bg-surface` with border-bottom.
- Actions: right-aligned, gap 8px.

### Status Indicators

- **OK:** Green dot + "Listo" text. No "● OK" — that's debug language.
- **Missing:** Red badge with "Faltante" text. Clickable to fix.
- **Processing:** Spinner or pulsing dot, never both.

### Data Table (Song List)

- Header: `bg-surface`, caption scale, `text-secondary`.
- Rows: `bg-root` default, `bg-surface` on hover, `bg-accent-bg` when selected.
- Selection: left accent border (2px, accent color) — not a gradient.
- Grid columns match mockup: #, title, artist, path, status.
- Click outside to deselect.

## Motion

- Duration: 120ms for micro-interactions (hover, focus), 200ms for transitions (open/close).
- Easing: `ease-out` (exponential curve). No bounce, no elastic.
- `prefers-reduced-motion: reduce` → all transitions instant.
- No entrance animations on the library view. Content is immediately visible.

## Z-Index Scale

| Level | Value | Usage |
|-------|-------|-------|
| Base | 0 | Default |
| Dropdown | 10 | Sort menus, tooltips |
| Sticky | 20 | Toolbar, status bar |
| Modal backdrop | 30 | Dialog overlay |
| Modal | 40 | Dialogs |
| Toast | 50 | Notifications |
