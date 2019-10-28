
watchdog_config = """
# SDSLabs Watchdog configuration START

session optional pam_exec.so seteuid /opt/watchdog/bin/pam_su

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
	iput = open("/etc/pam.d/su")
	oput = open("tmp_su", "w")
	lines = iput.readlines()
	for l in lines:
		oputline = process_line(l)
		oput.write(oputline)

	oput.write(watchdog_config)

	iput.close()
	oput.close()


main()