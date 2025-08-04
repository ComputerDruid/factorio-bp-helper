mod blueprint;

use std::io::{Read, stderr, stdin};

use blueprint::{blueprint_to_json, json_to_blueprint};
use clap::{Parser, Subcommand};
use crossterm::clipboard::CopyToClipboard;

/// Collection of factorio blueprint helpers
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Counts the number of items needed to construct the blueprint.
    CountEntities {
        /// Outputs the counts as a blueprint of a constant combinator
        #[arg(long)]
        to_blueprint: bool,
        /// Sends the output to the clipboard
        #[arg(long)]
        to_clipboard: bool,
        blueprint_string: Option<String>,
    },
    /// Unwraps a blueprint string to reveal the json representation.
    Unwrap { blueprint_string: Option<String> },

    /// Wraps json from stdin into a blueprint string.
    Wrap {},
}

mod terminal;

impl Commands {
    fn run(self) -> () {
        match self {
            Commands::CountEntities {
                to_blueprint,
                to_clipboard,
                blueprint_string,
            } => {
                let blueprint_string = if let Some(blueprint_string) = blueprint_string {
                    blueprint_string
                } else {
                    terminal::prompt_blueprint()
                };
                let json = blueprint::blueprint_to_json(&blueprint_string);
                let counts = blueprint::count_entities::count(&json);
                if to_blueprint {
                    let mut counts = counts
                        .into_iter()
                        .map(|(name, count)| (name, i64::try_from(count).unwrap()))
                        .collect::<Vec<_>>();
                    counts.sort_by_key(|(_name, count)| -count);
                    let combinator = blueprint::make_constant_combinator_json(counts);
                    let bp = blueprint::json_to_blueprint(combinator);
                    if to_clipboard {
                        crossterm::execute!(stderr(), CopyToClipboard::to_clipboard_from(bp))
                            .unwrap();
                        println!("blueprint copied to clipboard.")
                    } else {
                        println!("{bp}");
                    }
                } else {
                    if to_clipboard {
                        crossterm::execute!(
                            stderr(),
                            CopyToClipboard::to_clipboard_from(format!("{counts:?}"))
                        )
                        .unwrap();
                        println!("counts copied to clipboard.");
                    } else {
                        println!("{counts:?}");
                    }
                }
            }
            Commands::Unwrap { blueprint_string } => {
                let blueprint_string = blueprint_string.unwrap_or_else(terminal::prompt_blueprint);
                let json = blueprint_to_json(&blueprint_string);
                println!("{}", json);
            }
            Commands::Wrap {} => {
                let mut buf = String::new();
                stdin().read_to_string(&mut buf).unwrap();
                let blueprint_string = json_to_blueprint(serde_json::from_str(&buf).unwrap());
                println!("{}", blueprint_string);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    args.command.run();
}
