
watchdog_config = """
# SDSLabs Watchdog configuration START

UsePAM yes
PasswordAuthentication no
AuthorizedKeysCommand /opt/watchdog/bin/auth_keys_cmd %u %h %t %f %k
AuthorizedKeysCommandUser root

# SDSLabs Watchdog configuration END
"""


modified_keys = [
	'AuthorizedKeysCommand',
	'AuthorizedKeysCommandUser',
	'PasswordAuthentication',
	'UsePAM'
]

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

	l = line.strip()
	i = l.find('#')
	if i != -1:
		l = l[:i]
	if len(l) == 0:
		return line
	i = l.find(' ')
	j = l.find('\t')
	if i == -1 and j != -1:
		i = j
	elif j == -1 and i != -1:
		pass
	elif j == -1 and i == -1:
		return line
	else:
		i = min(i, j)
	key = l[:i]
	value = l[i+1:].strip()
	if key in modified_keys:
		# comment this line
		return '# Watchdog: Commenting the line below out\n#' + line
	else:
		return line

def main():
	iput = open("/etc/ssh/sshd_config")
	oput = open("tmp_sshd_config", "w")
	lines = iput.readlines()
	for l in lines:
		oputline = process_line(l)
		oput.write(oputline)

	oput.write(watchdog_config)

	iput.close()
	oput.close()


main()