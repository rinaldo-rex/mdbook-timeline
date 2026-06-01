# Project History

A timeline of major project milestones using a custom marker (`journey`) configured
in `book.toml`.

{{journey}}

{{label}}2024 Q1{{/label}}
{{card-start}}
#### v2.0 — Complete Rewrite
company: mdBook Timeline
desc: Migrated from proof-of-concept to production-ready preprocessor. Added custom markers, theme-aware CSS, and the full interactive hover system.
tags: Rust | mdBook | Preprocessor
active: true
{{card-end}}

{{label}}2023 Q3{{/label}}
{{card-start}}
#### v1.0 — Initial Release
company: Open Source Community
desc: First public release with basic timeline rendering, year labels, and card support. Adopted by three documentation projects within the first month.
tags: Release | Community
{{card-end}}

{{label}}2023 Q1{{/label}}
{{card-start}}
#### Prototype
company: Personal Project
desc: Weekend hack to see if mdBook preprocessors could render interactive timelines. Hand-crafted HTML injected via a simple regex replace.
tags: Prototype | Experiment
{{card-end}}

{{/journey}}

---

## What Makes This Different

This page uses `timeline-marker = "journey"` in `book.toml`:

```toml
[preprocessor.timeline]
timeline-marker = "journey"
```

The marker word is fully configurable — use `{{era}}` for historical timelines,
`{{sprint}}` for agile retrospectives, or any word that fits your domain.
