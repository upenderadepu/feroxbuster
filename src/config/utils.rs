use crate::{
    utils::{module_colorizer, status_colorizer},
    DEFAULT_STATUS_CODES, DEFAULT_WORDLIST, VERSION,
};
#[cfg(not(test))]
use std::process::exit;

/// simple helper to clean up some code reuse below; panics under test / exits in prod
pub(super) fn report_and_exit(err: &str) -> ! {
    eprintln!(
        "{} {}: {}",
        status_colorizer("ERROR"),
        module_colorizer("Configuration::new"),
        err
    );

    #[cfg(test)]
    panic!();
    #[cfg(not(test))]
    exit(1);
}

// functions timeout, threads, status_codes, user_agent, wordlist, save_state, and depth are used to provide
// defaults in the event that a ferox-config.toml is found but one or more of the values below
// aren't listed in the config.  This way, we get the correct defaults upon Deserialization

/// default Configuration type for use in json output
pub(super) fn serialized_type() -> String {
    String::from("configuration")
}

/// default timeout value
pub(super) fn timeout() -> u64 {
    7
}

/// default save_state value
pub(super) fn save_state() -> bool {
    true
}

/// default threads value
pub(super) fn threads() -> usize {
    50
}

/// default status codes
pub(super) fn status_codes() -> Vec<u16> {
    DEFAULT_STATUS_CODES
        .iter()
        .map(|code| code.as_u16())
        .collect()
}

/// default wordlist
pub(super) fn wordlist() -> String {
    String::from(DEFAULT_WORDLIST)
}

/// default user-agent
pub(super) fn user_agent() -> String {
    format!("feroxbuster/{}", VERSION)
}

/// default recursion depth
pub(super) fn depth() -> usize {
    4
}

/// enum representing the three possible states for informational output (not logging verbosity)
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OutputLevel {
    /// normal scan, no --quiet|--silent
    Default,

    /// quiet scan, print some information, but not all (new in versions >= 2.0.0)
    Quiet,

    /// silent scan, only print urls (used to be --quiet in versions 1.x.x)
    Silent,
}

/// implement a default for OutputLevel
impl Default for OutputLevel {
    /// return Default
    fn default() -> Self {
        Self::Default
    }
}

/// given the current settings for quiet and silent, determine output_level (DRY helper)
pub fn determine_output_level(quiet: bool, silent: bool) -> OutputLevel {
    if quiet && silent {
        // user COULD have both as true in config file, take the more quiet of the two
        OutputLevel::Silent
    } else if quiet {
        OutputLevel::Quiet
    } else if silent {
        OutputLevel::Silent
    } else {
        OutputLevel::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// test determine_output_level returns higher of the two levels if both given values are true
    fn determine_output_level_returns_correct_results() {
        let mut level = determine_output_level(true, true);
        assert_eq!(level, OutputLevel::Silent);

        level = determine_output_level(false, true);
        assert_eq!(level, OutputLevel::Silent);

        level = determine_output_level(false, false);
        assert_eq!(level, OutputLevel::Default);

        level = determine_output_level(true, false);
        assert_eq!(level, OutputLevel::Quiet);
    }

    #[test]
    #[should_panic]
    /// report_and_exit should panic/exit when called
    fn report_and_exit_panics_under_test() {
        report_and_exit("test");
    }
}
