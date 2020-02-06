watchdog_config = """
# SDSLabs Watchdog configuration START

session optional pam_exec.so seteuid log=/opt/watchdog/logs/sudo.logs /opt/watchdog/bin/watchdog sudo

# SDSLabs Watchdog configuration END
"""

inside_watchdog_config_section = False

def process_line(line):
	global inside_watchdog_config_section

	if inside_watchdog_config_section and line == "# SDSLabs Watchdog configuration END\n":
		inside_watchdog_config_section = False
		return ''

	if inside_watchdog_config_section:
		return ''

	if line == "# SDSLabs Watchdog configuration START\n":
		inside_watchdog_config_section = True
		return ''

	return line

def main():
	iput = open("/etc/pam.d/sudo")
	oput = open("watchdog_tmp_sudo", "w")
	lines = iput.readlines()
	for l in lines:
		oputline = process_line(l)
		oput.write(oputline)

	oput.write(watchdog_config)

	iput.close()
	oput.close()


main()