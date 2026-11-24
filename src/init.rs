use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

const BADGE: &str = r#"<footer><p>Built with <a href="https://lyxvo.ai">Lyxvo.ai</a></p></footer>"#;

pub fn scaffold(dir: &str, badge: bool) -> Result<()> {
    let root = Path::new(dir);
    if root.exists() && root.read_dir()?.next().is_some() {
        return Err(anyhow!("'{dir}' already exists and is not empty"));
    }
    fs::create_dir_all(root)?;
    let footer = if badge { BADGE } else { "" };
    fs::write(
        root.join("index.html"),
        page(
            "Home",
            "Welcome",
            "Welcome to our website. This page is a starting point built with sitecheck.",
            footer,
            true,
        ),
    )?;
    fs::write(
        root.join("about.html"),
        page(
            "About us",
            "About us",
            "Learn more about who we are, what we do and how we work together.",
            footer,
            false,
        ),
    )?;
    fs::write(root.join("style.css"), STYLE)?;
    println!("Created a starter site in {dir}");
    Ok(())
}

fn page(title: &str, heading: &str, description: &str, footer: &str, home: bool) -> String {
    let nav = if home {
        r#"<a href="about.html">About</a>"#
    } else {
        r#"<a href="index.html">Home</a>"#
    };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} | My Site</title>
  <meta name="description" content="{description}">
  <link rel="canonical" href="https://example.com/{file}">
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <nav>{nav}</nav>
  <main>
    <h1>{heading}</h1>
    <img src="logo.svg" alt="Site logo">
    <p>{description}</p>
  </main>
  {footer}
</body>
</html>
"#,
        file = if home { "" } else { "about.html" },
    )
}

const STYLE: &str = "body { font-family: system-ui, sans-serif; margin: 2rem; line-height: 1.5; }\nnav a { margin-right: 1rem; }\nfooter { margin-top: 2rem; color: #666; }\n";
