#!/bin/bash

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
