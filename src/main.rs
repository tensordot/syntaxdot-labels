use std::io::stdout;

use anyhow::Result;
use clap::{crate_version, App, AppSettings, Arg, Shell, SubCommand};

mod subcommands;

pub mod traits;
use traits::SyntaxDotApp;

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
    AppSettings::SubcommandRequiredElseHelp,
];

fn main() -> Result<()> {
    // Known subapplications.
    let apps = vec![subcommands::Dependency::app()];

    let cli = App::new("syntaxdot")
        .settings(DEFAULT_CLAP_SETTINGS)
        .about("SyntaxDot label converters")
        .version(crate_version!())
        .subcommands(apps)
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate completion scripts for your shell")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(Arg::with_name("shell").possible_values(&Shell::variants())),
        );
    let matches = cli.clone().get_matches();

    match matches.subcommand_name().unwrap() {
        "completions" => {
            let shell = matches
                .subcommand_matches("completions")
                .unwrap()
                .value_of("shell")
                .unwrap();
            write_completion_script(cli, shell.parse::<Shell>().unwrap());
            Ok(())
        }
        "dependency" => {
            subcommands::Dependency::parse(matches.subcommand_matches("dependency").unwrap())?
                .run()
        }
        _unknown => unreachable!(),
    }
}

fn write_completion_script(mut cli: App, shell: Shell) {
    cli.gen_completions_to("syntaxdot", shell, &mut stdout());
}
