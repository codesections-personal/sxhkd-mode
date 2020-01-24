use clap::{crate_name, crate_version, App, Arg};
use std::str::FromStr;
use strum_macros::{Display, EnumString, EnumVariantNames};
use utils::{dependencies, sh};

#[derive(EnumString, Display, EnumVariantNames)]
enum Modes {
    Emacs,
    Fundemental,
    APL,
    Firefox,
}

fn main() {
    let matches = App::new(crate_name!())
        .about("Update the current_mode symlink to the specified sxhkd mode file.")
        .version(crate_version!())
        .arg(
            Arg::with_name("MODE")
                .required(true)
                .help("The new sxhkd mode"),
        )
        .get_matches();
    dependencies(crate_name!(), &["ln", "pkill"]);

    let mut mode = matches.value_of("MODE").expect("required");
    if mode.contains("Mozilla Firefox") || mode.contains("TigerVNC") {
        mode = "Firefox"
    };

    let mode = Modes::from_str(mode).unwrap_or(Modes::Fundemental);
    let target_mode = match mode {
        Modes::Emacs => "emacs_mode",
        Modes::APL => "apl_mode",
        Modes::Firefox => "firefox_mode",
        _ => "empty_mode",
    };
    sh(&format!(
        r#"ln --symbolic --no-dereference --force {sxhkd_dir}/{mode} {sxhkd_dir}/current_mode"#,
        sxhkd_dir = "/home/dsock/.config/sxhkd",
        mode = target_mode
    ));
    sh("pkill -USR1 -x sxhkd");
}
