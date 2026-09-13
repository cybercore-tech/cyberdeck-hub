# CYBERDECK UI Cheatsheet

Every CSS class, element id, and JS function in the hub — so you can say
"make `.folder-chip` bigger" instead of screenshotting it. Lives in
`static/ui.css` (styles) and `src/views.rs` (HTML + inline `<script>`).

Regenerate this by hand whenever a UI pass changes selector names —
it's a snapshot, not generated from the code, so it can drift.

## Layout / chrome (every page)

| What it is | Selector |
|---|---|
| Whole top bar | `.topbar` |
| "CYBERDECK//" wordmark | `.brand`, `.brand-accent` (the `//`) |
| Nav links (Hub/Scans/Reports) | `.topbar-nav a` → `.btn-ghost` |
| Scanline/glow background layers | `.bg-scanlines`, `.bg-glow` |
| Page content wrapper | `main` |
| Page header pane (title + subtitle) | `.dash-head` (pane bg/border/glow), `.dash-head h1` (the purple pulsing title) |

## Theme picker + custom theme creator

| What it is | Selector / id |
|---|---|
| Theme picker button (top right) | `#theme-picker-btn`, label text `#theme-picker-label` |
| Its dropdown arrow | `.caret-down` |
| The dropdown panel itself | `#theme-dropdown` → `.theme-dropdown` |
| One family heading inside it ("CYBERPUNK", "DEFAULT", ...) | `.theme-group-label` |
| One clickable theme row | `.theme-item` (add `.selected` = currently active) |
| Where saved custom themes get injected | `#custom-theme-group` |
| "+ Theme" button | inline `.btn-ghost`, opens `#theme-creator-overlay` |
| Custom theme popup window | `#theme-creator-overlay` → `.viewer-overlay` / `.viewer-window` |
| The 11 color swatches | `#ct-swatches` → `.ct-grid` / `.ct-swatch` (each `<input type=color>` is `#ct-<role>`, role ∈ bg/fg/acid/pink/purple/cyan/orange/red/panel/line/muted) |
| Its name field | `#ct-name` |

**JS**: `toggleDropdown(event, id)` / `closeAllDropdowns()` open/close *any*
dropdown (theme picker, archive format) — generic, not theme-specific.
`selectTheme(name)`, `applyTheme(name)` (does the actual repaint),
`openThemeCreator()` / `closeThemeCreator()` / `previewCustomTheme()` /
`saveCustomTheme()` / `loadCustomThemesIntoPicker()`.

## Report viewer popup (the "View Output" modal)

| What it is | Selector / id |
|---|---|
| The whole dimmed overlay | `#viewer-overlay` → `.viewer-overlay` |
| The window box inside it | `.viewer-window` (reuses `.window`) |
| Its titlebar dots | `.dot`, `.dot-a` / `.dot-b` / `.dot-c` (pink/purple/cyan, all glow) |
| Its title text | `#viewer-title` → `.titlebar-text` |
| Close (X) button top-right | `.win-close` |
| The rendered report body | `#viewer-content` → `.markdown-body` |
| Bottom button row | `.viewer-actions` (Open Full Page = `#viewer-fullpage`) |

**JS**: `openViewer(path)` (fetches `/api/reports/*path`, fills the popup),
`closeViewer()`.

## Hub page (`/`)

| What it is | Selector |
|---|---|
| One collapsible category (Daemons, Network Tools, ...) | `<details class="cat-section">` |
| Its clickable header row | `<summary class="cat-head">` (colored left border via inline `--cat-color`) |
| The ▸ arrow that rotates when open | `.cat-caret` |
| The "(4)" count badge next to a category | `.cat-count` |
| The auto-refreshing card grid | `#hub-grid` → `.grid` |
| One tool card | `.card` |
| Card title / description / path line | `.card-title`, `.card-desc`, `.card-meta` |
| Status pill (ONLINE/ACTIVE/INSTALLED/...) | `.badge` + one of `.status-running` (green) / `.status-partial` (orange) / `.status-stopped` (grey) / `.status-unknown` (red) |
| Open/Wiki links row | `.card-actions` → `.btn-link` |

**JS**: `saveCatState(details)` / `restoreCatState()` persist which
categories are collapsed across the 10s htmx refresh.

## Scans page (`/scans`)

| What it is | Selector / id |
|---|---|
| One scan card | `.card` (same as hub cards) |
| Its Run button | `#btn-<id>` (id is a short code per scan — cpu/hw/pwr/mem/stg/thm/net/fan/aud/bios/disk/bat/svc/mobo/gpu/usb/kmod/mnt) |
| Its output badge | `#badge-<id>` |
| Its "View Output" link | inside `#view-<id>` |
| Files viewer window | `.window` → titled "diagnostics/ file viewer" |
| Its file list body | `#file-list-body` |
| A folder tag in that list ("cpu", "interfaces", ...) | `.folder-chip` (colored, stable per name) |
| A report with no folder | `.folder-chip-root` (red, square, **pulses**) |
| Archive panel window | `.window` → titled "compress diagnostics/ output" |
| Format dropdown button | `#archive-format-btn` / label `#archive-format-label` |
| Format dropdown panel | `#archive-format-dropdown` (rows are `.theme-item`, reused from the theme picker) |
| Compress button | `#btn-archive` |
| Result message line | `#archive-result` |

**JS**: `runScan(cmd, id, filename)`, `loadFileList()`, `folderColor(name)`
(JS mirror of the Rust `present::folder_color` hash), `runArchive()`,
`selectArchiveFormat(value, label)`.

## Reports page (`/reports`)

| What it is | Selector |
|---|---|
| The table | `.res-table` |
| Folder column chip | `.folder-chip` / `.folder-chip-root` (same as Scans page) |
| A report's title link | `.btn-link` |
| "This got auto-repaired" flag | `.badge.status-partial` with text "AUTO-FIXED" |
| Empty-state message ("No reports yet...") | `.empty-state` |

## Rendered report content (inside `.markdown-body`)

This is real report *content*, not UI chrome — deliberately **not**
uppercased, unlike everything else in this list.

| What it is | Selector |
|---|---|
| The container | `.markdown-body` |
| Headings | `.markdown-body h1/h2/h3` (cyan/purple/pink) |
| Inline `` `code` `` snippet ("the little info buttons") | `.markdown-body code` (cyan, not inside a `pre`) |
| A fenced multi-line code block | `.markdown-body pre` / `.markdown-body pre code` (plain, no cyan tint — a full command dump shouldn't look like a badge) |
| Tables inside a report | `.markdown-body table/th/td` |

## Generic building blocks (reused everywhere)

| What it is | Selector |
|---|---|
| Any terminal-style window (Files/Archive/Viewer popups) | `.window` → `.titlebar` (dots + text) → `.window-body` |
| A "fake button" link that looks like a button | `.btn-link` |
| A quiet outline button/link (nav, "Open Full Page", ...) | `.btn-ghost` |
| A themed text input or (via `select.text-input`, now unused) | `.text-input` |
| A form field caption | `.field-label` |
| Dimmed helper text | `.muted` |

## Global rules worth knowing about

- `[hidden] { display: none !important; }` — **always** use `el.hidden =
  true/false` to show/hide something; don't add a `display:` rule to
  something that also uses `[hidden]`, or it silently stops hiding (this
  bit us once already).
- `button, .btn-link, .btn-ghost, .titlebar-text, .card-title,
  .field-label { text-transform: uppercase; }` — the one place to change
  if the "shouty caps" look ever needs to go away globally.
- Every color is a CSS var from the active theme (`--bg --fg --acid
  --pink --purple --cyan --orange --red --panel --line --muted`) injected
  into `<style id="theme-vars">` at runtime — never hardcode a hex color
  in a new rule, use `var(--role)` so theme-switching still repaints it.
