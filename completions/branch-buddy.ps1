
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'branch-buddy' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'branch-buddy'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'branch-buddy' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Create a new branch with a slugified name and set its base')
            [CompletionResult]::new('get-base', 'get-base', [CompletionResultType]::ParameterValue, 'Get the base branch for the specified branch (or current branch)')
            [CompletionResult]::new('set-base', 'set-base', [CompletionResultType]::ParameterValue, 'Set the base branch for a branch')
            [CompletionResult]::new('has-base', 'has-base', [CompletionResultType]::ParameterValue, 'Check if a branch has a base set (exits 0 if true, 1 otherwise)')
            [CompletionResult]::new('guess-base', 'guess-base', [CompletionResultType]::ParameterValue, 'Guess the base branch for a branch')
            [CompletionResult]::new('tree', 'tree', [CompletionResultType]::ParameterValue, 'Show the branch ancestry tree')
            [CompletionResult]::new('install-hooks', 'install-hooks', [CompletionResultType]::ParameterValue, 'Install git hooks (post-checkout) to automatically track branches')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Check repository for broken base branch links and optionally fix them')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show focused commit log between branch and base (or stack)')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Scaffold a new .branchbuddy.toml configuration file')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show branch health report (base, ahead/behind, staleness, diff stat)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'branch-buddy;new' {
            [CompletionResult]::new('--base', '--base', [CompletionResultType]::ParameterName, 'Base branch (defaults to current branch)')
            [CompletionResult]::new('--type', '--type', [CompletionResultType]::ParameterName, 'Optional prefix type (e.g., ''feature'', ''bugfix'')')
            [CompletionResult]::new('--ticket', '--ticket', [CompletionResultType]::ParameterName, 'Optional ticket ID (e.g., ''ABC-123'')')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Perform a dry run without creating the branch')
            [CompletionResult]::new('--no-checkout', '--no-checkout', [CompletionResultType]::ParameterName, 'Create the branch but do not check it out')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output results as JSON')
            [CompletionResult]::new('--fail-if-exists', '--fail-if-exists', [CompletionResultType]::ParameterName, 'Fail if branch already exists instead of appending a numeric suffix')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;get-base' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;set-base' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--no-validate', '--no-validate', [CompletionResultType]::ParameterName, 'Skip validating that the base is a valid ref')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;has-base' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;guess-base' {
            [CompletionResult]::new('--candidates', '--candidates', [CompletionResultType]::ParameterName, 'candidates')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--write', '--write', [CompletionResultType]::ParameterName, 'write')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;tree' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--no-legend', '--no-legend', [CompletionResultType]::ParameterName, 'Hide the branch tree legend at the bottom')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;install-hooks' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;doctor' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--fix', '--fix', [CompletionResultType]::ParameterName, 'Automatically attempt to fix broken links using guess-base')
            [CompletionResult]::new('--install-hook', '--install-hook', [CompletionResultType]::ParameterName, 'Install the post-checkout git hook for automatic health checks')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;log' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Limit number of commits displayed')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Limit number of commits displayed')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--stack', '--stack', [CompletionResultType]::ParameterName, 'Include commits across all parent base branches up to trunk()')
            [CompletionResult]::new('--stat', '--stat', [CompletionResultType]::ParameterName, 'Show file diff statistics for each commit')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;init' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Create global configuration file at ~/.config/branchbuddy/config.toml')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Overwrite existing configuration file if present')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Overwrite existing configuration file if present')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'Run interactive wizard to prompt for configuration options')
            [CompletionResult]::new('--interactive', '--interactive', [CompletionResultType]::ParameterName, 'Run interactive wizard to prompt for configuration options')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;completions' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;status' {
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to custom configuration file')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Emit JSON instead of human-readable output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'branch-buddy;help' {
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Create a new branch with a slugified name and set its base')
            [CompletionResult]::new('get-base', 'get-base', [CompletionResultType]::ParameterValue, 'Get the base branch for the specified branch (or current branch)')
            [CompletionResult]::new('set-base', 'set-base', [CompletionResultType]::ParameterValue, 'Set the base branch for a branch')
            [CompletionResult]::new('has-base', 'has-base', [CompletionResultType]::ParameterValue, 'Check if a branch has a base set (exits 0 if true, 1 otherwise)')
            [CompletionResult]::new('guess-base', 'guess-base', [CompletionResultType]::ParameterValue, 'Guess the base branch for a branch')
            [CompletionResult]::new('tree', 'tree', [CompletionResultType]::ParameterValue, 'Show the branch ancestry tree')
            [CompletionResult]::new('install-hooks', 'install-hooks', [CompletionResultType]::ParameterValue, 'Install git hooks (post-checkout) to automatically track branches')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Check repository for broken base branch links and optionally fix them')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show focused commit log between branch and base (or stack)')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Scaffold a new .branchbuddy.toml configuration file')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show branch health report (base, ahead/behind, staleness, diff stat)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'branch-buddy;help;new' {
            break
        }
        'branch-buddy;help;get-base' {
            break
        }
        'branch-buddy;help;set-base' {
            break
        }
        'branch-buddy;help;has-base' {
            break
        }
        'branch-buddy;help;guess-base' {
            break
        }
        'branch-buddy;help;tree' {
            break
        }
        'branch-buddy;help;install-hooks' {
            break
        }
        'branch-buddy;help;doctor' {
            break
        }
        'branch-buddy;help;log' {
            break
        }
        'branch-buddy;help;init' {
            break
        }
        'branch-buddy;help;completions' {
            break
        }
        'branch-buddy;help;status' {
            break
        }
        'branch-buddy;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
