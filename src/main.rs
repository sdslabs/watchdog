extern crate clap;

mod auth;
mod ssh;
mod su;
mod sudo;
mod update;

use clap::{App, AppSettings, Arg, SubCommand};

use auth::handle_auth;
use lib::errors::Error;
use lib::logger::{handle_logs_all, handle_logs_for, init_logger, LogTarget};
use log::{error, info};
use ssh::handle_ssh;
use su::handle_su;
use sudo::handle_sudo;
use update::handle_update;

fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Watchdog")
        .version("0.1.0")
        .author("SDSLabs <contact@sdslabs.co>")
        .about("Simple server access management system on a binary")
        .subcommand(
            SubCommand::with_name("logs")
                .about("Fetch logs from watchdog components")
                .arg(Arg::with_name("level")
                    .long("level")
                    .takes_value(true)
                    .help("Filter log level when no component is specified (defaults to 'watchdog')"))
                .subcommand(
                    SubCommand::with_name("all")
                        .about("Logs from whole watchdog")
                        .arg(Arg::with_name("level")
                            .short("l")
                            .long("level")
                            .takes_value(true)
                            .help("Filter by log level: info, warn, error"))
                )
                .subcommand(
                    SubCommand::with_name("update")
                        .about("Logs from watchdog update")
                        .arg(Arg::with_name("level")
                            .short("l")
                            .long("level")
                            .takes_value(true)
                            .help("Filter by log level: info, warn, error"))
                )
                .subcommand(
                    SubCommand::with_name("sudo")
                        .about("Logs from sudo")
                        .arg(Arg::with_name("level")
                            .short("l")
                            .long("level")
                            .takes_value(true)
                            .help("Filter by log level"))
                )
                .subcommand(
                    SubCommand::with_name("su")
                        .about("Logs from su")
                        .arg(Arg::with_name("level")
                            .short("l")
                            .long("level")
                            .takes_value(true)
                            .help("Filter by log level"))
                )
                .subcommand(
                    SubCommand::with_name("ssh")
                        .about("Logs from ssh")
                        .arg(Arg::with_name("level")
                            .short("l")
                            .long("level")
                            .takes_value(true)
                            .help("Filter by log level"))
                )
                .subcommand(
                    SubCommand::with_name("watchdog")
                        .about("Logs from watchdog")
                        .arg(Arg::with_name("level")
                            .short("l")
                            .long("level")
                            .takes_value(true)
                            .help("Filter by log level"))
                )
        )
        .subcommand(SubCommand::with_name("sudo")
            .about("Handles the PAM sudo calls by pam_exec for Watchdog"))
        .subcommand(SubCommand::with_name("su")
            .about("Handles the PAM su calls by pam_exec for Watchdog"))
        .subcommand(SubCommand::with_name("ssh")
            .about("Handles the PAM SSH calls by pam_exec for Watchdog"))
        .subcommand(SubCommand::with_name("auth")
            .about("Authorizes users based on from keyhouse repository. This command is passed through `AuthorizedKeysCommand` in sshd_config.")
            .arg(Arg::with_name("pubkey")
                .short("p")
                .long("pubkey")
                .help("Public key of the user trying to Authorize")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("keytype")
                .short("t")
                .long("type")
                .help("Type of Public Key/ Algorithm used")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("user")
                .short("u")
                .long("user")
                .help("Linux username requested access to. `user` in `ssh user@host`")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("config")
            .about("Get or set Watchdog configuration")
            .setting(AppSettings::ArgRequiredElseHelp)
            .arg(Arg::with_name("key")
                .index(1)
                .help("Config variable to be fetched/set"))
            .arg(Arg::with_name("value")
                .index(2)
                .help("Value to be set for the <key>. If no value is passed, the current value is returned.")))
        .subcommand(SubCommand::with_name("update"))
        .about("Update users and groups from Keyhouse")
}

fn print_traceback(e: Error) {
    println!("Traceback:");

    let mut i = 1;
    for e in e.iter().skip(1) {
        println!("[{}]: {}", i, e);
        i += 1;
    }
}

fn main() {
    let app = make_app();
    let matches = app.get_matches();

    init_logger().unwrap_or_else(|e| {
        eprintln!("Logger failed to initialize: {}", e);
    });
    info!(target: "watchdog", "Watchdog started.");
    if let Some(logs_matches) = matches.subcommand_matches("logs") {
        let level = logs_matches.value_of("level");
        match logs_matches.subcommand() {
            ("all", Some(sub_m)) => {
                handle_logs_all(sub_m.value_of("level").or(level));
            }
            ("update", Some(sub_m)) => {
                handle_logs_for(
                    LogTarget::UPDATE.as_str(),
                    sub_m.value_of("level").or(level),
                );
            }
            ("sudo", Some(sub_m)) => {
                handle_logs_for(LogTarget::SUDO.as_str(), sub_m.value_of("level").or(level));
            }
            ("su", Some(sub_m)) => {
                handle_logs_for(LogTarget::SU.as_str(), sub_m.value_of("level").or(level));
            }
            ("ssh", Some(sub_m)) => {
                handle_logs_for(LogTarget::SSH.as_str(), sub_m.value_of("level").or(level));
            }
            ("watchdog", Some(sub_m)) => {
                handle_logs_for(
                    LogTarget::WATCHDOG.as_str(),
                    sub_m.value_of("level").or(level),
                );
            }
            _ => {
                handle_logs_for("watchdog", level);
            }
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("sudo") {
        if let Err(e) = handle_sudo() {
            println!("watchdog-sudo error: {}", e);
            error!("watchdog-sudo error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("su") {
        if let Err(e) = handle_su() {
            println!("watchdog-su error: {}", e);
            error!("watchdog-su error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("ssh") {
        info!("SSH Command");
        if let Err(e) = handle_ssh() {
            println!("watchdog-ssh error: {}", e);
            error!("watchdog-ssh error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref matches) = matches.subcommand_matches("auth") {
        let pubkey = matches.value_of("pubkey").unwrap();
        let keytype = matches.value_of("keytype").unwrap();
        let user = matches.value_of("user").unwrap();
        let ssh_key = format!("{} {}", keytype, pubkey);
        if let Err(e) = handle_auth(&user, &ssh_key) {
            println!("watchdog-auth error: {}", e);
            error!("watchdog-auth error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("update") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(handle_update()) {
            println!("watchdog-update error: {}", e);
            error!("watchdog-update error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else {
        println!("No command passed");
        std::process::exit(1);
    }
}
