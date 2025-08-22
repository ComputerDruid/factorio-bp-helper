mod blueprint;
mod json_walk;
mod save;

use std::fmt::Write;
use std::io::{Read, stderr, stdin};
use std::str::FromStr;

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

    /// Upgrades the quality of recipies/filters/conditions without upgrading entities/modules
    UpgradeQuality {
        /// Sends the output to the clipboard
        #[arg(long)]
        to_clipboard: bool,
        blueprint_string: Option<String>,
    },
    /// Saves blueprint as a .json file, or as a directory of json files if it's a blueprint book.
    Save { blueprint_string: Option<String> },
}

mod terminal;

impl Commands {
    fn run(self) {
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
                let mut counts = counts
                    .into_iter()
                    .map(|(key, count)| (key, i64::try_from(count).unwrap()))
                    .collect::<Vec<_>>();
                counts.sort_by_key(|(__key, count)| -count);
                if to_blueprint {
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
                    let mut formatted = String::new();
                    for ((name, quality), count) in counts {
                        writeln!(
                            formatted,
                            "{count} {name}{quality}",
                            quality = quality.fmt_suffix()
                        )
                        .expect("a Display implementation returned an error unexpectedly");
                    }
                    let formatted = formatted.trim_end();
                    if to_clipboard {
                        crossterm::execute!(
                            stderr(),
                            CopyToClipboard::to_clipboard_from(formatted)
                        )
                        .unwrap();
                        println!("counts copied to clipboard.");
                    } else {
                        println!("{formatted}");
                    }
                }
            }
            Commands::Unwrap { blueprint_string } => {
                let blueprint_string = blueprint_string.unwrap_or_else(terminal::prompt_blueprint);
                let json = blueprint_to_json(&blueprint_string);
                println!("{json}");
            }
            Commands::Wrap {} => {
                let mut buf = String::new();
                stdin().read_to_string(&mut buf).unwrap();
                let blueprint_string = json_to_blueprint(serde_json::from_str(&buf).unwrap());
                println!("{blueprint_string}");
            }
            Commands::UpgradeQuality {
                to_clipboard,
                blueprint_string,
            } => {
                let blueprint_string = if let Some(blueprint_string) = blueprint_string {
                    blueprint_string
                } else {
                    terminal::prompt_blueprint()
                };
                let json = blueprint::blueprint_to_json(&blueprint_string);
                let json: serde_json::Value =
                    serde_json::from_str(&json).expect("blueprint should contain valid json");
                let bp = blueprint::json_to_blueprint(blueprint::upgrade_quality::upgrade(json));
                if to_clipboard {
                    crossterm::execute!(stderr(), CopyToClipboard::to_clipboard_from(bp)).unwrap();
                    println!("blueprint copied to clipboard.")
                } else {
                    println!("{bp}");
                }
            }
            Commands::Save { blueprint_string } => {
                let blueprint_string = blueprint_string.unwrap_or_else(terminal::prompt_blueprint);
                let json = blueprint_to_json(&blueprint_string);
                let json = serde_json::Value::from_str(&json).expect("should contain valid json");

                save::save(json, None);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    args.command.run();
}
