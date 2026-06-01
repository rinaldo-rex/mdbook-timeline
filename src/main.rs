use mdbook_timeline::TimelinePreprocessor;
use clap::{Arg, ArgMatches, Command};
use mdbook_preprocessor::Preprocessor;
use std::{io, process};

fn make_app() -> Command {
    Command::new("timeline-preprocessor")
        .about("A mdBook preprocessor that renders interactive timelines")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let preprocessor = TimelinePreprocessor::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{e:?}");
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> anyhow::Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;
    let processed = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;
    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer).unwrap_or(false);
    process::exit(if supported { 0 } else { 1 });
}
