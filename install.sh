#!/bin/bash

cp target/debug/pam_ssh /opt/watchdog/pam_ssh
chown root /opt/watchdog/pam_ssh
chgrp root /opt/watchdog/pam_ssh
chmod  700 /opt/watchdog/pam_ssh

cp target/debug/pam_su /opt/watchdog/pam_su
chown root /opt/watchdog/pam_su
chgrp root /opt/watchdog/pam_su
chmod  700 /opt/watchdog/pam_su

cp target/debug/auth_keys_cmd /opt/watchdog/auth_keys_cmd
chown root /opt/watchdog/auth_keys_cmd
chgrp root /opt/watchdog/auth_keys_cmd
chmod  700 /opt/watchdog/auth_keys_cmd

