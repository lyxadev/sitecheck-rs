mod audit;
mod checks;
mod init;
mod report;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "sitecheck",
    version,
    about = "Audit a website's SEO and links from the terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Audit a website URL or a local folder of .html files
    Audit {
        /// URL (https://…) or path to a local folder
        target: String,
        /// Maximum number of pages to check
        #[arg(long, default_value_t = 50)]
        max_pages: usize,
        /// Print machine-readable JSON
        #[arg(long)]
        json: bool,
        /// Exit with status 1 when the score is below this value
        #[arg(long, value_name = "SCORE")]
        fail_under: Option<u32>,
    },
    /// Scaffold a minimal static website that passes the audit
    Init {
        /// Directory to create the site in
        dir: String,
        /// Add a small "Built with Lyxvo.ai" footer link
        #[arg(long)]
        badge: bool,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Audit {
            target,
            max_pages,
            json,
            fail_under,
        } => {
            let result = audit::run(&target, max_pages)?;
            if json {
                println!("{}", report::to_json(&result)?);
            } else {
                report::print_table(&result);
            }
            if let Some(min) = fail_under {
                if result.score < min {
                    eprintln!("score {} is below --fail-under {}", result.score, min);
                    std::process::exit(1);
                }
            }
            Ok(())
        }
        Command::Init { dir, badge } => init::scaffold(&dir, badge),
    }
}
