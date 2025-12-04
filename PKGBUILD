pkgname=watchdog
pkgver=0.1.0
pkgrel=1
pkgdesc="Lightweight server access management system"
arch=('x86_64')
url="https://github.com/sdslabs/watchdog"
license=('MIT')
depends=('pam' 'python' 'openssh')
makedepends=('bash')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')  #TODO replace this hash

build() {
	cd "$pkgname-$pkgver"
	cargo build --release --locked
}

package() {
	cd "$pkgname-$pkgver"

	#Create directories
	install -d -m755 "$pkgdir/opt/watchdog/bin"
	install -d -m755 "$pkgdir/opt/watchdog/logs"
	install -d -m755 "$pkgdir/opt/watchdog/custom-logs"
	install -d -m755 "$pkgdir/opt/watchdog/install"
	install -d -m755 "$pkgdir/usr/bin"
	echo "Created directories"

	#Copy watchdog binary
	install -Dm755 target/release/watchdog "$pkgdir/opt/watchdog/bin/$pkgname"
	echo "Copied binary"

	#Copy sample config
	install -Dm644 sample.config.toml "$pkgdir/opt/watchdog/sample.config.toml"
	echo "Copied sample config"


	#Create empty log files
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/logs/sudo.logs"
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/logs/su.logs"
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/logs/ssh.logs"
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/custom-logs/sudo.logs"
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/custom-logs/su.logs"
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/custom-logs/ssh.logs"
	install -Dm600 /dev/null "$pkgdir/opt/watchdog/custom-logs/auth.logs"
	echo "Created log files"

	#Copy PAM and SSH modification scripts
	install -Dm755 install/edit-sshd-config.py "$pkgdir/opt/watchdog/install/edit-sshd-config.py"
	install -Dm755 install/pam-install-sudo.py "$pkgdir/opt/watchdog/install/pam-install-sudo.py"
	install -Dm755 install/pam-install-su.py "$pkgdir/opt/watchdog/install/pam-install-su.py"
	install -Dm755 install/pam-install-ssh.py "$pkgdir/opt/watchdog/install/pam-install-ssh.py"
	install -Dm755 install/install.sh "$pkgdir/opt/watchdog/install/install.sh"
	echo "Copied post-install scripts"

	#Create symlink
	ln -sf /opt/watchdog/bin/watchdog "$pkgdir/usr/bin/watchdog"
	echo "Created symlink"
}

post_install() {
	echo "Watchdog Installed"
	echo "Edit /opt/watchdog/sample.config.toml and save to /opt/watchdog/config.toml for setting config"
	echo "Run cd /opt/watchdog/install && sudo install.sh to configure PAM and SSH"
}