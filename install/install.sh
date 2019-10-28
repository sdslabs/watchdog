#!/bin/bash

# Install all the files at right place
mkdir -p /opt/watchdog/bin

cp ../target/debug/pam_ssh /opt/watchdog/bin/pam_ssh
chown root /opt/watchdog/bin/pam_ssh
chgrp root /opt/watchdog/bin/pam_ssh
chmod  700 /opt/watchdog/bin/pam_ssh

cp ../target/debug/pam_su /opt/watchdog/bin/pam_su
chown root /opt/watchdog/bin/pam_su
chgrp root /opt/watchdog/bin/pam_su
chmod  700 /opt/watchdog/bin/pam_su

cp ../target/debug/pam_sudo /opt/watchdog/bin/pam_sudo
chown root /opt/watchdog/bin/pam_sudo
chgrp root /opt/watchdog/bin/pam_sudo
chmod  700 /opt/watchdog/bin/pam_sudo

cp ../target/debug/auth_keys_cmd /opt/watchdog/bin/auth_keys_cmd
chown root /opt/watchdog/bin/auth_keys_cmd
chgrp root /opt/watchdog/bin/auth_keys_cmd
chmod  700 /opt/watchdog/bin/auth_keys_cmd

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
