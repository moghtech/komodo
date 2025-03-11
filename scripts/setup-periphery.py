import sys
import os
import shutil
import platform
import json
import urllib.request

def load_version():
	version = ""
	for arg in sys.argv:
		if arg.count("--version") > 0:
			version = arg.split("=")[1]
	if len(version) == 0:
		version = load_latest_version()
	return version

def load_latest_version():
	return json.load(urllib.request.urlopen("https://api.github.com/repos/mbecker20/komodo/releases/latest"))["tag_name"]

def uses_systemd():
	# First check if systemctl is an available command, then check if systemd is the init system
	return shutil.which("systemctl") is not None and os.path.exists("/run/systemd/system/")

def load_paths():
	home_dir = os.environ['HOME']
	# Checks if setup.py is passed --user arg
	user_install = sys.argv.count("--user") > 0
	if user_install:
		return [
			True,
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
			False,
			# home_dir
			home_dir,
			# binary location
			"/usr/local/bin",
			# config location
	 		"/etc/komodo",
			# service file location
	 		"/etc/systemd/system",
		]

def copy_binary(user_install, bin_dir, version):
	# stop periphery in case its currently in use
	user = ""
	if user_install:
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
	print(os.popen(f'curl -sSL https://github.com/mbecker20/komodo/releases/download/{version}/{periphery_bin} > {bin_path}').read())

	# add executable permissions
	os.popen(f'chmod +x {bin_path}')

def copy_config(config_dir):
	config_file = f'{config_dir}/periphery.config.toml'

	# early return if config file already exists
	if os.path.isfile(config_file):
		print("config already exists, skipping...")
		return
	
	print(f'creating config at {config_file}')

	# ensure config dir exists
	if not os.path.isdir(config_dir):
		os.makedirs(config_dir)

	print(os.popen(f'curl -sSL https://raw.githubusercontent.com/mbecker20/komodo/main/config/periphery.config.toml > {config_dir}/periphery.config.toml').read())

def copy_service_file(home_dir, bin_dir, config_dir, service_dir, user_install):
	service_file = f'{service_dir}/periphery.service'

	force_service_recopy = sys.argv.count("--force-service-file") > 0

	if force_service_recopy:
		print("forcing service file recreation")

	# early return is service file already exists
	if os.path.isfile(service_file):
		if force_service_recopy:
			print("deleting existing service file")
			os.remove(service_file)
		else:
			print("service file already exists, skipping...")
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
	if user_install:
		user = " --user"
	os.popen(f'systemctl{user} daemon-reload')
	
def main():
	print("=====================")
	print(" PERIPHERY INSTALLER ")
	print("=====================")

	if not uses_systemd():
		print("This installer requires systemd and systemd wasn't found. Exiting")
		sys.exit(1)

	version = load_version()
	[user_install, home_dir, bin_dir, config_dir, service_dir] = load_paths()
 
	print(f'version: {version}')
	print(f'user install: {user_install}')
	print(f'home dir: {home_dir}')
	print(f'bin dir: {bin_dir}')
	print(f'config dir: {config_dir}')
	print(f'service file dir: {service_dir}')

	force_service_recopy = sys.argv.count("--force-service-file") > 0
	if force_service_recopy:
		print('forcing service file rewrite')

	copy_binary(user_install, bin_dir, version)
	copy_config(config_dir)
	copy_service_file(home_dir, bin_dir, config_dir, service_dir, user_install)

	user = ""
	if user_install:
		user = " --user"

	print("starting periphery...")
	print(os.popen(f'systemctl{user} start periphery').read())

	print("Finished periphery setup.\n")
	print(f'Note. Use "systemctl{user} status periphery" to make sure periphery is running')
	print(f'Note. Use "systemctl{user} enable periphery" to have periphery start on system boot')

main()
