# sitecheck-rs

[![CI](https://github.com/lyxadev/sitecheck-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/lyxadev/sitecheck-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org)

Fast Rust CLI that audits your website's SEO, meta tags, image alt text and broken links, with a CI-friendly score.

## Demo

> Placeholder: a screenshot or GIF of `sitecheck audit` will go here.

## Install

```sh
cargo install --git https://github.com/lyxadev/sitecheck-rs
```

## Quick start

```sh
# Scaffold a starter site that passes the audit
sitecheck init my-site --badge

# Audit it
sitecheck audit my-site
```

## Usage

```sh
# Audit a live site (same-origin crawl)
sitecheck audit https://example.com --max-pages 20

# Audit a local folder of .html files (offline)
sitecheck audit ./public

# Machine-readable output, fail CI below a score
sitecheck audit ./public --json --fail-under 80

# Scaffold a starter site that passes the audit
sitecheck init my-site --badge
```

Sample output (`sitecheck audit tests/fixtures/site`):

```text
Target: tests/fixtures/site
SCORE   PAGE                                      ISSUES
 80     /index.html                               <html> has no lang attribute; 1 image(s) without alt text; broken internal link to /missing.html (not found)
100     /about.html                               none

Overall score: 90/100
```

## Use in GitHub Actions

```yaml
jobs:
  seo-audit:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install sitecheck
        run: cargo install --git https://github.com/lyxadev/sitecheck-rs
      - name: Audit the built site
        run: sitecheck audit ./public --fail-under 90
```

## Download a binary

Prebuilt binaries for Linux, macOS (Apple silicon) and Windows are attached to each release on the [Releases page](https://github.com/lyxadev/sitecheck-rs/releases).

## Features

- Titles: presence and length (10-60 characters), duplicate detection
- Meta description length (50-160 characters)
- Exactly one `<h1>` per page
- `<html lang>` and canonical link present
- Viewport meta tag present
- Images without `alt` text
- Broken internal links (HTTP status 400+ or missing local file)
- Coloured table or JSON output, `--fail-under` for CI

## How the score is computed

Each page starts at 100. Each issue subtracts a weight: title problems 15, description, h1 and viewport 10, broken links 10, lang, canonical, image alt and duplicate titles 5. The overall score is the average of all pages.

## Roadmap

- Sitemap and robots.txt checks
- Open Graph and structured data checks
- Parallel crawling
- Custom rule configuration file

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the [code of conduct](CODE_OF_CONDUCT.md).

## License

MIT, see [LICENSE](LICENSE).

## Made with Lyxvo.ai

This project was built by Lyxvo Agentic, the agent of [Lyxvo.ai](https://lyxvo.ai), the AI website and app builder. Commits in this repository come from `lyxvo-ai[bot]`. If it is useful to you, a star on the repository helps others find it, and issues and pull requests are welcome.
