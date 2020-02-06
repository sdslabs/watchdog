# Watchdog

> Lightweight server access management system

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/sdslabs/watchdog/blob/master/LICENSE.md)

Watchdog is a lightweight server access management system which does not itself need to be hosted on a server.

## Contents

* [Overview](#overview)
* [Features](#features)
* [Dependencies](#dependencies)
* [Installation](#installation)
* [Usage](#usage)
* [Development](#development)
* [Contact](#contact)

## Overview


## Features

* Request SSH access to a server just by creating a PR to the Keyhouse repository.
* Stateless and serverless. Watchdog runs on a single binary (can't make any promises on keeping this number the same though)
* Optional server activity logs to your favourite workspace like Slack or Discord.
* Easy Installation and Configuration
* Get notified when someone escalates privilidges or performs administrative tasks using `sudo` or `su`

## Dependencies

The following softwares are required for running Watchdog:-

* PAM
* OpenSSH server

## Installation

1. Create a Keyhouse Repository using the template repository [here](https://github.com/sdslabs/keyhouse-template).

2. Clone the the watchdog repository

 `git clone https://github.com/sdslabs/watchdog.git`

3. Change into the repository directory and build the latest binaries using Cargo

    `cargo build --release`

4. Copy `sample.config.toml` to `config.toml` and make changes to the config this way:

	1. `slack_api_url` : Make an incoming hook to your Slack workspace from [this app](https://slack.com/apps/A0F7XDUAZ-incoming-webhooks) and paste the hook URL here. You can customize the icon and name as you like.

	2. `keyhouse_base_url`: It should be of the format `https://api.github.com/repos/<ORGANIZATION>/<KEYHOUSE-REPOSITORY>/contents`.

	3. `keyhouse_token`: This should be a personal access token made by a member of organization on his/her behalf who can read the Keyhouse repository. Click [here](https://github.com/settings/tokens/new?description=Keyhouse%20Token&scopes=repo) to make a new token with correct scopes.

	4. `keyhouse_hostname`: Name you want to give to this server. Note that this should be same as the file you create in the `hosts` directory in keyhouse.

5. Once you are done configuring, run this command with root (sudo) priviledges

    `cd install && ./install.sh`

6. Add `/opt/watchdog/bin` to your PATH variable.

## Usage

```
$ watchdog --help
```

```
Watchdog 0.1.0
SDSLabs <contact@sdslabs.co>
Simple server access management system on a binary

USAGE:
    watchdog [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -l, --logs       Get the global watchdog logs
    -V, --version    Prints version information

SUBCOMMANDS:
    auth      Authorizes users based on from keyhouse repository. This command is passed through
              `AuthorizedKeysCommand` in sshd_config.
    config    Get or set Watchdog configuration
    help      Prints this message or the help of the given subcommand(s)
    ssh       Handles the PAM SSH calls by pam_exec for Watchdog
    su        Handles the PAM su calls by pam_exec for Watchdog
    sudo      Handles the PAM sudo calls by pam_exec for Watchdog
```

Though most of the commands are for internal use of PAM, you can edit configuration of Watchdog any time
```sh
$ watchdog config --help
```

```
```

To view logs
```sh
$ watchdog logs --help
```

## Development

You need to have [Rust](https://www.rust-lang.org/tools/install) installed along with the mentioned [dependencies](#dependencies)

Open your favourite terminal and perform the following tasks:-

1. Clone this repository.

    ```bash
    $ git clone https://github.com/sdslabs/watchdog
    ```

2. Make the required changes inside the source code directory ([src/](src/))

3. Run `cargo test` to test your changes.

4. Rebuild the binary using `cargo build` command.

## Contact

If you have a query regarding the product or just want to say hello then feel free to visit
[chat.sdslabs.co](http://chat.sdslabs.co/) or drop a mail at [contact@sdslabs.co.in](mailto:contact@sdslabs.co.in)