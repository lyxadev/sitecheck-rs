use crate::checks::{self, Issue, Page};
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Serialize)]
pub struct PageReport {
    pub url: String,
    pub score: u32,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Serialize)]
pub struct AuditResult {
    pub target: String,
    pub pages: Vec<PageReport>,
    pub score: u32,
}

/// Source of pages: either a live site or a local folder.
enum Source {
    Remote {
        base: Url,
        client: reqwest::blocking::Client,
    },
    Local {
        root: PathBuf,
    },
}

pub fn run(target: &str, max_pages: usize) -> Result<AuditResult> {
    let (source, start) = if target.starts_with("http://") || target.starts_with("https://") {
        let base = Url::parse(target)?;
        let client = reqwest::blocking::Client::builder()
            .user_agent("sitecheck/0.1 (+https://github.com/lyxadev/sitecheck-rs)")
            .timeout(std::time::Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;
        (
            Source::Remote {
                base: base.clone(),
                client,
            },
            "/".to_string(),
        )
    } else {
        let root = PathBuf::from(target);
        if !root.is_dir() {
            return Err(anyhow!("'{target}' is not a directory or URL"));
        }
        (Source::Local { root }, "/index.html".to_string())
    };

    let mut queue: VecDeque<String> = VecDeque::from([start]);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut pages: Vec<(String, Page)> = Vec::new();
    let mut broken: Vec<(String, String)> = Vec::new();

    while let Some(path) = queue.pop_front() {
        if pages.len() >= max_pages || !seen.insert(path.clone()) {
            continue;
        }
        match load(&source, &path) {
            Ok(Some(html)) => {
                let page = checks::parse(&html);
                for link in &page.links {
                    if let Some(next) = normalise(link, &path) {
                        if !seen.contains(&next) {
                            queue.push_back(next);
                        }
                    }
                }
                pages.push((path, page));
            }
            Ok(None) => broken.push((path.clone(), "not found".into())),
            Err(e) => broken.push((path.clone(), e.to_string())),
        }
    }

    let titles: Vec<(String, Option<String>)> = pages
        .iter()
        .map(|(p, pg)| (p.clone(), pg.title.clone()))
        .collect();
    let dup = checks::check_duplicate_titles(&titles);

    let mut reports = Vec::new();
    for (path, page) in &pages {
        let mut issues = Vec::new();
        issues.extend(checks::check_title(page.title.as_deref()));
        issues.extend(checks::check_description(page.description.as_deref()));
        issues.extend(checks::check_h1(page.h1_count));
        issues.extend(checks::check_lang(page.lang.as_deref()));
        issues.extend(checks::check_canonical(page.canonical.as_deref()));
        issues.extend(checks::check_viewport(page.viewport));
        issues.extend(checks::check_alt(page.images_without_alt));
        issues.extend(
            dup.iter()
                .filter(|(p, _)| p == path)
                .map(|(_, i)| i.clone()),
        );
        for link in &page.links {
            if let Some(target) = normalise(link, path) {
                if let Some((_, reason)) = broken.iter().find(|(p, _)| *p == target) {
                    issues.push(Issue {
                        code: "broken-link",
                        message: format!("broken internal link to {target} ({reason})"),
                    });
                }
            }
        }
        reports.push(PageReport {
            url: display_url(&source, path),
            score: page_score(&issues),
            issues,
        });
    }

    let score = if reports.is_empty() {
        0
    } else {
        reports.iter().map(|r| r.score).sum::<u32>() / reports.len() as u32
    };

    Ok(AuditResult {
        target: target.to_string(),
        pages: reports,
        score,
    })
}

/// Load a page body. Ok(None) means not found.
fn load(source: &Source, path: &str) -> Result<Option<String>> {
    match source {
        Source::Local { root } => {
            let file = root.join(path.trim_start_matches('/'));
            if file.is_file() {
                Ok(Some(fs::read_to_string(file)?))
            } else {
                Ok(None)
            }
        }
        Source::Remote { base, client } => {
            let url = base.join(path)?;
            let resp = client.get(url).send()?;
            let status = resp.status();
            if status.as_u16() == 404 {
                return Ok(None);
            }
            if status.is_client_error() || status.is_server_error() {
                return Err(anyhow!("HTTP {}", status.as_u16()));
            }
            Ok(Some(resp.text()?))
        }
    }
}

/// Turn an href into a same-origin path, or None if external/non-page.
fn normalise(href: &str, current: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
        || href.starts_with("http://")
        || href.starts_with("https://")
    {
        return None;
    }
    let path = href.split(['#', '?']).next().unwrap_or("");
    if path.is_empty() {
        return Some(current.to_string());
    }
    if path.starts_with('/') {
        Some(path.to_string())
    } else {
        let base_dir = match current.rfind('/') {
            Some(i) => &current[..=i],
            None => "/",
        };
        Some(format!("{base_dir}{path}"))
    }
}

fn display_url(source: &Source, path: &str) -> String {
    match source {
        Source::Remote { base, .. } => base
            .join(path)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| path.to_string()),
        Source::Local { .. } => path.to_string(),
    }
}

/// Per-page score: starts at 100, each issue costs a fixed weight.
fn page_score(issues: &[Issue]) -> u32 {
    let penalty: u32 = issues
        .iter()
        .map(|i| match i.code {
            "title-missing" | "title-length" => 15,
            "description-missing" | "description-length" => 10,
            "h1-missing" | "h1-multiple" => 10,
            "lang-missing" => 5,
            "canonical-missing" => 5,
            "viewport-missing" => 10,
            "img-alt-missing" => 5,
            "title-duplicate" => 5,
            "broken-link" => 10,
            _ => 5,
        })
        .sum();
    100u32.saturating_sub(penalty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_relative_links() {
        assert_eq!(
            normalise("about.html", "/index.html").as_deref(),
            Some("/about.html")
        );
        assert_eq!(
            normalise("/contact.html#x", "/a/b.html").as_deref(),
            Some("/contact.html")
        );
        assert_eq!(normalise("https://other.test/", "/"), None);
        assert_eq!(normalise("#top", "/"), None);
        assert_eq!(normalise("mailto:a@b.c", "/"), None);
    }

    #[test]
    fn perfect_page_scores_100() {
        assert_eq!(page_score(&[]), 100);
        assert_eq!(
            page_score(&[Issue {
                code: "lang-missing",
                message: String::new()
            }]),
            95
        );
    }
}
