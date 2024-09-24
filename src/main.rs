use anyhow::{anyhow, bail, Context, Result};
use atty::Stream;
use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};

/// Represents a saved pattern with optional flags, patterns, and engine.
#[derive(Serialize, Deserialize)]
struct Pattern {
    flags: Option<String>,
    pattern: Option<String>,
    patterns: Option<Vec<String>>,
    engine: Option<String>,
}

/// Command-line interface definition using clap.
#[derive(Parser)]
#[command(
    name = "gf",
    about = "Pattern manager for grep-like tools",
    version = "1.0.0"
)]
struct Cli {
    /// Save a pattern (e.g., gf --save pat-name -Hnri 'search-pattern')
    #[arg(long)]
    save: bool,

    /// List available patterns
    #[arg(long)]
    list: bool,

    /// Print the command rather than executing it
    #[arg(long)]
    dump: bool,

    /// The name of the pattern (when saving or using)
    name: Option<String>,

    /// Specify the engine to use (e.g., 'grep', 'rg', 'ag')
    #[arg(long)]
    engine: Option<String>,

    /// Additional arguments
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {}", "Error:".bright_red().bold(), err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.list {
        let patterns = get_patterns().context("Failed to list patterns")?;
        for pat in patterns {
            println!("{}", pat);
        }
        return Ok(());
    }

    if cli.save {
        let name = cli
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Name cannot be empty"))?;
        let flags = cli.args.first().map(|s| s.to_string()).unwrap_or_default();
        let pattern = cli
            .args
            .get(1)
            .ok_or_else(|| anyhow!("Pattern cannot be empty"))?;

        save_pattern(name, &flags, pattern, cli.engine.clone())?;
        return Ok(());
    }

    // Use the pattern
    let pat_name = cli
        .name
        .as_ref()
        .ok_or_else(|| anyhow!("Pattern name is required"))?;
    let files = cli.args.first().map(|s| s.as_str()).unwrap_or(".");

    let pattern_dir = get_pattern_dir().context("Unable to open user's pattern directory")?;
    let filename = pattern_dir.join(format!("{}.json", pat_name));

    let f = fs::File::open(&filename).with_context(|| format!("No such pattern '{}'", pat_name))?;

    let pat: Pattern = serde_json::from_reader(f)
        .with_context(|| format!("Pattern file '{}' is malformed", filename.display()))?;

    let pattern_str = if let Some(pat_pattern) = pat.pattern {
        pat_pattern
    } else if let Some(pat_patterns) = pat.patterns {
        if pat_patterns.is_empty() {
            bail!(
                "Pattern file '{}' contains no pattern(s)",
                filename.display()
            );
        }
        format!("({})", pat_patterns.join("|"))
    } else {
        bail!(
            "Pattern file '{}' contains no pattern(s)",
            filename.display()
        );
    };

    let operator = pat.engine.clone().unwrap_or_else(|| "grep".to_string());

    if cli.dump {
        // Use the operator instead of hardcoding "grep"
        let mut command = format!("{} ", operator);

        if let Some(flags) = &pat.flags {
            command.push_str(flags);
            command.push(' ');
        }

        command.push_str(&format!("{:?} {}", pattern_str, files));

        println!("{}", command);
    } else {
        let stdin_is_pipe = stdin_is_pipe();

        let mut cmd = ProcessCommand::new(operator);

        if let Some(flags) = pat.flags {
            cmd.args(flags.split_whitespace());
        }

        cmd.arg(pattern_str);

        if !stdin_is_pipe {
            cmd.arg(files);
        }

        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let status = cmd.status().context("Failed to execute command")?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Ok(())
}

/// Determines the pattern directory in the user's home directory.
fn get_pattern_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    let config_dir = home_dir.join(".config/gf");

    if config_dir.exists() {
        return Ok(config_dir);
    }

    let gf_dir = home_dir.join(".gf");

    Ok(gf_dir)
}

/// Saves a new pattern to the pattern directory.
fn save_pattern(name: &str, flags: &str, pat: &str, engine: Option<String>) -> Result<()> {
    if name.is_empty() {
        bail!("Name cannot be empty");
    }
    if pat.is_empty() {
        bail!("Pattern cannot be empty");
    }

    let p = Pattern {
        flags: if flags.is_empty() {
            None
        } else {
            Some(flags.to_string())
        },
        pattern: Some(pat.to_string()),
        patterns: None,
        engine,
    };

    let pattern_dir = get_pattern_dir().context("Failed to determine pattern directory")?;

    fs::create_dir_all(&pattern_dir).context("Failed to create pattern directory")?;

    let path = pattern_dir.join(format!("{}.json", name));

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true).mode(0o666);

    let f = options.open(&path).with_context(|| {
        format!(
            "Failed to create pattern file '{}': file may already exist",
            path.display()
        )
    })?;

    serde_json::to_writer_pretty(f, &p).context("Failed to write pattern file")?;

    Ok(())
}

/// Retrieves a list of saved pattern names.
fn get_patterns() -> Result<Vec<String>> {
    let mut out = Vec::new();

    let pattern_dir = get_pattern_dir().context("Failed to determine pattern directory")?;

    if !pattern_dir.exists() {
        // If the pattern directory doesn't exist, return an empty list
        return Ok(out);
    }

    let entries = fs::read_dir(&pattern_dir).context("Failed to read pattern directory")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                out.push(filename.to_string());
            }
        }
    }

    Ok(out)
}

/// Checks if stdin is a pipe.
fn stdin_is_pipe() -> bool {
    !atty::is(Stream::Stdin)
}
