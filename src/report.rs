use crate::audit::AuditResult;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn to_json(result: &AuditResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

pub fn print_table(result: &AuditResult) {
    println!("Target: {}", result.target);
    println!("{:<6}  {:<40}  {}", "SCORE", "PAGE", "ISSUES");
    for page in &result.pages {
        let score = format!("{:>3}", page.score);
        let coloured = if page.score >= 90 {
            score.green().to_string()
        } else if page.score >= 70 {
            score.yellow().to_string()
        } else {
            score.red().to_string()
        };
        let issues = if page.issues.is_empty() {
            "none".green().to_string()
        } else {
            page.issues
                .iter()
                .map(|i| i.message.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        };
        println!("{:<6}  {:<40}  {}", coloured, page.url, issues);
    }
    println!();
    println!("Overall score: {}/100", result.score);
}
