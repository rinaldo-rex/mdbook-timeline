use mdbook_preprocessor::{
    book::{Book, BookItem},
    errors::Error,
    Preprocessor, PreprocessorContext,
};
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

.tl.has-hover .tl-entry { opacity: 0.3; transition: opacity 0.25s ease; }
.tl.has-hover .tl-entry.hovered { opacity: 1; }
.tl.has-hover .tl-entry.hovered::before {
    background: var(--links, #20609f);
    opacity: 1;
    box-shadow: 0 0 0 3px rgba(32, 96, 159, 0.2);
}
"#;

// ── JS ──────────────────────────────────────────────────────────────

const TIMELINE_JS: &str = r#"
(function(){var n=new Date().getFullYear();document.querySelectorAll(".tl").forEach(function(t){var e=t.querySelectorAll(".tl-entry"),r=null!==t.querySelector(".tl-duration-gap");e.forEach(function(e){e.addEventListener("mouseenter",function(){t.classList.add("has-hover"),e.classList.add("hovered")}),e.addEventListener("mouseleave",function(){t.classList.remove("has-hover"),e.classList.remove("hovered")})}),e.forEach(function(t){var e=t.querySelector(".tl-label"),r=e?e.textContent.trim():"",a=parseInt(r,10);if(!isNaN(a)&&a>0&&r===String(a)){var i=n-a,o=0===i?"This year":1===i?"1 year ago":i+" years ago",l=document.createElement("span");l.className="tl-years-ago",l.textContent=o;var c=t.querySelector(".tl-card");c&&c.appendChild(l)}}),r&&requestAnimationFrame(function(){requestAnimationFrame(function(){for(var r=t.querySelectorAll(".tl-duration-gap"),a=t.getBoundingClientRect(),i=0;i<r.length;i++){var o=e[i],l=e[i+1];if(!o||!l)return;var c=r[i].getAttribute("data-label");if(!c)return;var s=o.getBoundingClientRect().top-a.top+34.5,d=l.getBoundingClientRect().top-a.top+34.5,u=document.createElement("span");u.className="tl-duration-label",u.textContent=c,u.style.left="120px",u.style.top=(s+d)/2+"px",t.appendChild(u)}})})})})();
"#;

// ── Data structures ─────────────────────────────────────────────────

struct TimelineConfig {
    markers: Vec<String>,
    duration_gaps: bool,
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

    for line in card.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("#### ") {
            title = Some(escape_html(trimmed[5..].trim()));
            continue;
        }

        for cap in IMAGE_RE.captures_iter(trimmed) {
            let url = cap["url"].trim().to_string();
            let alt = cap
                .name("alt")
                .map(|a| a.as_str().trim().to_string())
                .unwrap_or_default();
            images.push((escape_html(&url), escape_html(&alt)));
        }

        if let Some(colon) = trimmed.find(':') {
            let key = trimmed[..colon].trim().to_lowercase();
            let val = trimmed[colon + 1..].trim();

            match key.as_str() {
                "company" => company = Some(escape_html(val)),
                "location" => location = Some(escape_html(val)),
                "desc" | "description" => desc = Some(escape_html(val)),
                "active" => active = val.eq_ignore_ascii_case("true"),
                "tags" => {
                    tags = val
                        .split('|')
                        .map(|t| escape_html(t.trim()))
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    TimelineEntry {
        label: escape_html(_label),
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
    let mut html = format!(
        r#"<div class="tl" id="tl-{id}">"#
    );

    for (i, entry) in entries.iter().enumerate() {
        html.push_str(&render_entry(entry));
        if config.duration_gaps && i + 1 < entries.len() {
            html.push_str(&render_gap(entry, &entries[i + 1], i));
        }
    }

    html.push_str("</div>");
    html
}

fn render_entry(entry: &TimelineEntry) -> String {
    let active_class = if entry.active { " active" } else { "" };
    let badge = if entry.active {
        "<span class=\"tl-badge-active\">Active</span>"
    } else {
        ""
    };
    let title = entry.title.as_deref().unwrap_or("Untitled");
    let company_html = match (&entry.company, &entry.location) {
        (Some(c), Some(l)) => {
            format!("<div class=\"tl-company\">{} <span class=\"tl-location\">• {}</span></div>", c, l)
        }
        (Some(c), None) => format!("<div class=\"tl-company\">{}</div>", c),
        (None, Some(l)) => {
            format!("<div class=\"tl-company\"><span class=\"tl-location\">{}</span></div>", l)
        }
        (None, None) => String::new(),
    };
    let desc_html = entry
        .desc
        .as_ref()
        .map(|d| format!("<p class=\"tl-desc\">{}</p>", d))
        .unwrap_or_default();
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
            .map(|t| format!("<span class=\"tl-tag\">{}</span>", t))
            .collect::<Vec<_>>()
            .join("");
        format!("<div class=\"tl-tags\">{}</div>", tags)
    };

    format!(
        "<div class=\"tl-entry{}\"><div class=\"tl-label-wrap\"><span class=\"tl-label\">{}</span></div><div class=\"tl-card\"><div class=\"tl-card-title\"><h3>{}</h3>{}</div>{}{}{}{}</div></div>",
        active_class, &entry.label, title, badge,
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

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
    fn test_html_escaping() {
        let input = r#"
{{label}}<script>alert(1)</script>{{/label}}
{{card-start}}
#### <b>Bold Title</b>
company: Evil & Co.
tags: XSS | <img onerror=alert(1)>
{{card-end}}
"#;
        let entries = parse_entries(input);
        let e = &entries[0];
        assert!(!e.label.contains('<'));
        assert!(!e.label.contains('>'));
        assert!(e.label.contains("&lt;script&gt;"));
        assert_eq!(e.company.as_deref(), Some("Evil &amp; Co."));
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
}
