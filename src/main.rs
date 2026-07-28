use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use regex::Regex;
use serde::Deserialize;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct Config {
    pub naming: Option<NamingConfig>,
    pub defaults: Option<DefaultsConfig>,
    pub tree: Option<TreeConfig>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct NamingConfig {
    pub pattern: Option<String>,
    pub max_length: Option<usize>,
    pub prefix_separator: Option<String>,
    pub ticket_separator: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct DefaultsConfig {
    pub base: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct TreeConfig {
    pub no_legend: Option<bool>,
}

impl Config {
    pub fn load(explicit_path: Option<&std::path::Path>) -> Self {
        let mut config = Config::default();

        // 1. Global config (~/.config/branchbuddy/config.toml)
        if let Some(home) = dirs::home_dir() {
            let global_path = home.join(".config").join("branchbuddy").join("config.toml");
            if let Ok(content) = std::fs::read_to_string(global_path)
                && let Ok(parsed) = toml::from_str::<Config>(&content)
            {
                config.merge(parsed);
            }
        }

        // 2. Repo root config (.branchbuddy.toml)
        let repo_path = std::path::Path::new(".branchbuddy.toml");
        if let Ok(content) = std::fs::read_to_string(repo_path)
            && let Ok(parsed) = toml::from_str::<Config>(&content)
        {
            config.merge(parsed);
        }

        // 3. Explicit config path from CLI
        if let Some(path) = explicit_path
            && let Ok(content) = std::fs::read_to_string(path)
            && let Ok(parsed) = toml::from_str::<Config>(&content)
        {
            config.merge(parsed);
        }

        config
    }

    pub fn merge(&mut self, other: Config) {
        if let Some(other_naming) = other.naming {
            let n = self.naming.get_or_insert_with(Default::default);
            if other_naming.pattern.is_some() {
                n.pattern = other_naming.pattern;
            }
            if other_naming.max_length.is_some() {
                n.max_length = other_naming.max_length;
            }
            if other_naming.prefix_separator.is_some() {
                n.prefix_separator = other_naming.prefix_separator;
            }
            if other_naming.ticket_separator.is_some() {
                n.ticket_separator = other_naming.ticket_separator;
            }
        }
        if let Some(other_defaults) = other.defaults {
            let d = self.defaults.get_or_insert_with(Default::default);
            if other_defaults.base.is_some() {
                d.base = other_defaults.base;
            }
        }
        if let Some(other_tree) = other.tree {
            let t = self.tree.get_or_insert_with(Default::default);
            if other_tree.no_legend.is_some() {
                t.no_legend = other_tree.no_legend;
            }
        }
    }
}


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsMode {
    Git,
    Jj,
}

#[allow(dead_code)]
impl VcsMode {
    pub fn detect() -> Self {
        let current_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Self::detect_in_dir(&current_dir)
    }

    pub fn detect_in_dir(dir: &std::path::Path) -> Self {
        if dir.join(".jj").is_dir() {
            return VcsMode::Jj;
        }

        let is_jj = Command::new("jj")
            .arg("root")
            .current_dir(dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if is_jj {
            VcsMode::Jj
        } else {
            VcsMode::Git
        }
    }
}

#[derive(Parser)]
#[command(name = "branch-buddy")]
#[command(
    about = "Git companion CLI for persistent base-branch metadata and human-readable branch naming",
    long_about = None
)]
struct Cli {
    /// Path to custom configuration file
    #[arg(global = true, long)]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new branch with a slugified name and set its base
    New {
        /// Human-readable title for the branch
        title: String,

        /// Base branch (defaults to current branch)
        #[arg(long)]
        base: Option<String>,

        /// Optional prefix type (e.g., 'feature', 'bugfix')
        #[arg(long, id = "type")]
        r#type: Option<String>,

        /// Optional ticket ID (e.g., 'ABC-123')
        #[arg(long)]
        ticket: Option<String>,

        /// Perform a dry run without creating the branch
        #[arg(long)]
        dry_run: bool,

        /// Create the branch but do not check it out
        #[arg(long)]
        no_checkout: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Fail if branch already exists instead of appending a numeric suffix
        #[arg(long)]
        fail_if_exists: bool,
    },
    /// Get the base branch for the specified branch (or current branch)
    GetBase { branch: Option<String> },
    /// Set the base branch for a branch
    SetBase {
        /// The base branch to set
        base: String,
        /// The branch to update (defaults to current branch)
        branch: Option<String>,
        /// Skip validating that the base is a valid ref
        #[arg(long)]
        no_validate: bool,
    },
    /// Check if a branch has a base set (exits 0 if true, 1 otherwise)
    HasBase { branch: Option<String> },
    /// Guess the base branch for a branch
    GuessBase {
        branch: Option<String>,
        #[arg(long, default_value = "main,master,develop")]
        candidates: String,
        #[arg(long)]
        write: bool,
    },
    /// Show the branch ancestry tree
    Tree {
        branch: Option<String>,
        /// Hide the branch tree legend at the bottom
        #[arg(long)]
        no_legend: bool,
    },
    /// Install git hooks (post-checkout) to automatically track branches
    #[command(alias = "install")]
    InstallHooks,
    /// Check repository for broken base branch links and optionally fix them
    #[command(alias = "fsck")]
    Doctor {
        /// Automatically attempt to fix broken links using guess-base
        #[arg(long)]
        fix: bool,
        /// Install the post-checkout git hook for automatic health checks
        #[arg(long)]
        install_hook: bool,
    },
    /// Show focused commit log between branch and base (or stack)
    Log {
        /// Target branch (defaults to current branch)
        branch: Option<String>,

        /// Include commits across all parent base branches up to trunk()
        #[arg(long, alias = "all-ancestors")]
        stack: bool,

        /// Show file diff statistics for each commit
        #[arg(long)]
        stat: bool,

        /// Limit number of commits displayed
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },
}

fn current_branch() -> Result<String> {
    let check_repo = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to run git command")?;

    if !check_repo.status.success() {
        return Err(anyhow!("Not in a git repository."));
    }

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unknown revision or path") {
            return Err(anyhow!(
                "Repository has no commits yet. Please make an initial commit before creating branches."
            ));
        }
        return Err(anyhow!("Failed to get current branch."));
    }

    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    if branch == "HEAD" {
        return Err(anyhow!("Currently in detached HEAD state."));
    }

    Ok(branch)
}

fn slugify(input: &str, max_length: Option<usize>) -> String {
    let mut s = input.to_lowercase();
    let re = Regex::new(r"[^a-z0-9]").unwrap();
    s = re.replace_all(&s, "-").to_string();

    let re_multi_dash = Regex::new(r"-+").unwrap();
    s = re_multi_dash.replace_all(&s, "-").to_string();

    s = s.trim_matches('-').to_string();

    if s.is_empty() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        s = format!("branch-{}", now);
    }

    let limit = max_length.unwrap_or(63);
    s.chars().take(limit).collect()
}

fn ref_exists(git_ref: &str) -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", git_ref])
        .output()
        .unwrap();
    output.status.success()
}

fn branch_exists(branch: &str) -> bool {
    let output = Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", branch),
        ])
        .output()
        .unwrap();
    output.status.success()
}

// Write Base Metadata
fn set_base(base: &str, branch: Option<&str>, validate: bool) -> Result<()> {
    let target_branch = match branch {
        Some(b) => b.to_string(),
        None => current_branch()?,
    };

    if validate && !ref_exists(base) {
        return Err(anyhow!(
            "Base branch '{}' does not appear to be a valid ref",
            base
        ));
    }

    let config_key = format!("branch.{}.base", target_branch);
    let output = Command::new("git")
        .args(["config", "--local", &config_key, base])
        .output()
        .context("Failed to set git config")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to set base branch in git config"));
    }

    Ok(())
}

fn get_base(branch: Option<&str>) -> Result<String> {
    let target_branch = match branch {
        Some(b) => b.to_string(),
        None => current_branch()?,
    };

    let config_key = format!("branch.{}.base", target_branch);
    let output = Command::new("git")
        .args(["config", "--get", &config_key])
        .output()
        .context("Failed to get git config")?;

    if !output.status.success() {
        return Err(anyhow!("No base branch set for '{}'", target_branch));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn has_base(branch: Option<&str>) -> Result<bool> {
    let target_branch = match branch {
        Some(b) => b.to_string(),
        None => current_branch()?,
    };

    let config_key = format!("branch.{}.base", target_branch);
    let output = Command::new("git")
        .args(["config", "--get", &config_key])
        .output()?;

    Ok(output.status.success())
}

fn build_tree_lines<F>(start: &str, get_base_fn: F) -> Vec<String>
where
    F: FnMut(&str) -> Option<String>,
{
    build_tree_lines_with_formatter(start, get_base_fn, |s| s.to_string())
}

fn is_jj_trunk(ref_name: &str) -> bool {
    if ref_name == "root()" || ref_name == "zzzzzzzzzzzz" || ref_name.is_empty() {
        return false;
    }
    if let Ok(output) = Command::new("jj")
        .args([
            "log",
            "-r",
            &format!("({}) & trunk()", ref_name),
            "--no-graph",
            "-T",
            "commit_id",
        ])
        .output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return !stdout.trim().is_empty();
    }
    false
}

fn build_tree_lines_with_formatter<F, L>(
    start: &str,
    mut get_base_fn: F,
    mut format_label_fn: L,
) -> Vec<String>
where
    F: FnMut(&str) -> Option<String>,
    L: FnMut(&str) -> String,
{
    let mut lines = Vec::new();
    let mut current = start.to_string();

    let start_icon = if is_jj_trunk(&current) { "🪵" } else { "🌿" };
    let start_label = format_label_fn(&current);
    lines.push(format!("{} {}", start_icon, start_label.green()));

    let mut depth = 1;
    let mut seen = vec![current.clone()];

    while let Some(base) = get_base_fn(&current) {
        let base_icon = if is_jj_trunk(&base) { "🪵" } else { "🌿" };
        let base_label = format_label_fn(&base);
        if seen.contains(&base) {
            let prefix = "    ".repeat(depth - 1);
            lines.push(format!(
                "{}└── {} {} {}",
                prefix.dimmed(),
                base_icon,
                base_label.blue(),
                "(cycle detected)".red()
            ));
            break;
        }

        let prefix = "    ".repeat(depth - 1);
        lines.push(format!("{}└── {} {}", prefix.dimmed(), base_icon, base_label.blue()));
        seen.push(base.clone());
        current = base;
        depth += 1;
    }

    lines
}

fn format_jj_node_label(ref_name: &str) -> String {
    let output = Command::new("jj")
        .args([
            "log",
            "-r",
            ref_name,
            "--no-graph",
            "-T",
            r#"change_id.short() ++ "\t" ++ local_bookmarks"#,
        ])
        .output();

    if let Ok(o) = output
        && o.status.success()
    {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let line = stdout.lines().next().unwrap_or("").trim_end_matches(['\r', '\n']);
        if !line.is_empty() {
            let mut parts = line.split('\t');
            let change_id = parts.next().map(|s| s.trim()).filter(|s| !s.is_empty());
            let bookmarks_str = parts.next().unwrap_or("").trim();
            let bookmarks: Vec<&str> = bookmarks_str.split_whitespace().collect();

            if let Some(cid) = change_id {
                let name = if bookmarks.contains(&ref_name) {
                    ref_name
                } else if let Some(&first_bm) = bookmarks.first() {
                    first_bm
                } else if ref_name != "@" && !ref_name.is_empty() {
                    ref_name
                } else {
                    "(un-bookmarked)"
                };
                return format!("{} [change: {}]", name, cid);
            }
        }
    }

    ref_name.to_string()
}

fn print_tree(branch: Option<&str>, no_legend: bool, config: &Config) -> Result<()> {
    let mode = VcsMode::detect();
    print_tree_with_mode(branch, no_legend, mode, config)
}

fn get_jj_parent_ref(ref_name: &str) -> Option<String> {
    // Stop parent walk if we hit root or empty ref or trunk
    if ref_name == "root()" || ref_name == "zzzzzzzzzzzz" || ref_name.is_empty() || is_jj_trunk(ref_name) {
        return None;
    }

    let rev_arg = format!("parents({})", ref_name);
    let output = Command::new("jj")
        .args([
            "log",
            "-r",
            &rev_arg,
            "--no-graph",
            "-T",
            r#"local_bookmarks ++ "\t" ++ change_id.short() ++ "\t" ++ root"#,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next().unwrap_or("").trim();
        if !line.is_empty() {
            let mut parts = line.split('\t');
            let bookmarks_str = parts.next().unwrap_or("").trim();
            if !bookmarks_str.is_empty()
                && let Some(bm) = bookmarks_str.split_whitespace().next()
            {
                return Some(bm.to_string());
            }
            let cid = parts.next().unwrap_or("").trim();
            let is_root = parts.next().unwrap_or("").trim() == "true";
            if !is_root && !cid.is_empty() && cid != "zzzzzzzzzzzz" {
                return Some(cid.to_string());
            }
        }
    }
    None
}



fn print_tree_with_mode(
    branch: Option<&str>,
    no_legend: bool,
    mode: VcsMode,
    config: &Config,
) -> Result<()> {
    let start_branch = match branch {
        Some(b) => b.to_string(),
        None => match mode {
            VcsMode::Git => current_branch()?,
            VcsMode::Jj => get_current_jj_bookmark_or_at(),
        },
    };

    let lines = match mode {
        VcsMode::Git => build_tree_lines(&start_branch, |b| get_base(Some(b)).ok()),
        VcsMode::Jj => build_tree_lines_with_formatter(
            &start_branch,
            |b| get_base(Some(b)).ok().or_else(|| get_jj_parent_ref(b)),
            format_jj_node_label,
        ),
    };

    println!("{}", "🌳 Branch Ancestry (child → parent base):".bold());
    for line in lines {
        println!("{}", line);
    }

    let hide_legend =
        no_legend || config.tree.as_ref().and_then(|t| t.no_legend).unwrap_or(false);

    if !hide_legend {
        println!(
            "\n{}",
            "Legend: 🌿 Branch/Feature | 🪵 Trunk/Main Target".dimmed()
        );
    }

    Ok(())
}

fn get_git_default_branch() -> String {
    // 1. Try remote HEAD (origin/HEAD)
    if let Ok(output) = Command::new("git")
        .args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let name = stdout.trim();
        if !name.is_empty() {
            return name.strip_prefix("origin/").unwrap_or(name).to_string();
        }
    }

    // 2. Fall back to checking common default local branch refs
    for candidate in ["main", "master", "development", "dev", "trunk"] {
        if ref_exists(candidate) {
            return candidate.to_string();
        }
    }

    "main".to_string()
}

fn get_jj_trunk_name() -> String {
    if let Ok(output) = Command::new("jj")
        .args(["log", "-r", "trunk()", "--no-graph", "-T", "local_bookmarks"])
        .output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(first_bm) = stdout.split_whitespace().next()
            && !first_bm.trim().is_empty()
        {
            return first_bm.trim().to_string();
        }
    }
    "trunk()".to_string()
}

fn handle_log(
    branch: Option<&str>,
    stack: bool,
    stat: bool,
    limit: Option<usize>,
) -> Result<()> {
    let mode = VcsMode::detect();
    let target = match branch {
        Some(b) => b.to_string(),
        None => match mode {
            VcsMode::Git => current_branch()?,
            VcsMode::Jj => get_current_jj_bookmark_or_at(),
        },
    };

    let base = get_base(Some(&target)).unwrap_or_else(|_| match mode {
        VcsMode::Git => get_git_default_branch(),
        VcsMode::Jj => get_jj_trunk_name(),
    });

    println!(
        "{}",
        format!("🪵 Log for {} (base: {}):", target.green(), base.blue()).bold()
    );

    match mode {
        VcsMode::Git => {
            let root_base = if stack { get_git_default_branch() } else { base.clone() };
            let range = format!("{}..{}", root_base, target);
            let mut args = vec!["log", &range, "--oneline", "--color=always"];
            let limit_str = limit.map(|n| n.to_string());
            if let Some(ref l) = limit_str {
                args.push("-n");
                args.push(l);
            }
            if stat {
                args.push("--stat");
            }
            let output = Command::new("git").args(args).output()?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        VcsMode::Jj => {
            let revset = if stack {
                format!("trunk()::({}) & ~trunk()", target)
            } else {
                format!("({})::({}) & ~({})", base, target, base)
            };
            let mut args = vec![
                "log",
                "-r",
                &revset,
                "--no-graph",
                "-T",
                r#"commit_id.short() ++ " [" ++ change_id.short() ++ "] " ++ description.first_line() ++ " (" ++ author.name() ++ ")\n""#,
            ];
            let limit_str = limit.map(|n| n.to_string());
            if let Some(ref l) = limit_str {
                args.push("-n");
                args.push(l);
            }
            if stat {
                args.push("--stat");
            }
            let output = Command::new("jj").args(args).output()?;
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}

fn get_current_jj_bookmark_or_at() -> String {
    if let Ok(output) = Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", "local_bookmarks"])
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty()
            && let Some(b) = text.split_whitespace().next()
        {
            return b.to_string();
        }
    }

    if let Ok(output) = Command::new("jj")
        .args(["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"])
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty()
            && let Some(b) = text.split_whitespace().next()
        {
            return b.to_string();
        }
    }

    "@".to_string()
}

fn jj_ref_exists(git_ref: &str) -> bool {
    let output = Command::new("jj")
        .args(["log", "-r", git_ref, "--no-graph"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    output.map(|o| o.status.success()).unwrap_or(false)
}

fn jj_bookmark_exists(name: &str) -> bool {
    let output = Command::new("jj")
        .args(["bookmark", "list", name])
        .output();
    if let Ok(o) = output
        && o.status.success()
    {
        let stdout = String::from_utf8_lossy(&o.stdout);
        return stdout.contains(&format!("{}:", name));
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn new_branch(
    title: &str,
    base: Option<&str>,
    r#type: Option<&str>,
    ticket: Option<&str>,
    dry_run: bool,
    no_checkout: bool,
    json: bool,
    fail_if_exists: bool,
    config: &Config,
) -> Result<()> {
    let mode = VcsMode::detect();
    new_branch_with_mode(
        title,
        base,
        r#type,
        ticket,
        dry_run,
        no_checkout,
        json,
        fail_if_exists,
        mode,
        config,
    )
}

#[allow(clippy::too_many_arguments)]
fn new_branch_with_mode(
    title: &str,
    base: Option<&str>,
    r#type: Option<&str>,
    ticket: Option<&str>,
    dry_run: bool,
    no_checkout: bool,
    json: bool,
    fail_if_exists: bool,
    mode: VcsMode,
    config: &Config,
) -> Result<()> {
    let max_len = config.naming.as_ref().and_then(|n| n.max_length);
    let prefix_sep = config
        .naming
        .as_ref()
        .and_then(|n| n.prefix_separator.as_deref())
        .unwrap_or("/");
    let ticket_sep = config
        .naming
        .as_ref()
        .and_then(|n| n.ticket_separator.as_deref())
        .unwrap_or("-");

    match mode {
        VcsMode::Git => {
            let mut create_at = "HEAD".to_string();

            let base_branch = match base {
                Some(b) => {
                    create_at = b.to_string();
                    b.to_string()
                }
                None => {
                    if let Some(default_base) =
                        config.defaults.as_ref().and_then(|d| d.base.as_deref())
                    {
                        create_at = default_base.to_string();
                        default_base.to_string()
                    } else {
                        match current_branch() {
                            Ok(b) => {
                                create_at = b.clone();
                                b
                            }
                            Err(e) => {
                                if e.to_string().contains("detached HEAD") {
                                    println!(
                                        "{} {}",
                                        "⚠️".yellow(),
                                        "Currently in detached HEAD. Resolving base branch...".yellow()
                                    );

                                    let all_branches = get_all_local_branches();
                                    let candidates: Vec<&str> =
                                        all_branches.iter().map(|s| s.as_str()).collect();

                                    let ranked = rank_closest_bases("HEAD", &candidates);

                                    if ranked.is_empty() {
                                        return Err(anyhow!(
                                            "Failed to find any local branches. Please specify one explicitly: `branch-buddy new <name> <base>`"
                                        ));
                                    }

                                    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                                        .with_prompt(
                                            "Select base branch for metadata (closest match pre-selected)",
                                        )
                                        .default(0)
                                        .items(&ranked)
                                        .interact()?;

                                    let base = ranked[selection].clone();
                                    println!("💾 Selected base: {}", base.blue());
                                    base
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
            };

            // verify base
            if !ref_exists(&base_branch) {
                return Err(anyhow!(
                    "Base branch '{}' does not appear to be a valid ref",
                    base_branch
                ));
            }

            let slug = slugify(title, max_len);

            let mut branch_name = match (r#type, ticket) {
                (Some(t), Some(id)) => format!("{}{}{}{}{}", t, prefix_sep, id, ticket_sep, slug),
                (Some(t), None) => format!("{}{}{}", t, prefix_sep, slug),
                (None, Some(id)) => format!("{}{}{}", id, ticket_sep, slug),
                (None, None) => slug,
            };

            if branch_exists(&branch_name) {
                if fail_if_exists {
                    return Err(anyhow!("Branch '{}' already exists.", branch_name));
                }
                let mut i = 2;
                loop {
                    let alt_name = format!("{}-{}", branch_name, i);
                    if !branch_exists(&alt_name) {
                        branch_name = alt_name;
                        break;
                    }
                    i += 1;
                }
            }

            if !dry_run {
                let mut branch_cmd = Command::new("git");
                branch_cmd.args(["branch", &branch_name, &create_at]);
                let output = branch_cmd.output()?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow!("Failed to create branch: {}", stderr));
                }

                if !no_checkout {
                    let mut co_cmd = Command::new("git");
                    co_cmd.args(["checkout", &branch_name]);
                    let co_output = co_cmd.output()?;
                    if !co_output.status.success() {
                        let stderr = String::from_utf8_lossy(&co_output.stderr);
                        return Err(anyhow!("Failed to checkout branch: {}", stderr));
                    }
                }

                set_base(&base_branch, Some(&branch_name), false)?;
            }

            if json {
                println!(
                    r#"{{"branch": "{}", "base": "{}"}}"#,
                    branch_name, base_branch
                );
            } else {
                println!("✨ Created branch: {}", branch_name.green());
                println!("🌱 Base: {}", base_branch.blue());
            }

            Ok(())
        }
        VcsMode::Jj => {
            let base_branch = match base {
                Some(b) => b.to_string(),
                None => {
                    if let Some(default_base) =
                        config.defaults.as_ref().and_then(|d| d.base.as_deref())
                    {
                        default_base.to_string()
                    } else {
                        get_current_jj_bookmark_or_at()
                    }
                }
            };

            if !jj_ref_exists(&base_branch) && !ref_exists(&base_branch) {
                return Err(anyhow!(
                    "Base branch '{}' does not appear to be a valid ref",
                    base_branch
                ));
            }

            let slug = slugify(title, max_len);

            let mut branch_name = match (r#type, ticket) {
                (Some(t), Some(id)) => format!("{}{}{}{}{}", t, prefix_sep, id, ticket_sep, slug),
                (Some(t), None) => format!("{}{}{}", t, prefix_sep, slug),
                (None, Some(id)) => format!("{}{}{}", id, ticket_sep, slug),
                (None, None) => slug,
            };

            if branch_exists(&branch_name) || jj_bookmark_exists(&branch_name) {
                if fail_if_exists {
                    return Err(anyhow!("Branch/bookmark '{}' already exists.", branch_name));
                }
                let mut i = 2;
                loop {
                    let alt_name = format!("{}-{}", branch_name, i);
                    if !branch_exists(&alt_name) && !jj_bookmark_exists(&alt_name) {
                        branch_name = alt_name;
                        break;
                    }
                    i += 1;
                }
            }

            if !dry_run {
                let prev_at = if no_checkout {
                    Command::new("jj")
                        .args(["log", "-r", "@", "--no-graph", "-T", "change_id"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                } else {
                    None
                };

                let new_cmd = Command::new("jj")
                    .args(["new", &base_branch, "-m", title])
                    .output()?;
                if !new_cmd.status.success() {
                    let stderr = String::from_utf8_lossy(&new_cmd.stderr);
                    return Err(anyhow!("Failed to create jj revision: {}", stderr));
                }

                let bm_cmd = Command::new("jj")
                    .args(["bookmark", "create", &branch_name, "-r", "@"])
                    .output()?;
                if !bm_cmd.status.success() {
                    let stderr = String::from_utf8_lossy(&bm_cmd.stderr);
                    return Err(anyhow!("Failed to create jj bookmark: {}", stderr));
                }

                if let Some(prev) = prev_at
                    && !prev.is_empty()
                {
                    let _ = Command::new("jj").args(["edit", &prev]).output();
                }

                set_base(&base_branch, Some(&branch_name), false)?;
            }

            if json {
                println!(
                    r#"{{"branch": "{}", "base": "{}"}}"#,
                    branch_name, base_branch
                );
            } else {
                println!("✨ Created branch: {}", branch_name.green());
                println!("🌱 Base: {}", base_branch.blue());
            }

            Ok(())
        }
    }
}

fn get_all_local_branches() -> Vec<String> {
    let output = Command::new("git")
        .args(["for-each-ref", "--format=%(refname:short)", "refs/heads/"])
        .output()
        .unwrap_or_else(|_| std::process::Command::new("true").output().unwrap());

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![]
    }
}

fn rank_closest_bases(target: &str, candidates: &[&str]) -> Vec<String> {
    let mut scored: Vec<(String, usize)> = candidates
        .iter()
        .filter_map(|&cand| {
            if cand == target {
                return None;
            }
            if !ref_exists(cand) {
                return None;
            }

            let output = Command::new("git")
                .args(["merge-base", target, cand])
                .output()
                .unwrap_or_else(|_| std::process::Command::new("true").output().unwrap());

            if output.status.success() {
                let mb_sha = String::from_utf8(output.stdout)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if mb_sha.is_empty() {
                    return None;
                }

                let d_out = Command::new("git")
                    .args(["rev-list", "--count", &format!("{}..{}", mb_sha, target)])
                    .output()
                    .unwrap_or_else(|_| std::process::Command::new("true").output().unwrap());

                if d_out.status.success() {
                    let dist: usize = String::from_utf8(d_out.stdout)
                        .unwrap_or_default()
                        .trim()
                        .parse()
                        .unwrap_or(usize::MAX);
                    Some((cand.to_string(), dist))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    scored.sort_by_key(|&(_, dist)| dist);
    scored.into_iter().map(|(cand, _)| cand).collect()
}

fn guess_base(branch: Option<&str>, candidates: &str, write: bool) -> Result<()> {
    let target_branch = match branch {
        Some(b) => b.to_string(),
        None => current_branch()?,
    };

    let cand_list: Vec<&str> = candidates.split(',').map(|s| s.trim()).collect();
    let ranked = rank_closest_bases(&target_branch, &cand_list);
    let best_base = ranked.into_iter().next();

    if let Some(base) = best_base {
        println!("🔮 Guessed base: {}", base.blue());
        if write {
            set_base(&base, Some(&target_branch), false)?;
            println!(
                "💾 Saved base {} for branch {}",
                base.blue(),
                target_branch.green()
            );
        }
        Ok(())
    } else {
        Err(anyhow!("Could not guess a base branch from candidates"))
    }
}

fn do_install_hook(enable_health_check: bool) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Failed to get git directory")?;

    if !output.status.success() {
        return Err(anyhow!("Not in a git repository."));
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let hooks_dir = std::path::Path::new(&git_dir).join("hooks");

    if !hooks_dir.exists() {
        std::fs::create_dir_all(&hooks_dir)?;
    }

    let hook_path = hooks_dir.join("post-checkout");

    let health_check_snippet = if enable_health_check {
        r#"# If the current branch has a base, but that base branch was deleted, warn the user
base=$(branch-buddy get-base "$curr" 2>/dev/null)
if [ -n "$base" ] && ! git show-ref --verify --quiet "refs/heads/$base"; then
    echo "⚠️  Base branch '$base' is missing! Run 'branch-buddy doctor --fix' to heal it."
fi"#
    } else {
        r#"# OPTIONAL: Uncomment the lines below to enable automatic health checks
# If the current branch has a base, but that base branch was deleted, warn the user
# base=$(branch-buddy get-base "$curr" 2>/dev/null)
# if [ -n "$base" ] && ! git show-ref --verify --quiet "refs/heads/$base"; then
#     echo "⚠️  Base branch '$base' is missing! Run 'branch-buddy doctor --fix' to heal it."
# fi"#
    };

    let hook_content = format!(
        r#"#!/bin/bash
# post-checkout

# Flag 1 means a branch checkout (not a file checkout)
if [ "$3" != "1" ]; then exit 0; fi

# Get current branch
curr=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
if [ "$curr" == "HEAD" ]; then exit 0; fi

{}

# If base is already set, do nothing
if branch-buddy has-base "$curr" >/dev/null 2>&1; then
    exit 0
fi

# Try to determine the previous branch name
prev_branch=$(git rev-parse --abbrev-ref @{{-1}} 2>/dev/null)

if [ -n "$prev_branch" ] && [ "$prev_branch" != "HEAD" ]; then
    branch-buddy set-base "$prev_branch" "$curr" >/dev/null 2>&1
fi
"#,
        health_check_snippet
    );

    if hook_path.exists() {
        let content = std::fs::read_to_string(&hook_path)?;
        if !content.contains("branch-buddy") {
            return Err(anyhow!(
                "A post-checkout hook already exists at {}. Please merge the branch-buddy hook manually.",
                hook_path.display()
            ));
        } else {
            println!(
                "✅ branch-buddy post-checkout hook is {} installed.",
                "already".yellow()
            );
            return Ok(());
        }
    }

    std::fs::write(&hook_path, hook_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }

    println!(
        "🎉 {} post-checkout hook at {}",
        "Successfully installed".green(),
        hook_path.display()
    );

    if !enable_health_check {
        println!(
            "\n💡 {}: You can also enable automatic health checks that warn you about broken base branch links.",
            "Tip".yellow().bold()
        );
        println!(
            "Run `{}` to enable them!",
            "branch-buddy doctor --install-hook".cyan()
        );
    }

    Ok(())
}

fn doctor(fix: bool, install_hook: bool) -> Result<()> {
    if install_hook {
        do_install_hook(true)?;
        return Ok(());
    }
    let output = Command::new("git")
        .args(["for-each-ref", "--format=%(refname:short)", "refs/heads/"])
        .output()
        .context("Failed to list branches")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to list branches."));
    }

    let branches_str = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = branches_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut broken_count = 0;

    for branch in branches {
        if let Ok(base) = get_base(Some(branch))
            && !ref_exists(&base)
        {
            println!(
                "⚠️  Branch '{}' points to missing base '{}'",
                branch.yellow(),
                base.red()
            );
            broken_count += 1;

            if fix {
                println!("   Attempting to auto-fix '{}'...", branch.yellow());
                match guess_base(Some(branch), "main,master,develop", true) {
                    Ok(_) => println!("   ✅ Fixed '{}'", branch.green()),
                    Err(e) => println!("   ❌ Failed to auto-fix: {}", e.to_string().red()),
                }
            }
        }
    }

    if broken_count == 0 {
        println!(
            "🩺 Repository is perfectly healthy! All base branch links are {}.",
            "intact".green()
        );
    } else if !fix {
        println!(
            "\nFound {} broken link(s). Run `{}` to auto-heal them.",
            broken_count.to_string().red().bold(),
            "branch-buddy doctor --fix".cyan()
        );
    } else {
        println!(
            "\nDoctor finished repairing {} broken link(s).",
            broken_count.to_string().green().bold()
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref());

    match &cli.command {
        Commands::New {
            title,
            base,
            r#type,
            ticket,
            dry_run,
            no_checkout,
            json,
            fail_if_exists,
        } => {
            new_branch(
                title,
                base.as_deref(),
                r#type.as_deref(),
                ticket.as_deref(),
                *dry_run,
                *no_checkout,
                *json,
                *fail_if_exists,
                &config,
            )?;
        }
        Commands::GetBase { branch } => {
            let b = get_base(branch.as_deref())?;
            println!("{}", b);
        }
        Commands::SetBase {
            base,
            branch,
            no_validate,
        } => {
            set_base(base, branch.as_deref(), !*no_validate)?;
            let b = branch.as_deref().unwrap_or("current branch");
            println!("🔗 Set base of {} to {}", b.green(), base.blue());
        }
        Commands::HasBase { branch } => {
            let has = has_base(branch.as_deref()).unwrap_or(false);
            if !has {
                std::process::exit(1);
            }
        }
        Commands::GuessBase {
            branch,
            candidates,
            write,
        } => {
            guess_base(branch.as_deref(), candidates, *write)?;
        }
        Commands::Tree { branch, no_legend } => {
            print_tree(branch.as_deref(), *no_legend, &config)?;
        }
        Commands::Doctor { fix, install_hook } => {
            doctor(*fix, *install_hook)?;
        }
        Commands::InstallHooks => {
            do_install_hook(false)?;
        }
        Commands::Log {
            branch,
            stack,
            stat,
            limit,
        } => {
            handle_log(branch.as_deref(), *stack, *stat, *limit)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vcs_mode_detection() {
        let _detected = VcsMode::detect();

        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bb_test_vcs_{}", unique_id));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mode_clean = VcsMode::detect_in_dir(&temp_dir);
        assert_eq!(mode_clean, VcsMode::Git);

        let jj_dir = temp_dir.join(".jj");
        let _ = std::fs::create_dir(&jj_dir);
        let mode_jj = VcsMode::detect_in_dir(&temp_dir);
        assert_eq!(mode_jj, VcsMode::Jj);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Fix login bug", None), "fix-login-bug");
        assert_eq!(slugify("User Signup (New Flow)!", Some(15)), "user-signup-new");
        assert_eq!(slugify("User Signup (New Flow)!", None), "user-signup-new-flow");
        let empty = slugify("   ", None);
        assert!(empty.starts_with("branch-"));
    }

    #[test]
    fn test_build_tree_lines() {
        let mock_bases = |branch: &str| -> Option<String> {
            match branch {
                "feature/my-branch" => Some("dev".to_string()),
                "dev" => Some("main".to_string()),
                "main" => None,
                _ => None,
            }
        };

        let lines = build_tree_lines("feature/my-branch", mock_bases);
        assert_eq!(lines.len(), 3);
        // Strip ANSI codes for comparison
        let strip = |s: &str| -> String {
            let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
            re.replace_all(s, "").to_string()
        };
        assert_eq!(strip(&lines[0]), "🌿 feature/my-branch");
        assert_eq!(strip(&lines[1]), "└── 🌿 dev");
        assert_eq!(strip(&lines[2]), "    └── 🪵 main");
    }

    #[test]
    fn test_build_tree_lines_cycle() {
        let mock_bases = |branch: &str| -> Option<String> {
            match branch {
                "A" => Some("B".to_string()),
                "B" => Some("C".to_string()),
                "C" => Some("A".to_string()),
                _ => None,
            }
        };

        let lines = build_tree_lines("A", mock_bases);
        assert_eq!(lines.len(), 4);
        let strip = |s: &str| -> String {
            let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
            re.replace_all(s, "").to_string()
        };
        assert_eq!(strip(&lines[0]), "🌿 A");
        assert_eq!(strip(&lines[1]), "└── 🌿 B");
        assert_eq!(strip(&lines[2]), "    └── 🌿 C");
        assert_eq!(strip(&lines[3]), "        └── 🌿 A (cycle detected)");
    }

    #[test]
    fn test_jj_new_branch_slugification() {
        let title = "Fix user signup flow";
        let slug = slugify(title, None);
        assert_eq!(slug, "fix-user-signup-flow");
    }

    #[test]
    fn test_jj_new_branch_dry_run() {
        let res = new_branch_with_mode(
            "Fix user signup flow",
            Some("main"),
            Some("feature"),
            Some("ABC-123"),
            true,
            false,
            true,
            false,
            VcsMode::Jj,
            &Config::default(),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_build_tree_lines_with_formatter() {
        let mock_bases = |branch: &str| -> Option<String> {
            match branch {
                "feature/my-branch" => Some("dev".to_string()),
                "dev" => Some("main".to_string()),
                "main" => None,
                _ => None,
            }
        };

        let mock_formatter = |branch: &str| -> String {
            match branch {
                "feature/my-branch" => "feature/my-branch [change: abc12345]".to_string(),
                "dev" => "dev [change: def67890]".to_string(),
                "main" => "main [change: main1234]".to_string(),
                _ => branch.to_string(),
            }
        };

        let lines = build_tree_lines_with_formatter("feature/my-branch", mock_bases, mock_formatter);
        assert_eq!(lines.len(), 3);
        let strip = |s: &str| -> String {
            let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
            re.replace_all(s, "").to_string()
        };
        assert_eq!(strip(&lines[0]), "🌿 feature/my-branch [change: abc12345]");
        assert_eq!(strip(&lines[1]), "└── 🌿 dev [change: def67890]");
        assert_eq!(strip(&lines[2]), "    └── 🪵 main [change: main1234]");
    }

    #[test]
    fn test_build_tree_lines_unbookmarked() {
        let mock_bases = |branch: &str| -> Option<String> {
            match branch {
                "wip" => Some("@".to_string()),
                "@" => Some("main".to_string()),
                "main" => None,
                _ => None,
            }
        };

        let mock_formatter = |branch: &str| -> String {
            match branch {
                "wip" => "wip [change: wip12345]".to_string(),
                "@" => "(un-bookmarked) [change: at123456]".to_string(),
                "main" => "main [change: main1234]".to_string(),
                _ => branch.to_string(),
            }
        };

        let lines = build_tree_lines_with_formatter("wip", mock_bases, mock_formatter);
        assert_eq!(lines.len(), 3);
        let strip = |s: &str| -> String {
            let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
            re.replace_all(s, "").to_string()
        };
        assert_eq!(strip(&lines[0]), "🌿 wip [change: wip12345]");
        assert_eq!(strip(&lines[1]), "└── 🌿 (un-bookmarked) [change: at123456]");
        assert_eq!(strip(&lines[2]), "    └── 🪵 main [change: main1234]");
    }

    #[test]
    fn test_log_revset_formatting() {
        let base = "development";
        let branch = "feature-x";
        let git_range = format!("{}..{}", base, branch);
        assert_eq!(git_range, "development..feature-x");

        let jj_revset = format!("({})::({}) & ~({})", base, branch, base);
        assert_eq!(jj_revset, "(development)::(feature-x) & ~(development)");
    }

    #[test]
    fn test_log_cli_parsing() {
        let cli = Cli::try_parse_from([
            "branch-buddy",
            "log",
            "feature-x",
            "--stack",
            "--stat",
            "-n",
            "10",
        ])
        .unwrap();
        if let Commands::Log {
            branch,
            stack,
            stat,
            limit,
        } = cli.command
        {
            assert_eq!(branch, Some("feature-x".to_string()));
            assert!(stack);
            assert!(stat);
            assert_eq!(limit, Some(10));
        } else {
            panic!("Expected Commands::Log");
        }

        let cli_alias = Cli::try_parse_from(["branch-buddy", "log", "--all-ancestors"]).unwrap();
        if let Commands::Log { stack, .. } = cli_alias.command {
            assert!(stack);
        } else {
            panic!("Expected Commands::Log");
        }
    }

    #[test]
    fn test_config_partial_deserialization() {
        let toml_str = r#"
        [naming]
        max_length = 50

        [tree]
        no_legend = true
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.naming.as_ref().unwrap().max_length, Some(50));
        assert_eq!(config.naming.as_ref().unwrap().pattern, None);
        assert_eq!(config.tree.as_ref().unwrap().no_legend, Some(true));
        assert!(config.defaults.is_none());
    }

    #[test]
    fn test_config_cascade_merging() {
        let mut base_config = Config {
            naming: Some(NamingConfig {
                max_length: Some(63),
                ..Default::default()
            }),
            tree: Some(TreeConfig {
                no_legend: Some(false),
            }),
            ..Default::default()
        };

        let repo_config = Config {
            naming: Some(NamingConfig {
                max_length: Some(50),
                ..Default::default()
            }),
            tree: Some(TreeConfig {
                no_legend: Some(true),
            }),
            ..Default::default()
        };

        base_config.merge(repo_config);
        assert_eq!(base_config.naming.as_ref().unwrap().max_length, Some(50));
        assert_eq!(base_config.tree.as_ref().unwrap().no_legend, Some(true));
    }
}

