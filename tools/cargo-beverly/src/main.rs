//! `cargo-beverly` is a Cargo subcommand (`cargo beverly ...`) for Beverly app
//! developer workflows. See [`publish`] for the `cargo beverly publish` pipeline.

use clap::{Parser, Subcommand};

mod publish;

#[derive(Parser)]
#[command(name = "cargo-beverly", bin_name = "cargo-beverly")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build a release binary and package it for distribution.
    Publish(publish::PublishArgs),
}

fn main() {
    // Cargo invokes subcommands as `cargo-beverly beverly <args>`, re-passing the
    // subcommand name. Strip it so the binary also works when run directly
    // (e.g. `cargo run --bin cargo-beverly -- publish`) during development.
    let mut args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("beverly") {
        args.remove(1);
    }

    let cli = Cli::parse_from(args);
    let result = match cli.command {
        Command::Publish(args) => publish::run(&args),
    };

    if let Err(err) = result {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
