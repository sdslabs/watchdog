#!/bin/bash

# Install all the files at right place
mkdir -p /opt/watchdog/bin
mkdir -p /opt/watchdog/logs
touch /opt/watchdog/logs/sudo.logs
touch /opt/watchdog/logs/su.logs
touch /opt/watchdog/logs/ssh.logs
mkdir -p /opt/watchdog/custom-logs
touch /opt/watchdog/custom-logs/ssh.logs
touch /opt/watchdog/custom-logs/sudo.logs
touch /opt/watchdog/custom-logs/su.logs
touch /opt/watchdog/custom-logs/auth.logs

cp ../target/release/watchdog /opt/watchdog/bin/watchdog
chown root /opt/watchdog/bin/watchdog
chgrp root /opt/watchdog/bin/watchdog
chmod  700 /opt/watchdog/bin/watchdog

cp ../config.toml /opt/watchdog/config.toml
chmod 700 /opt/watchdog/config.toml

# edit `sshd_config` file
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.watchdog.bak
python3 edit-sshd-config.py
cp watchdog_tmp_sshd_config /etc/ssh/sshd_config
rm watchdog_tmp_sshd_config
service sshd restart

# installing pam_exec lines
python3 pam-install-sudo.py
python3 pam-install-su.py
python3 pam-install-ssh.py

cp watchdog_tmp_sudo /etc/pam.d/sudo
cp watchdog_tmp_su /etc/pam.d/su
cp watchdog_tmp_ssh /etc/pam.d/sshd

rm watchdog_tmp_sudo
rm watchdog_tmp_su
rm watchdog_tmp_ssh
