use clap::{crate_version, App};
use std::str::FromStr;
use strum;
use strum_macros::{Display, EnumString};

fn main() {
    let matches = App::new("sxhkd-mode")
        .version(crate_version!())
        .arg("<MODE> 'The new sxhkd mode")
        .get_matches();

    #[derive(EnumString, Display)]
    enum Modes {
        Emacs,
        Fundemental,
        APL,
    };

    let mode = Modes::from_str(matches.value_of("MODE").expect("required")).unwrap_or_else(|_| {
        eprintln!("Unrecognized mode!");
        std::process::exit(1);
    });

    println!("Hello, world? mode: {}", mode);
}
