
watchdog_config = """
# SDSLabs Watchdog configuration START

session optional pam_exec.so seteuid log=/opt/watchdog/logs/sudo.logs /opt/watchdog/bin/watchdog sudo

# SDSLabs Watchdog configuration END
"""

inside_watchdog_config = False

def process_line(line):
	global inside_watchdog_config

	if inside_watchdog_config and line == "# SDSLabs Watchdog configuration END\n":
		inside_watchdog_config = False
		return ''

	if inside_watchdog_config:
		return ''

	if line == "# SDSLabs Watchdog configuration START\n":
		inside_watchdog_config = True
		return ''

	return line

def main():
	iput = open("/etc/pam.d/sudo")
	oput = open("tmp_sudo", "w")
	lines = iput.readlines()
	for l in lines:
		oputline = process_line(l)
		oput.write(oputline)

	oput.write(watchdog_config)

	iput.close()
	oput.close()


main()