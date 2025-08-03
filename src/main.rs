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
                let file = std::fs::read_to_string(file.as_ref().expect("file required"))
                    .expect("read file");
                let json = blueprint::blueprint_to_json(&file);
                let counts = blueprint::count_entities::count(&json);
                if *to_blueprint {
                    let mut counts = counts
                        .into_iter()
                        .map(|(name, count)| (name, i64::try_from(count).unwrap()))
                        .collect::<Vec<_>>();
                    counts.sort_by_key(|(_name, count)| -count);
                    dbg!(&counts);
                    let combinator = blueprint::make_constant_combinator_json(counts);
                    let bp = blueprint::json_to_blueprint(combinator);
                    println!("{bp}");
                } else {
                    println!("{counts:?}");
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    args.command.run();
}
