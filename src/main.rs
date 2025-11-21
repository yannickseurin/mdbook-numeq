use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook_numeq::NumEqPreprocessor;
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::{parse_input, Preprocessor, MDBOOK_VERSION};
use semver::{Version, VersionReq};
use std::io;

/// Parse CLI options.
pub fn make_app() -> Command {
    Command::new("mdbook-numeq")
        .version(crate_version!())
        .about("An mdbook preprocessor that automatically numbers centered equations")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = parse_input(io::stdin())?;

    let pre = NumEqPreprocessor::new(&ctx);

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(sub_args: &ArgMatches) -> Result<()> {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");

    let pre = NumEqPreprocessor::default();

    let supported = pre.supports_renderer(renderer).unwrap_or(false);

    if supported {
        Ok(())
    } else {
        Err(Error::msg(format!(
            "The {} preprocessor does not support the '{}' renderer",
            pre.name(),
            renderer,
        )))
    }
}

fn main() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_env_var("MDBOOK_LOG")
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        handle_supports(sub_args)
    } else {
        // handle preprocessing
        handle_preprocessing()
    }
}
