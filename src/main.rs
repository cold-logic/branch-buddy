use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use regex::Regex;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "branch-buddy")]
#[command(
    about = "Git companion CLI for persistent base-branch metadata and human-readable branch naming",
    long_about = None
)]
struct Cli {
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
    GetBase {
        branch: Option<String>,
    },
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
    HasBase {
        branch: Option<String>,
    },
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

fn slugify(input: &str) -> String {
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

    s.chars().take(63).collect()
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

    if validate {
        if !ref_exists(base) {
            return Err(anyhow!(
                "Base branch '{}' does not appear to be a valid ref",
                base
            ));
        }
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

fn build_tree_lines<F>(start: &str, mut get_base_fn: F) -> Vec<String>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut lines = Vec::new();
    let mut current = start.to_string();

    lines.push(current.clone().green().to_string());

    let mut depth = 1;
    let mut seen = vec![current.clone()];

    loop {
        if let Some(base) = get_base_fn(&current) {
            if seen.contains(&base) {
                let prefix = "    ".repeat(depth - 1);
                lines.push(format!("{}└── {} {}", prefix.dimmed(), base.blue(), "(cycle detected)".red()));
                break;
            }

            let prefix = "    ".repeat(depth - 1);
            lines.push(format!("{}└── {}", prefix.dimmed(), base.blue()));
            seen.push(base.clone());
            current = base;
            depth += 1;
        } else {
            break;
        }
    }

    lines
}

fn print_tree(branch: Option<&str>) -> Result<()> {
    let start_branch = match branch {
        Some(b) => b.to_string(),
        None => current_branch()?,
    };

    let lines = build_tree_lines(&start_branch, |b| get_base(Some(b)).ok());
    for line in lines {
        println!("{}", line);
    }

    Ok(())
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
) -> Result<()> {
    let mut create_at = "HEAD".to_string();
    
    let base_branch = match base {
        Some(b) => {
            create_at = b.to_string();
            b.to_string()
        },
        None => {
            match current_branch() {
                Ok(b) => {
                    create_at = b.clone();
                    b
                },
                Err(e) => {
                    if e.to_string().contains("detached HEAD") {
                        println!("{} {}", "⚠️".yellow(), "Currently in detached HEAD. Resolving base branch...".yellow());
                        
                        let all_branches = get_all_local_branches();
                        let candidates: Vec<&str> = all_branches.iter().map(|s| s.as_str()).collect();
                        
                        let ranked = rank_closest_bases("HEAD", &candidates);
                        
                        if ranked.is_empty() {
                            return Err(anyhow!("Failed to find any local branches. Please specify one explicitly: `branch-buddy new <name> <base>`"));
                        }
                        
                        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select base branch for metadata (closest match pre-selected)")
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
    };

    // verify base
    if !ref_exists(&base_branch) {
        return Err(anyhow!(
            "Base branch '{}' does not appear to be a valid ref",
            base_branch
        ));
    }

    let slug = slugify(title);

    let mut branch_name = match (r#type, ticket) {
        (Some(t), Some(id)) => format!("{}/{}-{}", t, id, slug),
        (Some(t), None) => format!("{}/{}", t, slug),
        (None, Some(id)) => format!("{}-{}", id, slug),
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
    let mut scored: Vec<(String, usize)> = candidates.iter().filter_map(|&cand| {
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
            let mb_sha = String::from_utf8(output.stdout).unwrap_or_default().trim().to_string();
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
                    .unwrap_or(std::usize::MAX);
                Some((cand.to_string(), dist))
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

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
            println!("💾 Saved base {} for branch {}", base.blue(), target_branch.green());
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

    let hook_content = format!(r#"#!/bin/bash
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
"#, health_check_snippet);

    if hook_path.exists() {
        let content = std::fs::read_to_string(&hook_path)?;
        if !content.contains("branch-buddy") {
            return Err(anyhow!(
                "A post-checkout hook already exists at {}. Please merge the branch-buddy hook manually.",
                hook_path.display()
            ));
        } else {
            println!("✅ branch-buddy post-checkout hook is {} installed.", "already".yellow());
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

    println!("🎉 {} post-checkout hook at {}", "Successfully installed".green(), hook_path.display());
    
    if !enable_health_check {
        println!("\n💡 {}: You can also enable automatic health checks that warn you about broken base branch links.", "Tip".yellow().bold());
        println!("Run `{}` to enable them!", "branch-buddy doctor --install-hook".cyan());
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
        if let Ok(base) = get_base(Some(branch)) {
            if !ref_exists(&base) {
                println!("⚠️  Branch '{}' points to missing base '{}'", branch.yellow(), base.red());
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
    }

    if broken_count == 0 {
        println!("🩺 Repository is perfectly healthy! All base branch links are {}.", "intact".green());
    } else if !fix {
        println!(
            "\nFound {} broken link(s). Run `{}` to auto-heal them.",
            broken_count.to_string().red().bold(),
            "branch-buddy doctor --fix".cyan()
        );
    } else {
        println!("\nDoctor finished repairing {} broken link(s).", broken_count.to_string().green().bold());
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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
        Commands::Tree { branch } => {
            print_tree(branch.as_deref())?;
        }
        Commands::Doctor { fix, install_hook } => {
            doctor(*fix, *install_hook)?;
        }
        Commands::InstallHooks => {
            do_install_hook(false)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Fix login bug"), "fix-login-bug");
        assert_eq!(slugify("User Signup (New Flow)!"), "user-signup-new-flow");
        let empty = slugify("   ");
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
        assert_eq!(lines[0], "feature/my-branch");
        assert_eq!(lines[1], "└── dev");
        assert_eq!(lines[2], "    └── main");
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
        assert_eq!(lines[0], "A");
        assert_eq!(lines[1], "└── B");
        assert_eq!(lines[2], "    └── C");
        assert_eq!(lines[3], "        └── A (cycle detected)");
    }
}
