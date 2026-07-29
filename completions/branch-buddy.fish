# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_branch_buddy_global_optspecs
    string join \n config= h/help
end

function __fish_branch_buddy_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_branch_buddy_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_branch_buddy_using_subcommand
    set -l cmd (__fish_branch_buddy_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "new" -d 'Create a new branch with a slugified name and set its base'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "get-base" -d 'Get the base branch for the specified branch (or current branch)'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "set-base" -d 'Set the base branch for a branch'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "has-base" -d 'Check if a branch has a base set (exits 0 if true, 1 otherwise)'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "guess-base" -d 'Guess the base branch for a branch'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "tree" -d 'Show the branch ancestry tree'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "install-hooks" -d 'Install git hooks (post-checkout) to automatically track branches'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "doctor" -d 'Check repository for broken base branch links and optionally fix them'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "log" -d 'Show focused commit log between branch and base (or stack)'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "init" -d 'Scaffold a new .branchbuddy.toml configuration file'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "status" -d 'Show branch health report (base, ahead/behind, staleness, diff stat)'
complete -c branch-buddy -n "__fish_branch_buddy_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l base -d 'Base branch (defaults to current branch)' -r
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l type -d 'Optional prefix type (e.g., \'feature\', \'bugfix\')' -r
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l ticket -d 'Optional ticket ID (e.g., \'ABC-123\')' -r
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l dry-run -d 'Perform a dry run without creating the branch'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l no-checkout -d 'Create the branch but do not check it out'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l json -d 'Output results as JSON'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -l fail-if-exists -d 'Fail if branch already exists instead of appending a numeric suffix'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand new" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand get-base" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand get-base" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand set-base" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand set-base" -l no-validate -d 'Skip validating that the base is a valid ref'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand set-base" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand has-base" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand has-base" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand guess-base" -l candidates -r
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand guess-base" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand guess-base" -l write
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand guess-base" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand tree" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand tree" -l no-legend -d 'Hide the branch tree legend at the bottom'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand tree" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand install-hooks" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand install-hooks" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand doctor" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand doctor" -l fix -d 'Automatically attempt to fix broken links using guess-base'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand doctor" -l install-hook -d 'Install the post-checkout git hook for automatic health checks'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand log" -s n -l limit -d 'Limit number of commits displayed' -r
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand log" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand log" -l stack -d 'Include commits across all parent base branches up to trunk()'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand log" -l stat -d 'Show file diff statistics for each commit'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand log" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand init" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand init" -l global -d 'Create global configuration file at ~/.config/branchbuddy/config.toml'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand init" -s f -l force -d 'Overwrite existing configuration file if present'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand init" -s i -l interactive -d 'Run interactive wizard to prompt for configuration options'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand init" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand completions" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand completions" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand status" -l config -d 'Path to custom configuration file' -r -F
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand status" -l json -d 'Emit JSON instead of human-readable output'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand status" -s h -l help -d 'Print help'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "new" -d 'Create a new branch with a slugified name and set its base'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "get-base" -d 'Get the base branch for the specified branch (or current branch)'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "set-base" -d 'Set the base branch for a branch'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "has-base" -d 'Check if a branch has a base set (exits 0 if true, 1 otherwise)'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "guess-base" -d 'Guess the base branch for a branch'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "tree" -d 'Show the branch ancestry tree'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "install-hooks" -d 'Install git hooks (post-checkout) to automatically track branches'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "doctor" -d 'Check repository for broken base branch links and optionally fix them'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "log" -d 'Show focused commit log between branch and base (or stack)'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "init" -d 'Scaffold a new .branchbuddy.toml configuration file'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "status" -d 'Show branch health report (base, ahead/behind, staleness, diff stat)'
complete -c branch-buddy -n "__fish_branch_buddy_using_subcommand help; and not __fish_seen_subcommand_from new get-base set-base has-base guess-base tree install-hooks doctor log init completions status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
