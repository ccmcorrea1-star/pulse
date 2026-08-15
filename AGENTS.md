# Repository Guidelines

## Project Structure & Module Organization

This repository is a small, self-contained Pulse UI prototype. The root contains:

- `10-pulse-resumo.html`: the complete application, including markup, CSS, and browser-side JavaScript.
- `AGENTS.md`: contributor and automation guidance.

There are currently no separate source, test, asset, or build directories. Keep new UI work in the existing HTML file unless the project grows enough to justify splitting modules.

## Build, Test, and Development Commands

There is no package manager or build pipeline. Use a local static server for development:

```bash
python3 -m http.server 4173 --directory .
```

Open `http://127.0.0.1:4173/10-pulse-resumo.html` in a browser. For a JavaScript syntax check without creating temporary files:

```bash
sed -n '/<script>/,/<\/script>/p' 10-pulse-resumo.html | sed '1d;$d' | node --check
```

After UI changes, manually check the summary, devices, Clipboard, and History views at desktop and mobile widths (especially around 390px).

## Coding Style & Naming Conventions

Use two-space indentation and keep the document formatted consistently with the existing file. Use semantic HTML and accessible names for controls. Existing classes use the `pulse-` prefix and kebab-case (for example, `.pulse-device-detail`); follow that convention. Keep user-facing copy in Brazilian Portuguese, concise, and action-oriented. Prefer CSS custom properties for shared colors and preserve visible `:focus-visible` states. Keep interaction logic in the existing script and avoid adding dependencies without a clear need.

## Testing Guidelines

No automated test framework or coverage target is configured. Validate JavaScript syntax with the command above, then perform a manual smoke test of navigation, approval/rejection actions, transfer pause/resume, file selection, Clipboard actions, and responsive layout.

## Commit & Pull Request Guidelines

No Git history is available in the current workspace, so no existing commit convention can be inferred. Use short imperative messages, preferably Conventional Commit style (for example, `fix: stack summary columns on mobile`). Pull requests should explain the user-visible change, list manual checks, and include before/after screenshots for visual changes.

## Security & Configuration Notes

This is a front-end prototype with mock data and no backend. Do not add real credentials, private network data, or production transfer logic to the HTML. Treat uploaded file previews and generated object URLs as local test data only.
