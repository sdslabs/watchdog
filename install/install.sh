#!/bin/bash

# Install all the files at right place
mkdir -p /opt/watchdog/bin

cp ../target/debug/watchdog /opt/watchdog/bin/watchdog
chown root /opt/watchdog/bin/watchdog
chgrp root /opt/watchdog/bin/watchdog
chmod  700 /opt/watchdog/bin/watchdog

cp ../config.toml /opt/watchdog/config.toml

# edit `sshd_config` file
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.watchdog.bak
python3 edit-sshd-config.py
cp tmp_sshd_config /etc/ssh/sshd_config
rm tmp_sshd_config
service sshd restart

# installing pam_exec lines
python3 pam-install-sudo.py
python3 pam-install-su.py
python3 pam-install-ssh.py

cp tmp_sudo /etc/pam.d/sudo
cp tmp_su /etc/pam.d/su
cp tmp_ssh /etc/pam.d/sshd

rm tmp_sudo
rm tmp_su
rm tmp_ssh
