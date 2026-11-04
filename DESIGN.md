# Design System

## Universal File Converter

## 1. Design direction

Create a modern, calm, utility-focused desktop interface. The product should feel dependable and professional rather than flashy. Prioritize clarity, visible status, and low-friction file handling.

## 2. Visual principles

- Minimal visual noise.
- Strong hierarchy between file name, conversion target, status, and actions.
- Status must use both color and text/icon, never color alone.
- Use consistent spacing and predictable alignment.
- Avoid excessive gradients, glass effects, and decorative animation.
- Respect system light/dark preference; allow manual override.

## 3. Suggested palette

Use semantic tokens rather than hardcoding colors in components.

### Light theme

- `background`: #F7F8FA
- `surface`: #FFFFFF
- `surface-muted`: #EEF1F4
- `text-primary`: #17212B
- `text-secondary`: #5F6B76
- `border`: #DCE2E8
- `accent`: #2563EB
- `success`: #16845B
- `warning`: #A76510
- `danger`: #C43D3D
- `info`: #2878A8

### Dark theme

- `background`: #111820
- `surface`: #19232D
- `surface-muted`: #222F3B
- `text-primary`: #F2F5F7
- `text-secondary`: #A9B5C0
- `border`: #34424F
- `accent`: #6EA8FE
- `success`: #4CC38A
- `warning`: #F0B65A
- `danger`: #FF7777
- `info`: #75BDE8

## 4. Typography

- Primary font: system UI stack for reliable cross-platform rendering.
- Suggested stack: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif.
- Body: 14px desktop baseline.
- Secondary metadata: 12px.
- Page title: 24–28px.
- Section title: 16–18px.
- Avoid using too many font weights.

## 5. Spacing and shape

Use a 4px base spacing scale:

- 4, 8, 12, 16, 20, 24, 32, 40, 48px.
- Main content padding: 24px desktop; 16px compact.
- Card radius: 12px.
- Button/input radius: 8px.
- Minimum interactive target: 36px high; prefer 40px or more.
- Use subtle borders; avoid heavy shadows.

## 6. Main application layout

- Left navigation: Convert, Queue, History, Engines, Settings.
- Main content area: page title, concise helper text, primary work surface.
- Optional bottom status area: engine readiness and app version.
- Do not make the navigation wider than needed.

## 7. Convert screen

- Large drop zone with clear "Add files" action.
- Accepted file types are informational and must reflect actual installed capabilities.
- File list shows name, detected type, size, and remove action.
- Target format selector only shows valid targets for selected files.
- Output directory is visible before conversion.
- Primary CTA: "Convert".
- For mixed batches, clearly explain whether target format applies to all files or per-file.

## 8. Queue and result states

Use explicit states:

- Queued
- Checking file
- Waiting for engine
- Converting
- Validating output
- Completed
- Completed with warnings
- Failed
- Cancelled

Each row should show source name, target, status, progress when available, and a details action.

## 9. Empty, error, and dependency states

- Empty state should explain how to add files.
- Unsupported pair: state source and target formats and explain that this conversion is not currently supported.
- Missing engine: show engine name, detection status, and platform-specific setup guidance.
- Failed job: show a concise summary and expandable technical details.
- Never use generic "Something went wrong" when a useful cause is known.

## 10. Motion and feedback

- Use short, subtle transitions (120–180ms).
- Respect reduced-motion preferences.
- Do not animate progress if the underlying engine does not report progress.
- Provide clear feedback for file added, job started, job completed, and job failed.

## 11. Accessibility

- Keyboard-operable navigation and dialogs.
- Visible focus ring.
- Labels for all controls.
- Screen-reader announcements for job state changes.
- Sufficient contrast for text and status indicators.
- Do not rely on drag-and-drop as the only way to add files.
