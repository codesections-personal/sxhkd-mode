use atty::Stream;
use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches};
use config;
use dirs;
use regex::Regex;
use run_script::spawn_script;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
};
use utils::{dependencies, sh, Die};

fn main() {
    let sxhkd_dir = dirs::config_dir()
        .ok_or("Could not open config directory")
        .unwrap_or_die()
        .join("sxhkd");
    let sxhkd_dir = sxhkd_dir.to_str().expect("Valid unicode in path");

    #[rustfmt::skip]
    let cli = App::new(crate_name!())
        .about("Manage the current sxhkd mode.")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .subcommand(App::new("set")
                    .about("Set the current sxhkd mode")
                    .arg(Arg::with_name("TARGET_MODE")
                         .required(true)
                         .help("The new mode to set")))
        .subcommand(App::new("show")
                    .about("show the current sxhkd mode"))
        .subcommand(App::new("auto")
                    .about("Automatically update the current sxhkd mode based on the current xtitle \
and rules in a configuration file")
                    .arg(Arg::with_name("CONFIG")
                         .short('c')
                         .long("config")
                         .default_value(&format!("{}/modes.toml", sxhkd_dir))
                         .help("The TOML configuration file with rules for setting sxhkd modes \
(mode/regular-expression pairs)")))
        .subcommand(App::new("src").about("--src 'Prints this program's source to stdout'"))
        .get_matches();

    run(cli, sxhkd_dir).unwrap_or_die();
}
fn run(cli: ArgMatches, sxhkd_dir: &str) -> Result<(), Box<dyn Error>> {
    let cache_dir = dirs::cache_dir().ok_or("Could not open config directory")?;
    let cache_dir = cache_dir.to_str().expect("Valid unicode in path");

    match cli.subcommand_name() {
        Some("src") => {
            print!("/// main.rs\n{}", include_str!("main.rs"));
        }
        Some("set") => {
            let cli = cli.subcommand_matches("set").expect("guaranteed by match");
            let target_mode = cli.value_of("TARGET_MODE").expect("guaranteed by clap");
            update_mode(&sxhkd_dir, target_mode)?;
        }
        Some("show") => {
            dependencies(vec!["find"])?;

            let (path_to_mode, _) = sh(&format!("find {}/current_mode -printf %l", sxhkd_dir))?;
            println!("{}", &path_to_mode[sxhkd_dir.len() + 1..]);
        }
        Some("auto") => {
            let cli = cli.subcommand_matches("auto").expect("required by match");
            dependencies(vec!["rm", "mkfifo", "xtitle"])?;
            let pipe_path = &format!("{}/sxhkd.pipe", cache_dir);

            // check that the pipe exists (and create it if it doesn't)
            if fs::metadata(pipe_path).is_err() {
                sh(&format!("rm {pipe}; mkfifo {pipe}", pipe = pipe_path))?;
                fs::metadata(pipe_path)?;
            }

            // Write into the pipe
            spawn_script!(&format!("xtitle -s > {}/sxhkd.pipe &", cache_dir))?;

            // Read out of the pipe
            let pipe = File::open(&format!("{}/sxhkd.pipe", cache_dir))?;
            let reader = BufReader::new(pipe);

            let rules = get_xtitle_rules_from_config(cli.value_of("CONFIG").expect("default"))?;
            for line in reader.lines() {
                let xtitle = line?;
                let mut target_mode = "empty_mode";
                for (mode, regex) in rules.iter() {
                    if Regex::new(regex)?.is_match(&xtitle) {
                        target_mode = mode;
                        break;
                    }
                }
                update_mode(&sxhkd_dir, target_mode)?;
            }
            unreachable!("Unless something kills the pipe, we should never get here");
        }
        None | Some(_) => unreachable!("Clap requires and validates subcommand"),
    };
    Ok(())
}

fn update_mode(dir: &str, target_mode: &str) -> Result<(), Box<dyn Error>> {
    dependencies(vec!["ln", "killall"])?;
    fs::metadata(&format!("{}/{}", dir, target_mode)).map_err(|_| {
        format!(
            r"`{mode}` is not a valid mode.
Please select a mode with a corresponding file in {dir}",
            mode = target_mode,
            dir = dir
        )
    })?;

    sh(&format!(
        r#"ln --symbolic --no-dereference --force {sxhkd_dir}/{mode} {sxhkd_dir}/current_mode"#,
        sxhkd_dir = dir,
        mode = target_mode
    ))?;

    sh("killall -s USR1 sxhkd")?;
    if atty::is(Stream::Stderr) {
        eprintln!("sxhkd mode is now `{}`", target_mode);
    }
    Ok(())
}

fn get_xtitle_rules_from_config(cfg_file: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut cfg = config::Config::new();
    cfg.merge(config::File::with_name(cfg_file))?;
    Ok(cfg.try_into()?)
}
