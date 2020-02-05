extern crate clap;

mod auth;
mod ssh;
mod su;
mod sudo;

use crate::auth::{handle_auth, handle_auth_logs};
use crate::ssh::{handle_ssh, handle_ssh_logs};
use crate::su::{handle_su, handle_su_logs};
use crate::sudo::{handle_sudo, handle_sudo_logs};
use common_lib::errors::Error;

use clap::{App, Arg, SubCommand};

fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Watchdog")
        .version("0.1.0")
        .author("SDSLabs <contact@sdslabs.co>")
        .about("Simple server access management system on a binary")
        .arg(Arg::with_name("logs")
            .short("l")
            .long("logs")
            .help("Get the global watchdog logs"))
        .subcommand(SubCommand::with_name("sudo")
            .about("Handles the PAM sudo calls by pam_exec for Watchdog")
            .arg(Arg::with_name("logs")
                .short("l")
                .long("logs")
                .help("Get the logs for the PAM sudo calls")))
        .subcommand(SubCommand::with_name("su")
            .about("Handles the PAM su calls by pam_exec for Watchdog")
            .arg(Arg::with_name("logs")
                .short("l")
                .long("logs")
                .help("Get the logs for the PAM SSH calls")))
        .subcommand(SubCommand::with_name("ssh")
            .about("Handles the PAM SSH calls by pam_exec for Watchdog")
            .arg(Arg::with_name("logs")
                .short("l")
                .long("logs")
                .help("Get the logs for the PAM SSH calls")))
        .subcommand(SubCommand::with_name("auth")
            .about("Authorizes users based on from keyhouse repository. This command is passed through `AuthorizedKeysCommand` in sshd_config.")
            .arg(Arg::with_name("logs")
                .short("l")
                .long("logs")
                .help("Get the logs for authorized keys command."))
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
            .about("Get or set Watchdog configuration"))
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

    if let Some(ref matches) = matches.subcommand_matches("sudo") {
        if matches.is_present("logs") {
            handle_sudo_logs();
        } else {
            if let Err(e) = handle_sudo() {
                println!("watchdog-sudo error: {}", e);
                print_traceback(e);
                std::process::exit(1);
            }
        }
    } else if let Some(ref matches) = matches.subcommand_matches("su") {
        if matches.is_present("logs") {
            handle_su_logs();
        } else {
            if let Err(e) = handle_su() {
                println!("watchdog-su error: {}", e);
                print_traceback(e);
                std::process::exit(1);
            }
        }
    } else if let Some(ref matches) = matches.subcommand_matches("ssh") {
        if matches.is_present("logs") {
            handle_ssh_logs();
        } else {
            if let Err(e) = handle_ssh() {
                println!("watchdog-ssh error: {}", e);
                print_traceback(e);
                std::process::exit(1);
            }
        }
    } else if let Some(ref matches) = matches.subcommand_matches("auth") {
        if matches.is_present("logs") {
            handle_auth_logs();
        } else {
            let pubkey = matches.value_of("pubkey").unwrap();
            let keytype = matches.value_of("keytype").unwrap();
            let user = matches.value_of("user").unwrap();
            let ssh_key = format!("{} {}", keytype, pubkey);
            if let Err(e) = handle_auth(&user, &ssh_key) {
                println!("watchdog-auth error: {}", e);
                print_traceback(e);
                std::process::exit(1);
            }
        }
    } else {
        println!("No command passed");
    }
}
