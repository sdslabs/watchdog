#!/bin/bash

cp target/debug/auth_keys_cmd /opt/auth_keys_cmd
chown root /opt/auth_keys_cmd
chgrp root /opt/auth_keys_cmd
chmod  700 /opt/auth_keys_cmd
