# Watchdog

> Lightweight server management system

<img align="right" width="350px" height="400px" src="./docs/content/assets/logo/watchdoglogo.svg">

[![Build Status](https://travis-ci.org/sdslabs/watchdog.svg?branch=master)](https://travis-ci.org/sdslabs/watchdog)
[![Docs](https://img.shields.io/badge/docs-current-brightgreen.svg)](https://watchdog-docs.netlify.com/)
[![codecov](https://codecov.io/gh/sdslabs/watchdog/branch/master/graph/badge.svg)](https://codecov.io/gh/sdslabs/watchdog)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/sdslabs/watchdog/blob/master/LICENSE.md)

Watchdog is a lightweight server access management system which does not itself need to be hosted on a server.

## Contents

* [Overview](#overview)
* [Features](#features)
* [Supported Languages](#supported-languages)
* [Supported Databases](#supported-databases)
* [Documentation](#documentation)
* [Dependencies](#dependencies)
* [Download](#download)
* [Development](#development)
* [Contributing](#contributing)
* [Contact](#contact)

## Overview


## Features

* Request SSH access to a server just by creating a PR to the Keyhouse repository.
* Stateless and serverless. Watchdog runs on only *4* binaries (can't make any promises on keeping this number the same though)
* Optional server activity logs to your favourite workspace like Slack or Discord.
* Easy Installation and Configuration
* Get notified when someone escalates privilidges or performs administrative tasks using `sudo` or `su`

## Documentation

You can find the complete documentation of Watchdog at [https://watchdog-docs.netlify.com/](https://watchdog-docs.netlify.com/)

## Dependencies

The following softwares are required for running Watchdog:-

* PAM
* OpenSSH server

## Download

Assuming you have the [dependencies](#dependencies) installed, head over to Watchdog's [releases](https://github.com/sdslabs/watchdog/releases) page and grab the latest binary according to your operating system and system architecture

## Development

You need to have [Rust](https://golang.org/dl/) installed along with the mentioned [dependencies](#dependencies)

Open your favourite terminal and perform the following tasks:-

1. Clone this repository.

    ```bash
    $ git clone https://github.com/sdslabs/watchdog
    ```

## Contributing

If you'd like to contribute to this project, refer to the [contributing documentation](./CONTRIBUTING.md).

Created with 💖 by [SDSLabs](https://github.com/sdslabs)

## Contact

If you have a query regarding the product or just want to say hello then feel free to visit
[chat.sdslabs.co](http://chat.sdslabs.co/) or drop a mail at [contact@sdslabs.co.in](mailto:contact@sdslabs.co.in)