mod blueprint;

use clap::{Parser, Subcommand};

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

        file: Option<String>,
    },
}

impl Commands {
    fn run(&self) -> () {
        match self {
            Commands::CountEntities { to_blueprint, file } => {
                assert!(!to_blueprint, "--to-blueprint not supported yet");
                let file = std::fs::read_to_string(file.as_ref().expect("file required"))
                    .expect("read file");
                let json = blueprint::blueprint_to_json(&file);
                dbg!(blueprint::count_entities::count(&json));
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    args.command.run();
}
