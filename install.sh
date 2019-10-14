#!/bin/bash

cp target/debug/pam_ssh /opt/pam_ssh
chown root /opt/pam_ssh
chgrp root /opt/pam_ssh
chmod  700 /opt/pam_ssh

cp target/debug/auth_keys_cmd /opt/auth_keys_cmd
chown root /opt/auth_keys_cmd
chgrp root /opt/auth_keys_cmd
chmod  700 /opt/auth_keys_cmd

