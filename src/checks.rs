use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Issue {
    pub code: &'static str,
    pub message: String,
}

/// Extracted facts about one HTML page.
#[derive(Debug, Default, Clone)]
pub struct Page {
    pub title: Option<String>,
    pub description: Option<String>,
    pub h1_count: usize,
    pub lang: Option<String>,
    pub canonical: Option<String>,
    pub viewport: bool,
    pub images_without_alt: usize,
    /// Internal links (href values, unresolved).
    pub links: Vec<String>,
}

pub fn parse(html: &str) -> Page {
    let doc = Html::parse_document(html);
    let sel = |s: &str| Selector::parse(s).expect("valid selector");

    let title = doc
        .select(&sel("title"))
        .next()
        .map(|t| t.text().collect::<String>().trim().to_string());

    let description = doc
        .select(&sel(r#"meta[name="description"]"#))
        .next()
        .and_then(|m| m.value().attr("content"))
        .map(|s| s.trim().to_string());

    let h1_count = doc.select(&sel("h1")).count();

    let lang = doc
        .select(&sel("html"))
        .next()
        .and_then(|h| h.value().attr("lang"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let canonical = doc
        .select(&sel(r#"link[rel="canonical"]"#))
        .next()
        .and_then(|l| l.value().attr("href"))
        .map(|s| s.trim().to_string());

    let viewport = doc
        .select(&sel(r#"meta[name="viewport"]"#))
        .next()
        .is_some();

    let images_without_alt = doc
        .select(&sel("img"))
        .filter(|img| img.value().attr("alt").is_none())
        .count();

    let links = doc
        .select(&sel("a[href]"))
        .filter_map(|a| a.value().attr("href"))
        .map(|s| s.to_string())
        .collect();

    Page {
        title,
        description,
        h1_count,
        lang,
        canonical,
        viewport,
        images_without_alt,
        links,
    }
}

pub fn check_title(title: Option<&str>) -> Vec<Issue> {
    match title {
        None => vec![issue("title-missing", "missing <title>")],
        Some("") => vec![issue("title-missing", "empty <title>")],
        Some(t) => {
            let len = t.chars().count();
            if (10..=60).contains(&len) {
                vec![]
            } else {
                vec![issue(
                    "title-length",
                    &format!("title is {len} chars (expected 10-60)"),
                )]
            }
        }
    }
}

pub fn check_description(desc: Option<&str>) -> Vec<Issue> {
    match desc {
        None => vec![issue("description-missing", "missing meta description")],
        Some(d) => {
            let len = d.chars().count();
            if (50..=160).contains(&len) {
                vec![]
            } else {
                vec![issue(
                    "description-length",
                    &format!("meta description is {len} chars (expected 50-160)"),
                )]
            }
        }
    }
}

pub fn check_h1(count: usize) -> Vec<Issue> {
    match count {
        1 => vec![],
        0 => vec![issue("h1-missing", "no <h1> heading")],
        n => vec![issue(
            "h1-multiple",
            &format!("{n} <h1> headings (expected 1)"),
        )],
    }
}

pub fn check_lang(lang: Option<&str>) -> Vec<Issue> {
    match lang {
        Some(_) => vec![],
        None => vec![issue("lang-missing", "<html> has no lang attribute")],
    }
}

pub fn check_canonical(canonical: Option<&str>) -> Vec<Issue> {
    match canonical {
        Some(_) => vec![],
        None => vec![issue("canonical-missing", "missing canonical link")],
    }
}

pub fn check_viewport(present: bool) -> Vec<Issue> {
    if present {
        vec![]
    } else {
        vec![issue("viewport-missing", "missing viewport meta tag")]
    }
}

pub fn check_alt(images_without_alt: usize) -> Vec<Issue> {
    if images_without_alt == 0 {
        vec![]
    } else {
        vec![issue(
            "img-alt-missing",
            &format!("{images_without_alt} image(s) without alt text"),
        )]
    }
}

pub fn check_duplicate_titles(titles: &[(String, Option<String>)]) -> Vec<(String, Issue)> {
    let mut out = Vec::new();
    for (page, title) in titles {
        let Some(t) = title else { continue };
        let dups = titles
            .iter()
            .filter(|(_, other)| other.as_deref() == Some(t.as_str()))
            .count();
        if dups > 1 {
            out.push((
                page.clone(),
                issue(
                    "title-duplicate",
                    &format!("title shared with {} pages", dups - 1),
                ),
            ));
        }
    }
    out
}

fn issue(code: &'static str, message: &str) -> Issue {
    Issue {
        code,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_page_facts() {
        let html = r#"<html lang="en"><head><title>Hello there</title>
            <meta name="viewport" content="width=device-width">
            <link rel="canonical" href="https://x.test/">
            </head><body><h1>A</h1><img src="a.png"><img src="b.png" alt="b">
            <a href="/about.html">about</a></body></html>"#;
        let p = parse(html);
        assert_eq!(p.title.as_deref(), Some("Hello there"));
        assert_eq!(p.h1_count, 1);
        assert_eq!(p.lang.as_deref(), Some("en"));
        assert!(p.viewport);
        assert_eq!(p.images_without_alt, 1);
        assert_eq!(p.links, vec!["/about.html".to_string()]);
    }

    #[test]
    fn title_checks() {
        assert!(check_title(Some("A good title here")).is_empty());
        assert_eq!(check_title(None)[0].code, "title-missing");
        assert_eq!(check_title(Some("Hi"))[0].code, "title-length");
    }

    #[test]
    fn description_checks() {
        let good = "x".repeat(80);
        assert!(check_description(Some(&good)).is_empty());
        assert_eq!(check_description(None)[0].code, "description-missing");
        assert_eq!(
            check_description(Some("short"))[0].code,
            "description-length"
        );
    }

    #[test]
    fn h1_checks() {
        assert!(check_h1(1).is_empty());
        assert_eq!(check_h1(0)[0].code, "h1-missing");
        assert_eq!(check_h1(2)[0].code, "h1-multiple");
    }

    #[test]
    fn duplicate_titles_are_flagged() {
        let pages = vec![
            ("a.html".to_string(), Some("Same title here".to_string())),
            ("b.html".to_string(), Some("Same title here".to_string())),
            ("c.html".to_string(), Some("Unique title here".to_string())),
        ];
        let dups = check_duplicate_titles(&pages);
        assert_eq!(dups.len(), 2);
        assert!(dups.iter().all(|(_, i)| i.code == "title-duplicate"));
    }
}
