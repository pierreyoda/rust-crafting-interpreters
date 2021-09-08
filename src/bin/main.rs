use std::path::Path;

use clap::clap_app;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // TODO: add Lox code as input support
    let matches = clap_app!(rust_crafting_interpreters =>
        (version: VERSION)
        (author: "pierreyoda <pierreyoda@users.noreply.github.com>")
        (about: "Crafting Interpreters - Lox interpreter implementations (both tree-walk and bytecode-based) in Rust")
        (@arg INPUT: +required "Lox source file to execute.")
        (@arg tree_walk_version: -t --tree-walk "Use the first interpreter version (tree-walk interpreter)")
        (@subcommand repl =>
            (about: "Start an interactive REPL session in Lox.")
            (version: VERSION)
            (@arg tree_walk_version: -t --tree-walk "Use the first interpreter version (tree-walk interpreter)")
        )
    )
    .get_matches();

    if let Some(repl_matches) = matches.subcommand_matches("repl") {
        // TODO: REPL
    } else {
        let input_file: String = matches.value_of_t("INPUT").unwrap(); // TODO: errors handling
        let input_filepath = Path::new(&input_file);
        println!("{:?}", input_filepath);
    }
}
