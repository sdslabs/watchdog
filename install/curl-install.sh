#                   __         .__         .___             
#  __  _  _______ _/  |_  ____ |  |__    __| _/____   ____  
#  \ \/ \/ /\__  \\   __\/ ___\|  |  \  / __ |/  _ \ / ___\ 
#   \     /  / __ \|  | \  \___|   Y  \/ /_/ (  <_> ) /_/  >
#    \/\_/  (____  /__|  \___  >___|  /\____ |\____/\___  / 
#                \/          \/     \/      \/     /_____/  
#
#  Lightweight server management system, made by SDSLabs

echo "ICAgICAgICAgICAgICAgICAgIF9fICAgICAgICAgLl9fICAgICAgICAgLl9fXyAgICAgICAgICAgICAKICBfXyAgXyAgX19fX19fXyBfLyAgfF8gIF9fX18gfCAgfF9fICAgIF9ffCBfL19fX18gICBfX19fICAKICBcIFwvIFwvIC9cX18gIFxcICAgX19cLyBfX19cfCAgfCAgXCAgLyBfXyB8LyAgXyBcIC8gX19fXCAKICAgXCAgICAgLyAgLyBfXyBcfCAgfCBcICBcX19ffCAgIFkgIFwvIC9fLyAoICA8Xz4gKSAvXy8gID4KICAgIFwvXF8vICAoX19fXyAgL19ffCAgXF9fXyAgPl9fX3wgIC9cX19fXyB8XF9fX18vXF9fXyAgLyAKICAgICAgICAgICAgICAgXC8gICAgICAgICAgXC8gICAgIFwvICAgICAgXC8gICAgIC9fX19fXy8gIAoKICBMaWdodHdlaWdodCBzZXJ2ZXIgbWFuYWdlbWVudCBzeXN0ZW0sIG1hZGUgYnkgU0RTTGFicwo=" | base64 -D -i -
echo
echo "Welcome to watchdog installation wizard. This script will guide you through installation of Watchdog on this server."
echo

# checking for operating system
nameOut="$(uname -s)"

case "${unameOut}" in
    Linux*)     machine=0;;
    Darwin*)    machine=1;;
    *)          machine=2
esac

if [ machine == 2 ]; then
    echo "error: Unsupported operating system"
    exit
elif [[ machine == 0 ]]; then
    browser="xdg-open"
else
    browser="open"
fi

function hl {
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
}

hl

function setup_keyhouse {
    
    echo "We will now setup a Keyhouse repository for your organization. This is a GitHub repository where all the access management rules are written for your organization."
    echo
    echo "Setup will now open a page in browser where you create the repository using the template. The repository should be kept private. Please enter the name of your Github organization and keyhouse repository after you create one."
    read -p "Press enter to open browser."
    eval "$browser" "https://github.com/sdslabs/keyhouse-template/generate"

    echo
    read -p "Organization: " ORG
    read -p "Keyhouse Repository: " KEYHOUSE

    echo
    hl

    echo "Now you need to generate an access token for watchdog to use. Setup will open the browser again with the page where you create the personal access token. Copy the access token and keep it safe with you. That token will be used for every server you setup watchdog with."
    read -p "Press enter to open browser."
    eval "$browser" "https://github.com/settings/tokens/new?description=Keyhouse%20Token&scopes=repo"

    echo
    read -s -p "GitHub Access Token: " GITHUB_TOKEN
    echo
    hl
}

function connect_keyhouse {
    read -p "Organization: " ORG
    read -p "Keyhouse Repository: " KEYHOUSE
    read -s -p "GitHub Access Token (Leave it empty if you forgot the last one and want to generate a new one): " GITHUB_TOKEN
    echo
    echo
    hl

    if [[ "$GITHUB_TOKEN" == "" ]]; then
        echo "Now you need to generate an access token for watchdog to use. Setup will open the browser again with the page where you create the personal access token. Copy the access token and keep it safe with you. That token will be used for every server you setup watchdog with."
        read -p "Press enter to open browser."
        eval "$browser" "https://github.com/settings/tokens/new?description=Keyhouse%20Token&scopes=repo"

        echo
        read -s -p "GitHub Access Token: " GITHUB_TOKEN
        echo
        echo
        hl
    fi
}



echo "Is this the first time watchdog is being set up for the organization? (i.e have you set up the Keyhouse repository for the organization?) Select 'Installing Watchdog for the first time' if you have no idea what keyhouse is."
echo
echo "1. Installing Watchdog for the first time"
echo "2. Have already set up the Keyhouse repository"

read -p "Make a choice (1-2): " choice

echo
hl

if [[ "$choice" == "1" ]]; then
    setup_keyhouse
elif [[ "$choice" == "2" ]]; then
    connect_keyhouse
else
    echo "error: Invalid choice"
fi

read -p "What would you like to name this server? Make sure it doesn't clash with other servers where watchdog is deployed: " HOSTNAME
echo
hl

read -p "Would you like to set up Slack for server notifications? We highly recommend you to set up this feature (y/n): " choice
echo

if [[ "$choice" == "y" ]]; then

    eval "$browser \"https://slack.com/oauth/authorize?scope=incoming-webhook,chat:write:bot&client_id=805471384768.805051328564\""
    
    slack_url=$(python -c """
from BaseHTTPServer import BaseHTTPRequestHandler, HTTPServer
from urlparse import urlparse
import sys

class S(BaseHTTPRequestHandler):
    def _set_headers(self):
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.send_header('Access-Control-Allow-Origin', 'http://watchdog.sdslabs.co')
        self.end_headers()

    def do_GET(self):
        self._set_headers()
        query = urlparse(self.path).query
        query_components = dict(qc.split('=') for qc in query.split('&'))
        slack_url = query_components['token']
        print slack_url
        self.wfile.write('ok')
        return

    def log_message(self, format, *args):
        pass


def run(server_class=HTTPServer, handler_class=S, port=9876):
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    httpd.handle_request()

if __name__ == '__main__':
    from sys import argv

if len(argv) == 2:
    run(port=int(argv[1]))
else:
    run()
""")

    echo "Slack setup successfully"

fi

hl
echo "Fetching latest Watchdog Binaries..."
mkdir -p /opt/watchdog/bin

curl -o /opt/watchdog/bin/pam_su         "http://watchdog.sdslabs.co/releases/latest/pam_su"
curl -o /opt/watchdog/bin/pam_ssh        "http://watchdog.sdslabs.co/releases/latest/pam_ssh"
curl -o /opt/watchdog/bin/pam_sudo       "http://watchdog.sdslabs.co/releases/latest/pam_sudo"
curl -o /opt/watchdog/bin/auth_keys_cmd  "http://watchdog.sdslabs.co/releases/latest/auth_keys_cmd"

# Setting right user, group and permissions
chown root /opt/watchdog/bin/*
if [[ machine == 0 ]]; then
    chgrp root  /opt/watchdog/bin/*
else
    chgrp admin /opt/watchdog/bin/*
fi
chmod 700  /opt/watchdog/bin/*


tmpdir=$(mktemp -d -t watchdog-XXXX)

curl -o "$tmpdir/pam-install-su.py" "http://watchdog.sdslabs.co/releases/latest/installer/pam-install-su.py"
curl -o "$tmpdir/pam-install-ssh.py" "http://watchdog.sdslabs.co/releases/latest/installer/pam-install-ssh.py"
curl -o "$tmpdir/pam-install-sudo.py" "http://watchdog.sdslabs.co/releases/latest/installer/pam-install-sudo.py"
curl -o "$tmpdir/edit-sshd-config.py" "http://watchdog.sdslabs.co/releases/latest/installer/edit-sshd-config.py"

cd "$tmpdir"

echo "Editing SSHD Config..."
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.watchdog.bak
python3 edit-sshd-config.py
cp tmp_sshd_config /etc/ssh/sshd_config
rm tmp_sshd_config

if [[ machine == 0 ]]; then
    service sshd restart
else
    launchctl stop com.openssh.sshd
    launchctl start com.openssh.sshd
fi

echo "Setting up PAM Configuration..."
python3 pam-install-sudo.py
python3 pam-install-su.py
python3 pam-install-ssh.py

cp tmp_sudo /etc/pam.d/sudo
cp tmp_su /etc/pam.d/su
cp tmp_ssh /etc/pam.d/sshd

rm tmp_sudo
rm tmp_su
rm tmp_ssh

WATCHDOG_CONFIG="""
slack_api_url = '$slack_url'
keyhouse_base_url = 'https://api.github.com/repos/$ORG/$KEYHOUSE/contents'
keyhouse_token = '$GITHUB_TOKEN'
keyhouse_hostname = '$HOSTNAME'
watchdog_base_url = 'https://api.github.com/repos/sdslabs/watchdog/contents'
temp_env_file = '/opt/watchdog/ssh_env'
error_log_file = '/home/kanav/watchdog.log'
"""

echo "$WATCHDOG_CONFIG" > /opt/watchdog/config.toml

cd
rm -r "$tmpdir"

echo "Watchdog Installation Complete"

