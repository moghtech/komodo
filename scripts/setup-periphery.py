import argparse
import sys
import os
import shutil
import platform
import json
import urllib.request

def parse_args():
	p = argparse.ArgumentParser(
		prog="setup-periphery",
		description="Install systemd-managed Komodo Periphery",
		formatter_class=argparse.ArgumentDefaultsHelpFormatter,
	)

	p.add_argument(
		"--version", "-v",
		default=json.load(urllib.request.urlopen("https://api.github.com/repos/moghtech/komodo/releases/latest"))["tag_name"],
		help="Install a specific Komodo version, like 'v2.0.0'"
	)

	p.add_argument(
		"--user", "-u",
		action="store_true",
		help="Install systemd '--user' service"
	)

	p.add_argument(
		"--root-directory", "-r",
		default="/etc/komodo",
		help="Specify a specific Periphery root directory."
	)

	p.add_argument(
		"--core-address", "-c",
		help="Specify the Komodo Core address for outbound connection. Leave blank to enable inbound connection server."
	)

	p.add_argument(
		"--connect-as", "-n",
		default=os.uname().nodename,
		help="Specify the Server name to connect as. Defaults to hostname."
	)

	p.add_argument(
		"--onboarding-key", "-k",
		help="Give an onboarding key for automatic Server onboarding into Komodo Core."
	)

	p.add_argument(
		"--force-service-file",
		help="Recreate the systemd service file even if it already exists."
	)

	p.add_argument(
		"--config-url",
		default="https://raw.githubusercontent.com/moghtech/komodo/refs/heads/main/config/periphery.config.toml",
		help="Use a custom config url."
	)

	p.add_argument(
		"--binary-url",
		default="https://github.com/moghtech/komodo/releases/download",
		help="Use alternate binary source"
	)

	return p.parse_args()

def load_paths(args):
	home_dir = os.environ['HOME']
	if args.user:
		return [
			# home_dir
			home_dir,
			# binary location
			f'{home_dir}/.local/bin',
			# config location
	 		f'{home_dir}/.config/komodo',
			# service file location
	 		f'{home_dir}/.config/systemd/user',
		]
	else:
		return [
			# home_dir
			home_dir,
			# binary location
			"/usr/local/bin",
			# config location
	 		"/etc/komodo",
			# service file location
	 		"/etc/systemd/system",
		]

def download_binary(args, bin_dir):
	# stop periphery in case its currently in use
	user = ""
	if args.user:
		user = " --user"
	os.popen(f'systemctl{user} stop periphery')

	# ensure bin_dir exists
	if not os.path.isdir(bin_dir):
		os.makedirs(bin_dir)

	# delete binary if it already exists
	bin_path = f'{bin_dir}/periphery'
	if os.path.isfile(bin_path):
		os.remove(bin_path)

	periphery_bin = "periphery-x86_64"
	arch = platform.machine().lower()
	if arch == "aarch64" or arch == "arm64":
		print("aarch64 detected")
		periphery_bin = "periphery-aarch64"
	else:
		print("using x86_64 binary")

	# download the binary to bin path
	print(os.popen(f'curl -sSL {args.binary_url}/{args.version}/{periphery_bin} > {bin_path}').read())

	# add executable permissions
	os.popen(f'chmod +x {bin_path}')

def map_config_line(args, home_dir, line):
	## Handle root directory
	if line.startswith("root_directory ="):
		if args.root_directory != None:
			return f'root_directory = "{args.root_directory}"'
		if args.user:
			return f'root_directory = "{home_dir}/komodo"'
	## Handle core_address
	if line.startswith("# core_address =") and args.core_address != None:
		return f'core_address = "{args.core_address}"'
	## Handle connect_as
	if line.startswith("# connect_as ="):
		return f'connect_as = "{args.connect_as}"'
	## Handle onboarding key
	if line.startswith("# onboarding_key =") and args.onboarding_key != None:
		return f'onboarding_key = "{args.onboarding_key}"'
	return line

def write_config(args, home_dir, config_dir):
	config_file = f'{config_dir}/periphery.config.toml'

	# early return if config file already exists
	if os.path.isfile(config_file):
		print(f'Config at {config_file} already exists, skipping...')
		return

	print(f'creating config at {config_file}')

	# ensure config dir exists
	if not os.path.isdir(config_dir):
		os.makedirs(config_dir)

	template = urllib.request.urlopen(args.config_url).read().decode("utf-8").split("\n")
	lines = [map_config_line(args, home_dir, line) for line in template]
	config = "\n".join(lines)

	with open(config_file, "w", encoding="utf-8", newline="\n") as f:
		f.write(config)

def write_service_file(args, home_dir, bin_dir, config_dir, service_dir):
	service_file = f'{service_dir}/periphery.service'

	if args.force_service_file:
		print("forcing service file recreation")

	# early return is service file already exists
	if os.path.isfile(service_file):
		if args.force_service_file:
			print("deleting existing service file")
			os.remove(service_file)
		else:
			print(f'service file already exists at {service_file}, skipping...')
			return
	
	print(f'creating service file at {service_file}')
	
	# ensure service_dir exists
	if not os.path.isdir(service_dir):
		os.makedirs(service_dir)

	f = open(service_file, "x")
	f.write((
		"[Unit]\n"
		"Description=Agent to connect with Komodo Core\n"
		"\n"
		"[Service]\n"
		f'Environment="HOME={home_dir}"\n'
		f'ExecStart=/bin/sh -lc "{bin_dir}/periphery --config-path {config_dir}/periphery.config.toml"\n'
		"Restart=on-failure\n"
		"TimeoutStartSec=0\n"
		"\n"
		"[Install]\n"
		"WantedBy=default.target"
	))

	user = ""
	if args.user:
		user = " --user"
	os.popen(f'systemctl{user} daemon-reload')

def uses_systemd():
	# First check if systemctl is an available command, then check if systemd is the init system
	return shutil.which("systemctl") is not None and os.path.exists("/run/systemd/system/")

def main():
	args = parse_args()

	print("=====================")
	print(" PERIPHERY INSTALLER ")
	print("=====================")

	if not uses_systemd():
		print("This installer requires systemd and systemd wasn't found. Exiting")
		sys.exit(1)

	[home_dir, bin_dir, config_dir, service_dir] = load_paths(args)
	
	print(f'version: {args.version}')
	print(f'core address: {args.core_address}')
	print(f'connect as: {args.connect_as}')
	print(f'user install: {args.user}')
	print(f'home dir: {home_dir}')
	print(f'bin dir: {bin_dir}')
	print(f'config dir: {config_dir}')
	print(f'service file dir: {service_dir}')

	download_binary(args, bin_dir)
	write_config(args, home_dir, config_dir)
	write_service_file(args, home_dir, bin_dir, config_dir, service_dir)

	user = ""
	if args.user:
		user = " --user"

	print("Starting Periphery...")
	print(os.popen(f'systemctl{user} start periphery').read())

	print("Finished Periphery setup.\n")
	print(f'Note. Use "systemctl{user} status periphery" to make sure Periphery is running')
	print(f'Note. Use "systemctl{user} enable periphery" to have Periphery start on system boot')

main()
