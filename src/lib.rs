use mdbook_preprocessor::{
    book::{Book, BookItem},
    errors::Error,
    Preprocessor, PreprocessorContext,
};
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::sync::LazyLock;

// ── Regex patterns (compiled once) ───────────────────────────────────

static ENTRY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)\{\{label\}\}(?P<label>.*?)\{\{/label\}\}.*?\{\{card-start\}\}(?P<card>.*?)\{\{card-end\}\}",
    )
    .unwrap()
});

static IMAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!\[(?P<alt>[^\]]*)\]\((?P<url>[^\)]+)\)"#).unwrap());

// ── CSS ─────────────────────────────────────────────────────────────

const TIMELINE_CSS: &str = r#"
/* ═══════════════════════════════════════════════════════════════════
   mdBook Timeline — theme-aware via mdBook CSS custom properties
   ═══════════════════════════════════════════════════════════════════ */

.tl {
    position: relative;
    padding: 0 48px;
    width: calc(100% + 280px);
    max-width: 1280px;
    margin: 32px 0 32px -200px;
    font-size: inherit;
}

.tl::before {
    content: "";
    position: absolute;
    left: 120px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--fg);
    opacity: 0.15;
}

.tl-entry {
    display: grid;
    grid-template-columns: 56px 1fr;
    gap: 0 32px;
    align-items: start;
    position: relative;
}

.tl-entry::before {
    content: "";
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--fg);
    opacity: 0.25;
    position: absolute;
    left: 72px;
    transform: translateX(-50%);
    top: 30px;
    z-index: 1;
    transition: background 0.25s ease, box-shadow 0.25s ease;
}

.tl-entry.active::before {
    background: var(--links, #20609f);
    opacity: 1;
    box-shadow: 0 0 0 3px rgba(32, 96, 159, 0.2);
}

.tl-entry.hovered::before {
    background: var(--links, #20609f);
    opacity: 1;
    box-shadow: 0 0 0 3px rgba(128,128,128,0.15);
    animation: tl-dot-pulse 0.8s ease-in-out infinite;
}

@keyframes tl-dot-pulse {
    0%, 100% { box-shadow: 0 0 0 3px rgba(128,128,128,0.15); }
    50% { box-shadow: 0 0 0 8px rgba(128,128,128,0.20); }
}

.tl-label-wrap {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding-top: 26px;
    padding-right: 16px;
}

.tl-label {
    font-size: inherit;
    opacity: 0.55;
    letter-spacing: 0.04em;
    color: var(--fg);
}

.tl-card {
    background: var(--theme-popup-bg, var(--bg, #fafafa));
    border: 1px solid var(--theme-popup-border, rgba(0,0,0,0.12));
    border-radius: 10px;
    padding: 28px 32px;
    position: relative;
}

.tl-card .tl-years-ago {
    position: absolute;
    bottom: 14px;
    right: 18px;
    font-size: inherit;
    opacity: 0;
    transition: opacity 0.25s ease;
    pointer-events: none;
    letter-spacing: 0.03em;
    color: var(--fg);
}

.tl-entry.hovered .tl-card .tl-years-ago { opacity: 0.45; }

.tl-card-title {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 8px;
}

.tl-card-title h3 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--fg);
    margin: 0;
}

.tl-badge-active {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: inherit;
    font-weight: 500;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--links, #20609f);
    border: 1px solid var(--links, #20609f);
    border-radius: 20px;
    padding: 4px 10px;
    white-space: nowrap;
    flex-shrink: 0;
}

.tl-badge-active::before {
    content: "";
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--links, #20609f);
}

.tl-company { font-size: inherit; opacity: 0.7; margin-bottom: 14px; color: var(--fg); }
.tl-company .tl-location { opacity: 0.6; }

.tl-desc {
    font-size: inherit;
    opacity: 0.8;
    line-height: 1.75;
    margin-bottom: 20px;
    color: var(--fg);
}

/* Paragraphs inside a multi-line description keep the card's line-height. */
.tl-desc p { margin: 0 0 12px 0; line-height: 1.75; }
.tl-desc p:last-child { margin-bottom: 0; }

.tl-image {
    width: 100%;
    border-radius: 6px;
    overflow: hidden;
    margin-bottom: 18px;
}

.tl-image img {
    width: 100%;
    height: 180px;
    object-fit: cover;
    display: block;
}

.tl-tags { display: flex; flex-wrap: wrap; gap: 8px; }

.tl-tag {
    font-size: inherit;
    opacity: 0.7;
    background: var(--theme-popup-bg, var(--bg));
    border: 1px solid var(--theme-popup-border, rgba(0,0,0,0.15));
    border-radius: 4px;
    padding: 4px 10px;
    letter-spacing: 0.02em;
    color: var(--fg);
}

.tl-duration-gap { margin-bottom: 32px; }

.tl-duration-label {
    position: absolute;
    font-size: inherit;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    opacity: 0.45;
    writing-mode: vertical-rl;
    transform: translateX(-50%) translateY(-50%) rotate(180deg);
    z-index: 2;
    pointer-events: none;
    color: var(--fg);
}

.tl.dimmed .tl-entry { opacity: 0.3; transition: opacity 0.25s ease; }
.tl.dimmed .tl-entry.hovered { opacity: 1; }
.tl.dimmed .tl-duration-label { opacity: 0.15; }
.tl.dimmed .tl-duration-label.keep { opacity: 0.45 !important; }
"#;

// ── JS ──────────────────────────────────────────────────────────────

const TIMELINE_JS: &str = r#"
(function () {
    var currentYear = new Date().getFullYear();

    document.querySelectorAll(".tl").forEach(function (timeline) {
        var entries = timeline.querySelectorAll(".tl-entry");
        var hasGaps = timeline.querySelector(".tl-duration-gap") !== null;

        entries.forEach(function (entry, idx) {
            entry.addEventListener("mouseenter", function () {
                timeline.classList.add("dimmed");
                entry.classList.add("hovered");
                var labels = timeline.querySelectorAll(".tl-duration-label");
                labels.forEach(function (lbl, li) {
                    lbl.classList.toggle("keep", li === idx - 1 || li === idx);
                });
            });
            entry.addEventListener("mouseleave", function () {
                timeline.classList.remove("dimmed");
                entry.classList.remove("hovered");
                var labels = timeline.querySelectorAll(".tl-duration-label");
                labels.forEach(function (lbl) { lbl.classList.remove("keep"); });
            });
        });

        entries.forEach(function (entry) {
            var labelEl = entry.querySelector(".tl-label");
            var text = labelEl ? labelEl.textContent.trim() : "";
            var yr = parseInt(text, 10);
            if (!isNaN(yr) && yr > 0 && text === String(yr)) {
                var diff = currentYear - yr;
                var ago = diff === 0 ? "This year" : diff === 1 ? "1 year ago" : diff + " years ago";
                var span = document.createElement("span");
                span.className = "tl-years-ago";
                span.textContent = ago;
                var card = entry.querySelector(".tl-card");
                if (card) card.appendChild(span);
            }
        });

        if (!hasGaps) return;
        requestAnimationFrame(function () {
            requestAnimationFrame(function () {
                var gaps = timeline.querySelectorAll(".tl-duration-gap");
                var containerTop = timeline.getBoundingClientRect().top;
                var DOT = 30 + 4.5;
                for (var i = 0; i < gaps.length; i++) {
                    var a = entries[i];
                    var b = entries[i + 1];
                    if (!a || !b) continue;
                    var label = gaps[i].getAttribute("data-label");
                    if (!label) continue;
                    var topA = a.getBoundingClientRect().top - containerTop + DOT;
                    var topB = b.getBoundingClientRect().top - containerTop + DOT;
                    var mid = (topA + topB) / 2;
                    var span = document.createElement("span");
                    span.className = "tl-duration-label";
                    span.textContent = label;
                    span.style.left = "120px";
                    span.style.top = mid + "px";
                    timeline.appendChild(span);
                }
            });
        });
    });
})();
"#;

// ── Data structures ─────────────────────────────────────────────────

struct TimelineConfig {
    markers: Vec<String>,
    duration_gaps: bool,
    before: Option<String>,
    after: Option<String>,
}

impl TimelineConfig {
    fn from_ctx(ctx: &PreprocessorContext) -> Self {
        let custom = ctx
            .config
            .get::<String>("preprocessor.timeline.timeline-marker")
            .ok()
            .flatten();

        let mut markers = vec!["timeline".to_string()];
        if let Some(c) = custom {
            let trimmed = c.trim().to_string();
            if !trimmed.is_empty() && trimmed != "timeline" {
                markers.push(trimmed);
            }
        }

        Self {
            markers,
            duration_gaps: ctx
                .config
                .get::<bool>("preprocessor.timeline.duration-gaps")
                .ok()
                .flatten()
                .unwrap_or(true),
            before: ctx
                .config
                .get::<String>("preprocessor.timeline.before")
                .ok()
                .flatten(),
            after: ctx
                .config
                .get::<String>("preprocessor.timeline.after")
                .ok()
                .flatten(),
        }
    }
}

struct TimelineEntry {
    label: String,
    title: Option<String>,
    company: Option<String>,
    location: Option<String>,
    desc: Option<String>,
    tags: Vec<String>,
    active: bool,
    images: Vec<(String, String)>,
}

// ── Preprocessor ────────────────────────────────────────────────────

pub struct TimelinePreprocessor;

impl TimelinePreprocessor {
    pub fn new() -> Self {
        Self
    }
}

impl Preprocessor for TimelinePreprocessor {
    fn name(&self) -> &str {
        "timeline-preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let config = TimelineConfig::from_ctx(ctx);

        let mut had_timeline = false;
        let mut had_css = false;

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chap) = item {
                let mut local_count = 0usize;

                for marker in &config.markers {
                    let escaped = regex::escape(marker);
                    let block_re = Regex::new(&format!(
                        r"(?s)\{{\{{{}\}}\}}(?P<body>.*?)\{{\{{/{}\}}\}}",
                        escaped, escaped
                    ))
                    .unwrap(); // safe: regex::escape always produces valid regex

                    let result =
                        block_re.replace_all(&chap.content, |caps: &regex::Captures| {
                            had_timeline = true;
                            local_count += 1;
                            let block_content = caps.name("body").unwrap().as_str();
                            let entries = parse_entries(block_content);
                            render_timeline(&entries, local_count, &config)
                        });
                    chap.content = result.to_string();
                }

                if had_timeline && !had_css && local_count > 0 {
                    chap.content = format!(
                        "<style>\n{TIMELINE_CSS}\n</style>\n{}",
                        chap.content
                    );
                    had_css = true;
                }
            }
        });

        if had_timeline {
            book.for_each_mut(|item| {
                if let BookItem::Chapter(chap) = item {
                    if chap.content.contains("class=\"tl\"") {
                        chap.content = format!(
                            "{}\n<script>\n{TIMELINE_JS}\n</script>",
                            chap.content
                        );
                    }
                }
            });
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool, Error> {
        Ok(renderer == "html" || renderer == "markdown")
    }
}

// ── Parsing ─────────────────────────────────────────────────────────

fn parse_entries(block: &str) -> Vec<TimelineEntry> {
    ENTRY_RE
        .captures_iter(block)
        .map(|caps| {
            let label = caps
                .name("label")
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let card_body = caps.name("card").unwrap().as_str();
            parse_card(&label, card_body)
        })
        .collect()
}

fn parse_card(_label: &str, card: &str) -> TimelineEntry {
    let mut title: Option<String> = None;
    let mut company: Option<String> = None;
    let mut location: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut active = false;
    let mut images: Vec<(String, String)> = Vec::new();

    let lines: Vec<&str> = card.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if trimmed.starts_with("#### ") {
            title = Some(trimmed[5..].trim().to_string());
            i += 1;
            continue;
        }

        for cap in IMAGE_RE.captures_iter(trimmed) {
            let url = cap["url"].trim().to_string();
            let alt = cap
                .name("alt")
                .map(|a| a.as_str().trim().to_string())
                .unwrap_or_default();
            images.push((url, alt));
        }

        if let Some(colon) = trimmed.find(':') {
            let key = trimmed[..colon].trim().to_lowercase();
            let val = trimmed[colon + 1..].trim();

            match key.as_str() {
                "company" => company = Some(val.to_string()),
                "location" => location = Some(val.to_string()),
                "desc" | "description" => {
                    // Multi-line description: collect any line indented further
                    // than the `desc:` key itself as continuation content. A
                    // blank line between content lines becomes a paragraph
                    // break (`\n\n`); adjacent lines join with a soft `\n`.
                    let base_indent = indentation_of(line);
                    let mut block = Vec::new();
                    if !val.is_empty() {
                        block.push(val.to_string());
                    }
                    let mut j = i + 1;
                    while j < lines.len() {
                        let next = lines[j];
                        if next.trim().is_empty() {
                            // Blank line: candidate paragraph separator.
                            block.push(String::new());
                            j += 1;
                            continue;
                        }
                        if indentation_of(next) > base_indent {
                            block.push(next.trim().to_string());
                            j += 1;
                        } else {
                            break; // next field / title / end of block
                        }
                    }
                    desc = Some(join_desc_block(block));
                    i = j;
                    continue;
                }
                "active" => active = val.eq_ignore_ascii_case("true"),
                "tags" => {
                    tags = val
                        .split('|')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
        i += 1;
    }

    TimelineEntry {
        label: _label.to_string(),
        title,
        company,
        location,
        desc,
        tags,
        active,
        images,
    }
}

// ── Rendering ───────────────────────────────────────────────────────

fn render_timeline(entries: &[TimelineEntry], id: usize, config: &TimelineConfig) -> String {
    let mut html = String::new();

    if let Some(ref before) = config.before {
        html.push_str(before);
        html.push('\n');
    }

    html.push_str(&format!(
        r#"<div class="tl" id="tl-{id}">"#
    ));

    for (i, entry) in entries.iter().enumerate() {
        html.push_str(&render_entry(entry));
        if config.duration_gaps && i + 1 < entries.len() {
            html.push_str(&render_gap(entry, &entries[i + 1], i));
        }
    }

    html.push_str("</div>");

    if let Some(ref after) = config.after {
        html.push('\n');
        html.push_str(after);
    }

    html
}

fn render_entry(entry: &TimelineEntry) -> String {
    let active_class = if entry.active { " active" } else { "" };
    let badge = if entry.active {
        "<span class=\"tl-badge-active\">Active</span>"
    } else {
        ""
    };
    // Applied at render time (not parse time) so raw values stay intact for
    // parse tests and for HTML injected by other preprocessors.
    let title = entry
        .title
        .as_deref()
        .map(md_inline)
        .unwrap_or_else(|| "Untitled".to_string());
    let company_html = match (&entry.company, &entry.location) {
        (Some(c), Some(l)) => format!(
            "<div class=\"tl-company\">{} <span class=\"tl-location\">• {}</span></div>",
            md_inline(c),
            md_inline(l)
        ),
        (Some(c), None) => format!("<div class=\"tl-company\">{}</div>", md_inline(c)),
        (None, Some(l)) => {
            format!("<div class=\"tl-company\"><span class=\"tl-location\">{}</span></div>", md_inline(l))
        }
        (None, None) => String::new(),
    };
    let desc_html = entry.desc.as_ref().map(|d| render_desc_html(d)).unwrap_or_default();
    let images_html: String = entry
        .images
        .iter()
        .map(|(url, alt)| format!("<div class=\"tl-image\"><img src=\"{}\" alt=\"{}\" /></div>", url, alt))
        .collect();
    let tags_html = if entry.tags.is_empty() {
        String::new()
    } else {
        let tags: String = entry
            .tags
            .iter()
            .map(|t| format!("<span class=\"tl-tag\">{}</span>", md_inline(t)))
            .collect::<Vec<_>>()
            .join("");
        format!("<div class=\"tl-tags\">{}</div>", tags)
    };

    format!(
        "<div class=\"tl-entry{}\"><div class=\"tl-label-wrap\"><span class=\"tl-label\">{}</span></div><div class=\"tl-card\"><div class=\"tl-card-title\"><h3>{}</h3>{}</div>{}{}{}{}</div></div>",
        active_class, md_inline(&entry.label), title, badge,
        company_html, desc_html, images_html, tags_html,
    )
}

fn render_gap(a: &TimelineEntry, b: &TimelineEntry, idx: usize) -> String {
    let label = compute_gap_label(&a.label, &b.label).unwrap_or_default();
    if label.is_empty() {
        return String::new();
    }
    format!(
        "<div class=\"tl-duration-gap\" data-gap=\"{idx}\" data-label=\"{label}\"></div>"
    )
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Render markdown to full HTML (paragraphs included) via pulldown-cmark,
/// with GFM strikethrough enabled.
fn md_html(text: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(text.trim(), options);
    let mut buf = String::new();
    html::push_html(&mut buf, parser);
    buf.trim().to_string()
}

/// Convert markdown to inline HTML, unwrapping the single wrapping `<p>…</p>`
/// that pulldown-cmark generates for a text fragment. Supports bold, italics,
/// strikethrough, inline code, links, inline HTML (e.g. `<u>…</u>`) and more.
fn md_inline(text: &str) -> String {
    let trimmed = md_html(text);
    if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
        trimmed[3..trimmed.len() - 4].trim().to_string()
    } else {
        trimmed
    }
}

/// Render a description value as HTML. A single-paragraph description stays a
/// `<p class="tl-desc">`; a multi-paragraph (block) description is wrapped in
/// `<div class="tl-desc">` so each paragraph keeps its own spacing.
fn render_desc_html(desc: &str) -> String {
    let html = md_html(desc);
    if html.matches("<p>").count() <= 1 {
        format!("<p class=\"tl-desc\">{}</p>", md_inline(desc))
    } else {
        format!("<div class=\"tl-desc\">{}</div>", html)
    }
}

/// Number of leading whitespace characters in a line.
fn indentation_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Join a description block's raw lines into markdown:
/// adjacent content lines join with a single `\n`, and a blank line between
/// content becomes a paragraph break (`\n\n`). Leading/trailing blank lines
/// and run-on blanks are dropped.
fn join_desc_block(raw: Vec<String>) -> String {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for line in raw {
        if line.is_empty() {
            if !cur.is_empty() {
                paragraphs.push(cur.join("\n"));
                cur.clear();
            }
        } else {
            cur.push(line);
        }
    }
    if !cur.is_empty() {
        paragraphs.push(cur.join("\n"));
    }
    paragraphs.join("\n\n")
}

fn compute_gap_label(a: &str, b: &str) -> Option<String> {
    let x: i64 = a.parse().ok()?;
    let y: i64 = b.parse().ok()?;
    let diff = x - y;
    if diff <= 0 {
        Some("< 1 Year".to_string())
    } else if diff == 1 {
        Some("1 Year".to_string())
    } else {
        Some(format!("{diff} Years"))
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_entry() {
        let input = r#"
{{label}}2024{{/label}}
{{card-start}}
#### Executive Director
company: Global Tech Solutions
location: London, UK
desc: Leading the transformation.
tags: AI | Strategy | Leadership
active: true
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.label, "2024");
        assert_eq!(e.title.as_deref(), Some("Executive Director"));
        assert_eq!(e.company.as_deref(), Some("Global Tech Solutions"));
        assert_eq!(e.location.as_deref(), Some("London, UK"));
        assert!(e.active);
        assert_eq!(e.tags.len(), 3);
        assert_eq!(e.tags[0], "AI");
    }

    #[test]
    fn test_parse_multiple_entries() {
        let input = r#"
{{label}}2024{{/label}}
{{card-start}}
#### Role A
company: Company A
{{card-end}}

{{label}}2021{{/label}}
{{card-start}}
#### Role B
company: Company B
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "2024");
        assert_eq!(entries[1].label, "2021");
    }

    #[test]
    fn test_parse_minimal_entry() {
        let input = r#"
{{label}}2020{{/label}}
{{card-start}}
#### Just a title
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title.as_deref(), Some("Just a title"));
        assert!(!entries[0].active);
    }

    #[test]
    fn test_compute_gap_label() {
        assert_eq!(
            compute_gap_label("2024", "2021"),
            Some("3 Years".to_string())
        );
        assert_eq!(
            compute_gap_label("2021", "2020"),
            Some("1 Year".to_string())
        );
        assert_eq!(
            compute_gap_label("2020", "2021"),
            Some("< 1 Year".to_string())
        );
        assert_eq!(compute_gap_label("Ancient", "Medieval"), None);
    }

    #[test]
    fn test_html_in_values_is_preserved() {
        let input = r#"
{{label}}<script>alert(1)</script>{{/label}}
{{card-start}}
#### <b>Bold Title</b>
company: Evil & Co.
tags: XSS | <img onerror=alert(1)>
{{card-end}}
"#;
        let entries = parse_entries(input);
        // Values are no longer escaped in parse_card — HTML from other
        // preprocessors (e.g. inplace-note) must be preserved.
        assert_eq!(entries[0].label, "<script>alert(1)</script>");
        assert_eq!(entries[0].title.as_deref(), Some("<b>Bold Title</b>"));
        assert_eq!(entries[0].company.as_deref(), Some("Evil & Co."));
        assert_eq!(entries[0].tags[0], "XSS");
        assert_eq!(entries[0].tags[1], "<img onerror=alert(1)>");
    }

    #[test]
    fn test_tags_pipe_separated() {
        let input = r#"
{{label}}2023{{/label}}
{{card-start}}
#### Role
tags: Rust | mdBook | Preprocessor
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(entries[0].tags, vec!["Rust", "mdBook", "Preprocessor"]);
    }

    #[test]
    fn test_md_inline_basic() {
        // bold, italics, strikethrough, inline code, links, underline HTML
        assert_eq!(md_inline("**bold**"), "<strong>bold</strong>");
        assert_eq!(md_inline("*italic*"), "<em>italic</em>");
        assert_eq!(md_inline("~~strike~~"), "<del>strike</del>");
        assert_eq!(md_inline("`code`"), "<code>code</code>");
        assert_eq!(
            md_inline("[site](https://example.com)"),
            r#"<a href="https://example.com">site</a>"#
        );
        // `<u>` is a raw inline HTML tag, so it passes straight through.
        assert_eq!(md_inline("<u>under</u>"), "<u>under</u>");
    }

    #[test]
    fn test_render_desc_markdown() {
        let entry = TimelineEntry {
            label: "2024".to_string(),
            title: Some("**Executive** Director".to_string()),
            company: Some("Global *Tech*".to_string()),
            location: Some("London, UK".to_string()),
            desc: Some(
                "Leading **the** transformation with *AI* and ~~legacy~~ systems.".to_string(),
            ),
            tags: vec!["AI".to_string()],
            active: true,
            images: Vec::new(),
        };
        let html = render_entry(&entry);
        assert!(html.contains("<h3><strong>Executive</strong> Director</h3>"));
        assert!(html.contains("Global <em>Tech</em>"));
        assert!(html.contains(
            "<p class=\"tl-desc\">Leading <strong>the</strong> transformation with <em>AI</em> \
             and <del>legacy</del> systems.</p>"
        ));
    }

    #[test]
    fn test_md_inline_preserves_html_ampersand() {
        // `&` should be escaped in the rendered HTML, not passed raw.
        assert_eq!(md_inline("Evil & Co."), "Evil &amp; Co.");
    }

    #[test]
    fn test_parse_multiline_desc_block() {
        let input = r#"
{{label}}2024{{/label}}
{{card-start}}
#### Director
company: Global Tech
desc:
    First paragraph with **bold**.
    Second line of the first paragraph.

    Second paragraph with *italic*.
tags: AI | Strategy | Leadership
active: true
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(
            e.desc.as_deref(),
            Some(
                "First paragraph with **bold**.\nSecond line of the first paragraph.\n\n\
                 Second paragraph with *italic*."
            )
        );
        // Fields after the block must still be detected.
        assert_eq!(e.tags, vec!["AI", "Strategy", "Leadership"]);
        assert!(e.active);
        assert_eq!(e.company.as_deref(), Some("Global Tech"));
    }

    #[test]
    fn test_parse_multiline_desc_keeps_subsequent_fields() {
        // Fields after an indented desc block must not be swallowed.
        let input = r#"
{{label}}2024{{/label}}
{{card-start}}
#### Director
desc:
    Only paragraph here.
company: After Block
location: Paris
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(entries[0].desc.as_deref(), Some("Only paragraph here."));
        assert_eq!(entries[0].company.as_deref(), Some("After Block"));
        assert_eq!(entries[0].location.as_deref(), Some("Paris"));
    }

    #[test]
    fn test_parse_desc_mixed_same_line_and_block() {
        // A value on the `desc:` line plus indented continuation lines.
        let input = r#"
{{label}}2024{{/label}}
{{card-start}}
#### Director
desc: Opening sentence.
    Continued on the next indented line.
{{card-end}}
"#;
        let entries = parse_entries(input);
        assert_eq!(
            entries[0].desc.as_deref(),
            Some("Opening sentence.\nContinued on the next indented line.")
        );
    }

    #[test]
    fn test_render_desc_paragraphs() {
        // Single paragraph stays a `<p class="tl-desc">`.
        let single = render_desc_html("Just **one** paragraph.");
        assert_eq!(
            single,
            "<p class=\"tl-desc\">Just <strong>one</strong> paragraph.</p>"
        );

        // Multiple paragraphs wrap in `<div class="tl-desc">`.
        let multi = render_desc_html("First paragraph.\n\nSecond paragraph.");
        assert!(multi.starts_with("<div class=\"tl-desc\">"));
        assert!(multi.contains("<p>First paragraph.</p>"));
        assert!(multi.contains("<p>Second paragraph.</p>"));
        assert!(multi.ends_with("</div>"));
    }

    #[test]
    fn test_join_desc_block() {
        assert_eq!(join_desc_block(vec![]), "");
        assert_eq!(join_desc_block(vec!["a".into(), "b".into()]), "a\nb");
        assert_eq!(
            join_desc_block(vec!["a".into(), "".into(), "b".into()]),
            "a\n\nb"
        );
        // Leading/trailing blanks and run-on blanks are dropped.
        assert_eq!(
            join_desc_block(vec!["".into(), "a".into(), "".into(), "".into(), "b".into(), "".into()]),
            "a\n\nb"
        );
    }
}
