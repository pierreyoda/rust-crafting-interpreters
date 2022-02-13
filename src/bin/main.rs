use std::{fs::read_to_string, path::Path};

use clap::{Parser, Subcommand};

use rust_crafting_interpreters_lib::{
    errors::{LoxInterpreterError, Result},
    interpreter::{LoxInterpreter, LoxTreeWalkInterpreter},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[clap(
    version = VERSION,
    author = "pierreyoda <pierreyoda@users.noreply.github.com>", 
    about = "Crafting Interpreters - Lox interpreter implementations (both tree-walk and bytecode-based) in Rust",
)]
struct CLIArgs {
    input: String,

    #[clap(subcommand)]
    command: Option<CLICommands>,
}

#[derive(Debug, Subcommand)]
enum CLICommands {
    /// Start an interactive REPL session in Lox
    REPL {
        /// Use the first interpreter version (tree-walk interpreter).
        #[clap(short, long)]
        tree_walk_version: bool,
    },
}

fn main() -> Result<()> {
    let cli_args = CLIArgs::parse();
    match &cli_args.command {
        Some(CLICommands::REPL {
            tree_walk_version: _,
        }) => {
            // TODO: REPL
            Ok(())
        }
        _ => {
            let input_file = cli_args.input;
            let input_filepath = Path::new(&input_file);
            let input_source =
                read_to_string(input_filepath).map_err(LoxInterpreterError::IOError)?;
            let mut interpreter = LoxTreeWalkInterpreter::new(None);
            let parsed_operations = interpreter.parse(input_source)?;
            let _ = interpreter.interpret(&parsed_operations)?;
            Ok(())
        }
    }
}
