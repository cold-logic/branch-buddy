
use builtin;
use str;

set edit:completion:arg-completer[branch-buddy] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'branch-buddy'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'branch-buddy'= {
            cand --config 'Path to custom configuration file'
            cand -h 'Print help'
            cand --help 'Print help'
            cand new 'Create a new branch with a slugified name and set its base'
            cand get-base 'Get the base branch for the specified branch (or current branch)'
            cand set-base 'Set the base branch for a branch'
            cand has-base 'Check if a branch has a base set (exits 0 if true, 1 otherwise)'
            cand guess-base 'Guess the base branch for a branch'
            cand tree 'Show the branch ancestry tree'
            cand install-hooks 'Install git hooks (post-checkout) to automatically track branches'
            cand doctor 'Check repository for broken base branch links and optionally fix them'
            cand log 'Show focused commit log between branch and base (or stack)'
            cand init 'Scaffold a new .branchbuddy.toml configuration file'
            cand completions 'Generate shell completion scripts'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'branch-buddy;new'= {
            cand --base 'Base branch (defaults to current branch)'
            cand --type 'Optional prefix type (e.g., ''feature'', ''bugfix'')'
            cand --ticket 'Optional ticket ID (e.g., ''ABC-123'')'
            cand --config 'Path to custom configuration file'
            cand --dry-run 'Perform a dry run without creating the branch'
            cand --no-checkout 'Create the branch but do not check it out'
            cand --json 'Output results as JSON'
            cand --fail-if-exists 'Fail if branch already exists instead of appending a numeric suffix'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;get-base'= {
            cand --config 'Path to custom configuration file'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;set-base'= {
            cand --config 'Path to custom configuration file'
            cand --no-validate 'Skip validating that the base is a valid ref'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;has-base'= {
            cand --config 'Path to custom configuration file'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;guess-base'= {
            cand --candidates 'candidates'
            cand --config 'Path to custom configuration file'
            cand --write 'write'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;tree'= {
            cand --config 'Path to custom configuration file'
            cand --no-legend 'Hide the branch tree legend at the bottom'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;install-hooks'= {
            cand --config 'Path to custom configuration file'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;doctor'= {
            cand --config 'Path to custom configuration file'
            cand --fix 'Automatically attempt to fix broken links using guess-base'
            cand --install-hook 'Install the post-checkout git hook for automatic health checks'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;log'= {
            cand -n 'Limit number of commits displayed'
            cand --limit 'Limit number of commits displayed'
            cand --config 'Path to custom configuration file'
            cand --stack 'Include commits across all parent base branches up to trunk()'
            cand --stat 'Show file diff statistics for each commit'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;init'= {
            cand --config 'Path to custom configuration file'
            cand --global 'Create global configuration file at ~/.config/branchbuddy/config.toml'
            cand -f 'Overwrite existing configuration file if present'
            cand --force 'Overwrite existing configuration file if present'
            cand -i 'Run interactive wizard to prompt for configuration options'
            cand --interactive 'Run interactive wizard to prompt for configuration options'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;completions'= {
            cand --config 'Path to custom configuration file'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'branch-buddy;help'= {
            cand new 'Create a new branch with a slugified name and set its base'
            cand get-base 'Get the base branch for the specified branch (or current branch)'
            cand set-base 'Set the base branch for a branch'
            cand has-base 'Check if a branch has a base set (exits 0 if true, 1 otherwise)'
            cand guess-base 'Guess the base branch for a branch'
            cand tree 'Show the branch ancestry tree'
            cand install-hooks 'Install git hooks (post-checkout) to automatically track branches'
            cand doctor 'Check repository for broken base branch links and optionally fix them'
            cand log 'Show focused commit log between branch and base (or stack)'
            cand init 'Scaffold a new .branchbuddy.toml configuration file'
            cand completions 'Generate shell completion scripts'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'branch-buddy;help;new'= {
        }
        &'branch-buddy;help;get-base'= {
        }
        &'branch-buddy;help;set-base'= {
        }
        &'branch-buddy;help;has-base'= {
        }
        &'branch-buddy;help;guess-base'= {
        }
        &'branch-buddy;help;tree'= {
        }
        &'branch-buddy;help;install-hooks'= {
        }
        &'branch-buddy;help;doctor'= {
        }
        &'branch-buddy;help;log'= {
        }
        &'branch-buddy;help;init'= {
        }
        &'branch-buddy;help;completions'= {
        }
        &'branch-buddy;help;help'= {
        }
    ]
    $completions[$command]
}
