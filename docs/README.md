# Cyberdeck Hub project site

Served by GitHub Pages from `main` → `/docs` at
<https://cybercore-tech.github.io/cyberdeck-hub/>.

- `index.html`: page content, plus `window.SITE` (default theme and signature mashups)
- `styles.css`, `app.js`: the shared Cybercore project-site kit (same files on every family site)
- `shots/`: the 12 gallery screenshots (WebP)

This folder also holds the mdBook source (`book.toml`, `src/`, `theme/`,
`STYLE_REFERENCE.md`). Pages serves only the site files above; `.nojekyll`
keeps GitHub from running Jekyll over the markdown.

Themes load at runtime from the org site's registry (`/data/cybergrid.json`,
`/data/themes/<family>/<name>.json` on cybercore-tech.github.io), so every
project site shares one source of truth. A signature mashup takes surfaces
(bg/panel/line/muted/white) from its `base` palette and neons from its
`accent` palette.

Local preview: serve a folder containing `cyberdeck-hub/` (this dir) and `data/`
(copied or symlinked from the org site repo), then open
`http://127.0.0.1:<port>/cyberdeck-hub/`.
