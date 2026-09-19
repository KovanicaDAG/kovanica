---
name: rust-cli
description: Use when building CLI tools in Rust: clap argument parsing, dialoguer prompts, console styling, indicatif progress bars, terminal handling, and common CLI patterns (subcommands, flags, config files, completions).
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, CLI, clap, dialoguer, console, indicatif, terminal, subcommands]
    related_skills: [rust-basics, rust-error-handling, rust-logging-tracing]
---

# Rust CLI

## Overview

Rust is a popular language for CLI tools because it compiles to a single static binary, has no runtime dependency, and has a strong ecosystem for argument parsing, terminal interaction, and output formatting. The dominant argument parser is **clap** (with `derive` API), and common companion crates are **dialoguer** (prompts), **console** (styling, measuring), **indicatif** (progress bars, spinners), and **clap_complete** (shell completions).

## When to Use

- Building a command-line tool (single binary or suite of subcommands)
- Parsing arguments, flags, options, subcommands
- Asking the user for input (confirmation, selection, input, password)
- Showing progress bars, spinners, or status messages
- Styling output (colors, bold, italics, tables)
- Generating shell completions (bash, zsh, fish, powershell)

**Don't use for:** TUI apps with full-screen UI (use `ratatui` or `crossterm` directly), or web-based CLIs.

## Clap: Argument Parsing

### Derive API (idiomatic)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(name = "mytool", about = "A sample CLI tool", version)]
struct Cli {
    /// Optional config file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Verbose mode (can be used multiple times: -v, -vv, -vvv)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Subcommands
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new item
    Add(AddCmd),
    /// List items
    List(ListCmd),
    /// Remove an item by ID
    Remove(RemoveCmd),
}

#[derive(Args, Debug)]
struct AddCmd {
    /// Item name
    name: String,

    /// Item category
    #[arg(short, long, default_value = "general")]
    category: String,

    /// Force overwrite if item exists
    #[arg(short, long)]
    force: bool,
}

#[derive(Args, Debug)]
struct ListCmd {
    /// Filter by category
    #[arg(short, long)]
    category: Option<String>,

    /// Limit number of results
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

#[derive(Args, Debug)]
struct RemoveCmd {
    /// ID of item to remove
    id: String,

    /// Confirm before removing
    #[arg(short, long)]
    confirm: bool,
}

fn main() {
    let cli = Cli::parse();

    let verbosity = cli.verbose;
    if verbosity > 0 {
        println!("Verbose mode (level {})", verbosity);
    }

    match cli.command {
        Commands::Add(cmd) => {
            println!("Adding {} in category {}", cmd.name, cmd.category);
            if cmd.force {
                println!("Force mode enabled");
            }
        }
        Commands::List(cmd) => {
            if let Some(cat) = cmd.category {
                println!("Listing items in category {}", cat);
            } else {
                println!("Listing all items");
            }
            println!("Limit: {}", cmd.limit);
        }
        Commands::Remove(cmd) => {
            if cmd.confirm {
                println!("Confirm removal of {}", cmd.id);
            } else {
                println!("Removing {}", cmd.id);
            }
        }
    }
}
```

**Derive API features used:**
- `#[derive(Parser)]` on the top-level struct
- `#[command(name, about, version)]` for metadata
- `#[arg(short, long)]` for flags/options
- `#[command(subcommand)]` for subcommands
- `#[arg(action = ArgAction::Count)]` for repeatable flags
- `#[arg(default_value = "...")]` / `#[arg(default_value_t = ...)]` for defaults
- `Option<T>` for optional arguments, `T` for required ones

### Builder API

For dynamic argument construction (when you can't use derive, e.g., building args procedurally):

```rust
use clap::{Command, Arg};

let cli = Command::new("mytool")
    .about("A sample CLI tool")
    .arg(
        Arg::new("config")
            .short('c')
            .long("config")
            .value_name("PATH")
            .help("Config file path")
    )
    .arg(
        Arg::new("verbose")
            .short('v')
            .action(clap::ArgAction::Count)
            .help("Increase verbosity")
    )
    .subcommand(
        Command::new("add")
            .about("Add a new item")
            .arg(Arg::new("name").required(true))
            .arg(Arg::new("category").short('C').long("category"))
    );

let matches = cli.get_matches();
let config = matches.get_one::<String>("config").map(|s| s.as_str());
let verbose = matches.get_count("verbose");
if let Some(sub) = matches.subcommand() {
    match sub {
        ("add", sub_matches) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            // ...
        }
        _ => {}
    }
}
```

Prefer derive when possible — it's self-documenting, generates help text automatically, and is less error-prone.

### Argument Types

| Type | Example | Behavior |
|---|---|---|
| `bool` | `#[arg(short, long)] verbose: bool` | flag, true if present |
| `Option<T>` | `#[arg(short, long)] config: Option<String>` | optional, None if absent |
| `T` | `#[arg(short, long)] name: String` | required, error if missing |
| `Vec<T>` | `#[arg(short, long)] files: Vec<String>` | multiple values (e.g. `-f a -f b`) |
| `u8` with `ArgAction::Count` | `#[arg(short, long, action = ArgAction::Count)] verbose: u8` | counts occurrences (-v = 1, -vv = 2) |
| `num` with `default_value_t` | `#[arg(long, default_value_t = 10)] limit: usize` | default value |

### Value Hints and Validation

```rust
#[derive(Args, Debug)]
struct Cli {
    /// Port number (1024-65535)
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1024..=65535))]
    port: u16,

    /// Log level (debug, info, warn, error)
    #[arg(long, value_parser = ["debug", "info", "warn", "error"])]
    log_level: String,
}
```

`value_parser` lets you use any function or closure that converts a `&str` to the target type and returns `Result<T, String>`.

### Environment Variables

```rust
#[derive(Parser, Debug)]
struct Cli {
    /// API key
    #[arg(long, env = "API_KEY")]
    api_key: String,

    /// Timeout in seconds
    #[arg(long, env = "TIMEOUT", default_value_t = 30)]
    timeout: u64,
}
```

Users can set the value via CLI flag or environment variable. `env` takes precedence if both set (default behavior; configurable).

### Help and Version

Clap generates `-h`/`--help` and `-V`/`--version` automatically. Customize:

```rust
#[derive(Parser, Debug)]
#[command(
    name = "mytool",
    about = "Description of my tool",
    version = "1.2.0",
    author = "Your Name",
    long_about = None,   // use `about` for short help
    after_help = "More info: https://example.com/docs"
)]
struct Cli { /* ... */ }
```

### Completions

```toml
[dependencies]
clap = { version = "4", features = ["derive", "env"] }
clap_complete = "4"
clap_complete_command = { version = "4", features = ["generate"] }

# Alternative: use clap_complete's generate function directly
```

```rust
use clap_complete::{CommandCompleter, Generator, shells};

fn main() {
    let cli = Cli::parse_from(["mytool", "--generate-completions", "bash"]);
    // ... or use clap_complete library
}
```

For a dedicated completions command:

```rust
use clap_complete::{generate, shells};

fn generate_completions(shell: &str) {
    let mut cmd = Cli::command();
    let shell = match shell {
        "bash" => shells::Bash,
        "zsh" => shells::Zsh,
        "fish" => shells::Fish,
        "powershell" => shells::PowerShell,
        " elvish" => shells::Elvish,
        _ => panic!("unknown shell"),
    };
    generate(shell, &mut cmd, "mytool", &mut std::io::stdout());
}
```

### Args from stdin

```rust
#[derive(Parser, Debug)]
struct Cli {
    /// Items (or read from stdin if not provided)
    #[arg(value_name = "ITEM", num_args = 1..)]
    items: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let items: Vec<String> = if cli.items.is_empty() {
        // read from stdin
        let mut stdin = String::new();
        std::io::stdin().read_to_string(&mut stdin).unwrap();
        stdin.lines().map(|l| l.to_string()).collect()
    } else {
        cli.items
    };
}
```

## Dialoguer: Interactive Prompts

```toml
[dependencies]
dialoguer = "0.10"
```

```rust
use dialoguer::{Confirm, Input, Password, Select};

// Confirm (y/n)
let confirm = Confirm::new("Are you sure you want to delete?")
    .default(false)
    .interact()
    .unwrap();

if confirm {
    println!("Deleted");
}

// Input
let name = Input::new()
    .with_prompt("Enter your name")
    .validate_on_insert(true)
    .interact()
    .unwrap();

// Password (hidden input)
let password = Password::new()
    .with_prompt("Enter password")
    .interact()
    .unwrap();

// Selection from list
let options = vec!["Option A", "Option B", "Option C"];
let selection = Select::new()
    .with_prompt("Choose an option")
    .items(&options)
    .default(0)
    .interact()
    .unwrap();
println!("Selected: {}", options[selection]);
```

For multi-select and other prompt types, see dialoguer docs.

## Console: Terminal Styling

```toml
[dependencies]
console = "0.15"
```

```rust
use console::{Style, color, measure};

// Styled output
println!("{}", Style::new().green().bold().apply_to("Success!"));
println!("{}", Style::new().red().apply_to("Error!"));
println!("{}", Style::new().dim().apply_to("Secondary info"));

// Colors
println!("{}", color::GREEN.fg()).apply_to("green text");

// Measure width (handles Unicode)
let width = measure("한글").sum();   // width in cells
println!("Width: {}", width);
```

For tables:

```rust
use console::Table;

let table = Table::new()
    .columns(&["Name", "Value"])
    .widths(&[20, 30])
    .add(&["Alice", "42"])
    .add(&["Bob", "99"]);

table.print(&mut std::io::stdout()).unwrap();
```

## Indicatif: Progress Bars and Spinners

```toml
[dependencies]
indicatif = "0.17"
```

```rust
use indicatif::{ProgressBar, ProgressStyle, HumanDuration, Spinner};

// Progress bar
let pb = ProgressBar::new(100);
pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
    .unwrap()
    .progress_chars("#>-"));

for i in 0..100 {
    do_work_item(i);
    pb.inc(1);
    // or pb.set_position(i);
}
pb.finish_with_message("Done!");

// Alternative: set_length and use set_position
let pb = ProgressBar::new(1000);
for i in 0..1000 {
    pb.set_position(i);
    std::thread::sleep(std::time::Duration::from_millis(10));
}

// Spinner (indeterminate)
let spinner = Spinner::new("Processing...");
std::thread::sleep(std::time::Duration::from_secs(2));
spinner.finish_with("Finished!");

// Multi-progress (multiple bars)
use indicatif::MultiProgress;
let multi = MultiProgress::new();
let pb1 = multi.add(ProgressBar::new(50));
let pb2 = multi.add(ProgressBar::new(50));
// ...
multi.clear().unwrap();   // clean up on exit
```

**Tips:**
- Avoid displaying progress bars when output is redirected to a file/pipe. Check `std::io::stdin().is_terminal()` or use `indicatif`'s `ProgressStyle::with_template` and the `progress_bar.set_style` based on terminal.
- For quiet mode, skip progress bars entirely when a `--quiet` flag is set.

## Terminal Detection and Colors

```rust
use std::io::{self, Write};

fn is_terminal() -> bool {
    io::stdout().is_terminal()
}

fn print_colored(msg: &str, color: bool) {
    if color && is_terminal() {
        println!("\x1b[32m{}\x1b[0m", msg);   // ANSI green
    } else {
        println!("{}", msg);
    }
}
```

Prefer `console` crate for cross-platform color handling. For environment-based color control:

```rust
use console::Term;
use std::env;

fn use_color() -> bool {
    env::var("NO_COLOR").is_err()
        && env::var("CLICOLOR").map(|v| v != "0").unwrap_or(true)
        && Term::stdout().is_rgb_terminal().unwrap_or(false)
}
```

**NO_COLOR:** presence of this env var means "don't use colors" (content is irrelevant, per no-color.org).
**CLICOLOR:** "0" means no colors, other value or unset means colors allowed.

## Configuration Files

Common patterns:

- **TOML** (Rust-native): `serde + toml` crate
- **JSON**: `serde_json`
- **YAML**: `serde_yaml`
- **INI**: `ini` crate

```rust
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct Config {
    api_key: String,
    timeout: u64,
    endpoints: Vec<String>,
}

fn load_config(path: &Path) -> Result<Config, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
```

For layered config (defaults → config file → env vars → CLI args):

```rust
// 1. Default config
let config = Config::default();

// 2. Override from file if present
if let Some(path) = cli.config.as_deref() {
    let file_config = load_config(Path::new(path))?;
    config = config.merge(file_config);   // or use a crate like config-rs
}

// 3. Override from env
if let Ok(val) = env::var("API_KEY") {
    config.api_key = val;
}

// 4. Override from CLI (already done during arg parsing)
```

Use the `config` crate for comprehensive layered config if needed.

## Common CLI Patterns

### Subcommand Dispatch

```rust
match cli.command {
    Commands::Add(args) => cmd_add(args),
    Commands::List(args) => cmd_list(args),
    Commands::Remove(args) => cmd_remove(args),
}
```

Each command function takes its args struct and returns `Result<(), Error>`.

### Error Handling in CLI

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Config error: {0}")]
    Config(String),
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    // ...
    Ok(())
}
```

Exit codes: 0 for success, 1 for general error, 2 for usage error (clap does this automatically for parse errors).

### Quiet/Verbose Flags

```rust
fn log_verbose(verbose: u8, msg: &str) {
    if verbose > 0 {
        eprintln!("[verbose] {}", msg);
    }
}

fn log_info(quiet: bool, msg: &str) {
    if !quiet {
        println!("{}", msg);
    }
}
```

### Output Formatting

For machine-readable output (JSON), add a `--json` flag:

```rust
#[derive(Parser, Debug)]
struct Cli {
    /// Output in JSON format
    #[arg(long)]
    json: bool,
}

fn output(result: &Item, json: bool) {
    if json {
        println!("{}", serde_json::to_string(result).unwrap());
    } else {
        println!("Name: {}, Value: {}", result.name, result.value);
    }
}
```

## Verification Checklist

- [ ] Can set up clap derive with `Parser`, `Subcommand`, `Args` derives
- [ ] Can define flags, options, positional args, subcommands with clap
- [ ] Can parse arguments and handle `Cli::parse()` / `parse_from()`
- [ ] Can use `value_parser` for custom validation
- [ ] Can set env var fallback with `#[arg(env = "...")]`
- [ ] Can generate shell completions with `clap_complete`
- [ ] Can use dialoguer for confirmation, input, password, and selection prompts
- [ ] Can style output with `console` (colors, bold, dim, tables)
- [ ] Can create a progress bar with `indicatif` and custom template
- [ ] Can use a spinner for indeterminate progress
- [ ] Can detect terminal vs piped output and adjust behavior (colors, progress bars)
- [ ] Can implement layered config (defaults → file → env → CLI)
- [ ] Can handle errors with a custom error enum + exit code

## Common Pitfalls

1. **Forgetting `#[command(subcommand)]` field.** Without it, subcommands don't parse. The field must be the enum of subcommands.

2. **Using `String` for optional args without `Option<String>`.** Required vs optional is determined by the type. `String` = required, `Option<String>` = optional.

3. **Not handling `clap::Error` in `try_parse()`.** `parse()` panics on error (prints help and exits). Use `try_parse()` if you want to handle errors yourself. Most CLIs want `parse()` behavior.

4. **Asking prompts when output is piped.** If stdout is not a terminal, prompts hang waiting for input. Check `is_terminal()` before prompting, or use a `--yes`/`--non-interactive` flag.

5. **Progress bars in CI.** CI systems often don't handle ANSI escape codes well, or capture them as output. Detect CI environment or `--quiet` and disable progress bars.

6. **Not validating early.** If a config file or argument is invalid, fail fast with a clear message. Don't proceed with partially initialized state.

7. **Missing `value_name` for clarity.** `clap` uses the field name as the value name in help text. Use `value_name` to give a clearer name (e.g., `value_name = "PATH"` for a path argument).

8. **Conflicting arg definitions.** If a flag is defined in multiple subcommands or globally, ensure the short/long names don't conflict. Use `global = true` for flags that apply to all subcommands.

9. **Not handling `--help` in custom argument parsing.** If you use `try_parse()`, handle the help case. If using `parse()`, clap handles it.

10. **Forgetting to set `default_value` for arguments that should have sensible defaults.** Users expect `--limit` to default to something reasonable, not fail with "missing argument."
