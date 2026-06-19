# Product

## Register

product

## Users

Electric bass players at intermediate to advanced level. They practice at home, often in a dim room with their bass and an amp or audio interface. They're musicians, not developers — they care about getting notes right, not about software. They want the tool to disappear and let them focus on playing.

## Product Purpose

Bassical replaces scattered practice workflows (YouTube slow-down, paper tabs, metronome apps) with a single desktop tool. The user loads their own audio, calibrates tempo, writes or imports tabs, and practices with a synchronized play-along. Success means the bassist spends more time playing and less time managing tools.

## Brand Personality

**Dark, precise, technical.** Like a DAW or audio workstation. The UI should feel like a professional tool — dense when it needs to be, quiet when it doesn't. No decorative flourishes. Every pixel earns its place. The green accent is the "active" signal — it means "this is working, this is live."

## Anti-references

- Songsterr (cluttered web UI, gamification, tabs as content platform)
- Ultimate Guitar (ads, premium gates, visual noise)
- Generic SaaS dashboards (card grids, rounded corners, pastel palettes)
- Music-learning apps with gamification (streaks, points, cartoon mascots)

## Design Principles

1. **Studio monitor aesthetic** — the UI is the control surface, not the performance. Dark background, high contrast for data, muted for chrome.
2. **Content is the waveform and the tab** — everything else is scaffolding. Minimize chrome, maximize the working area.
3. **Precise feedback** — BPM to the decimal, milliseconds for timing, exact file paths. No vague states.
4. **Zero friction to play** — adding a song should take under 10 seconds. Practice should start in one click.
5. **No personality theater** — no onboarding wizards, no tooltips explaining what a bass is, no decorative icons. The user knows what they're doing.

## Accessibility & Inclusion

- WCAG AA contrast for all text
- Keyboard navigation for all core flows
- Reduced motion: no essential animation, all transitions are instant on `prefers-reduced-motion`
- Color is never the only indicator — status always has a text label
