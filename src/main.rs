extern crate clap;

mod auth;
mod ssh;
mod su;
mod sudo;

use std::process::Command;

use clap::{App, Arg, SubCommand};

use lib::errors::Error;

use auth::handle_auth;
use ssh::{handle_ssh, handle_ssh_logs};
use su::{handle_su, handle_su_logs};
use sudo::{handle_sudo, handle_sudo_logs};

fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Watchdog")
        .version("0.1.0")
        .author("SDSLabs <contact@sdslabs.co>")
        .about("Simple server access management system on a binary")
        .subcommand(SubCommand::with_name("logs")
            .about("Get the global watchdog logs")
            .arg(Arg::with_name("filter")
                .short("f")
                .long("filter")
                .help("Filter logs according to service. Can take value among `su`, `sudo`, `ssh` or `all`")
                .takes_value(true)
                .default_value("all")))
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

    if let Some(ref _matches) = matches.subcommand_matches("sudo") {
        if let Err(e) = handle_sudo() {
            println!("watchdog-sudo error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("su") {
        if let Err(e) = handle_su() {
            println!("watchdog-su error: {}", e);
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref _matches) = matches.subcommand_matches("ssh") {
        if let Err(e) = handle_ssh() {
            println!("watchdog-ssh error: {}", e);
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
            print_traceback(e);
            std::process::exit(1);
        }
    } else if let Some(ref matches) = matches.subcommand_matches("logs") {
        let filter = matches.value_of("filter").unwrap();
        if filter == "all" {
            handle_all_logs();
        } else if filter == "sudo" {
            handle_sudo_logs();
        } else if filter == "su" {
            handle_su_logs();
        } else if filter == "ssh" {
            handle_ssh_logs();
        } else {
            println!("Invalid Filter");
        }
    } else {
        println!("No command passed");
    }
}

fn handle_all_logs() {
    /* TODO: Unimplemented function */
    Command::new("less")
        .arg("/opt/watchdog/logs/sudo.logs")
        .status()
        .expect("Something went wrong. Is `less` command present in your environment?");
}
