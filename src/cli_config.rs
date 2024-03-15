use crate::RenamerArgs;

/// Renamer CLI settings.
#[derive(Default, Debug)]
pub struct CliConfig {
    pub force: bool,
    pub rename_files: bool,
    pub sort_files: bool,
    pub print_only: bool,
    pub tags_only: bool,
    pub verbose: bool,
    pub debug: bool,
    pub test_mode: bool,
    pub log_failures: bool,
}

impl CliConfig {
    /// Create config from command line args.
    pub fn from_args(args: RenamerArgs) -> Self {
        CliConfig {
            force: args.force,
            rename_files: args.rename,
            sort_files: args.sort,
            print_only: args.print,
            tags_only: args.tags_only,
            verbose: args.verbose,
            debug: args.debug,
            test_mode: args.test,
            log_failures: args.log,
        }
    }

    #[cfg(test)]
    /// Used in tests.
    pub fn new_for_tests() -> Self {
        CliConfig {
            force: true,
            rename_files: true,
            sort_files: false,
            print_only: false,
            tags_only: false,
            verbose: false,
            debug: false,
            test_mode: true,
            log_failures: false,
        }
    }
}
