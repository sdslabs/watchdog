extern crate clap;

mod sudo;
mod su;
mod ssh;
mod auth;

use crate::sudo::{handle_sudo, handle_sudo_logs};
use crate::su::{handle_su, handle_su_logs};
use crate::ssh::{handle_ssh, handle_ssh_logs};
use crate::auth::{handle_auth, handle_auth_logs};

use clap::{Arg, App,SubCommand};

fn make_app<'a,'b>() -> App<'a,'b> {
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
                .help("Get the logs for authorized keys command.")))
        .subcommand(SubCommand::with_name("config")
            .about("Get or set Watchdog configuration"))
}

fn main() {
    let app = make_app();
    let matches = app.get_matches();

    if let Some(ref matches) = matches.subcommand_matches("sudo") {
        if matches.is_present("logs") {
            handle_sudo_logs();
        } else {
            handle_sudo();
        }
    }
    else if let Some(ref matches) = matches.subcommand_matches("su") {
        if matches.is_present("logs") {
            handle_su_logs();
        } else {
            handle_su();
        }
    }
    else if let Some(ref matches) = matches.subcommand_matches("ssh") {
        if matches.is_present("logs") {
            handle_ssh_logs();
        } else {
            handle_ssh();
        }
    }
    else if let Some(ref matches) = matches.subcommand_matches("auth") {
        if matches.is_present("logs") {
            handle_auth_logs();
        } else {
            handle_auth();
        }
    }
    else {
        println!("No command passed");
    }

}