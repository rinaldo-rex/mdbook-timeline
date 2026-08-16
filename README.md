# mdBook Timeline

A [mdBook](https://rust-lang.github.io/mdBook/) preprocessor that renders
interactive, theme-aware timelines using pure HTML + CSS.

## Design

Timelines are defined with simple markers in any chapter:

```markdown
{{timeline}}

{{label}}2024{{/label}}
{{card-start}}
#### Executive Director of Innovation
company: Global Tech Solutions
location: London, UK
desc: Leading the transformation of archival systems through AI-driven indexing.
tags: Strategic Planning | AI Systems | Team Leadership
active: true
{{card-end}}

{{label}}2021{{/label}}
{{card-start}}
#### Senior Product Architect
company: CloudStream Infrastructure
desc: Spearheaded the migration of legacy data centers to a unified cloud architecture.
tags: Architecture | Migration
![Data center](https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=900&q=60)
{{card-end}}

{{/timeline}}
```

This produces an interactive timeline with:
- Vertical line with dots for each entry
- Cards with title, company, location, description, tags, and images
- **Active** badge for current entries
- Duration labels between entries (e.g., "3 Years")
- "Years ago" indicator on hover (for numeric labels)
- Hover dimming: all other entries fade when one is highlighted
- Full dark/light theme support via mdBook CSS variables

## Installation

```bash
cargo install mdbook-timeline
```

## Usage

Add to your `book.toml`:

```toml
[preprocessor.timeline]
```

## Configuration

All options live under `[preprocessor.timeline]` in `book.toml`:

| Option             | Type     | Default      | Description                                              |
|--------------------|----------|--------------|----------------------------------------------------------|
| `timeline-marker`  | string   | `"timeline"` | The word used in `{{marker}}`/`{{/marker}}` blocks       |
| `duration-gaps`    | bool     | `true`       | Show duration labels between entries                     |
| `before`           | [string] | `[]`         | Run this preprocessor **before** the named preprocessors |
| `after`            | [string] | `[]`         | Run this preprocessor **after** the named preprocessors  |

The `before` and `after` keys are [standard mdBook preprocessor config](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html#regular-preprocessors)
that control execution order relative to other preprocessors. mdBook handles them
automatically — the preprocessor does not need code changes.

For example, to run before the built-in `index` preprocessor and after `links`:

```toml
[preprocessor.timeline]
before = ["index"]
after = ["links"]
timeline-marker = "journey"
```

Then in markdown:

```markdown
{{journey}}

{{label}}2024{{/label}}
{{card-start}}
#### First Milestone
{{card-end}}

{{/journey}}
```

## Syntax Reference

### Timeline Block

```
{{timeline}}
  ... entries ...
{{/timeline}}
```

The marker word (`timeline` by default) is configurable via `timeline-marker`.

### Entry Structure

Each entry consists of a **label** and a **card**:

```
{{label}}2024{{/label}}
{{card-start}}
#### Title Heading (h4)
key: value
key: value
...
{{card-end}}
```

### Label

The label is the time marker — it can be a year, month, era name, or any text:

```
{{label}}2024{{/label}}
{{label}}January 2024{{/label}}
{{label}}Medieval Period{{/label}}
```

- If the label is a **number** (e.g., `2024`), the preprocessor injects a
  "years ago" indicator that appears on hover.
- The duration gap between two entries is auto-computed when both labels parse
  as integers.

### Card Fields

Inside `{{card-start}}` / `{{card-end}}`, fields follow `key: value` syntax:

| Key           | Description                       | Example                                          |
|---------------|-----------------------------------|--------------------------------------------------|
| `#### Title`  | Card title (h4 heading)           | `#### Executive Director`                        |
| `company`     | Organization / subtitle           | `company: Global Tech`                           |
| `location`    | Location (shown after company)    | `location: London, UK`                           |
| `desc`        | Description paragraph             | `desc: Led the transformation.`                  |
| `tags`        | Pipe-separated tag list           | `tags: AI | Strategy | Leadership`                |
| `active`      | Highlight entry as current        | `active: true`                                   |
| `![alt](url)` | Standard markdown image           | `![screenshot](https://example.com/img.png)`     |

All fields are **optional** except the title.

### Markdown in cards

Every text field (`title`, `company`, `location`, `desc`, `tags`, and the entry
`label`) supports **inline markdown**:

| Syntax           | Result                                    |
|------------------|-------------------------------------------|
| `**bold**`       | **bold**                                  |
| `*italic*`       | *italic*                                  |
| `~~strikethrough~~` | ~~strikethrough~~                      |
| `` `code` ``     | `` `code` `` (inline)                      |
| `[link](url)`    | a link                                    |
| `<u>underline</u>` | <u>underline</u> (raw inline HTML)      |

```markdown
desc: Led the **transformation** using *agile* and ~~waterfall~~ processes.
```

Inline HTML (such as `<u>…</u>` for underline) is passed through unchanged.

### Multiple Images

Add as many `![alt](url)` lines as needed — each renders as a full-width image.

## How It Works

1. The preprocessor scans each chapter for `{{timeline}}` … `{{/timeline}}` blocks
2. Each block is parsed into entries (label + card)
3. Cards are parsed using `key: value` lines and `####` headings
4. The block is replaced with HTML — a `<div class="tl">` container with entry/card markup
5. A `<style>` block with theme-aware CSS is injected into the chapter
6. A `<script>` block handles hover effects, "years ago" labels, and
   duration-gap positioning at runtime

The CSS uses mdBook theme variables (`--fg`, `--bg`, `--links`,
`--theme-popup-bg`, `--theme-popup-border`) so timelines automatically match
light, dark, and custom themes.

## License

MIT
