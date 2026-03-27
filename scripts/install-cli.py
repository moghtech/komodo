import argparse
import sys
import os
import platform
import json
import urllib.request

def parse_args():
	p = argparse.ArgumentParser(
		prog="setup-km",
		description="Install the Komodo CLI",
		formatter_class=argparse.ArgumentDefaultsHelpFormatter,
	)

	p.add_argument(
		"--version", "-v",
		default=json.load(urllib.request.urlopen("https://api.github.com/repos/moghtech/komodo/releases/latest"))["tag_name"],
		help="Install a specific version, like 'v2.0.0'"
	)

	p.add_argument(
		"--user", "-u",
		action="store_true",
		help="Install systemd '--user' service"
	)

	p.add_argument(
		"--binary-url",
		default="https://github.com/moghtech/komodo/releases/download",
		help="Use alternate binary source"
	)

	return p.parse_args()

def load_bin_dir(args):
	home_dir = os.environ['HOME']
	if args.user:
		return f'{home_dir}/.local/bin'
	else:
		return "/usr/local/bin"

def download_binary(args, bin_dir):
	# ensure bin_dir exists
	if not os.path.isdir(bin_dir):
		os.makedirs(bin_dir)

	# delete binary if it already exists
	bin_path = f'{bin_dir}/km'
	if os.path.isfile(bin_path):
		os.remove(bin_path)

	km_bin = "km-x86_64"
	arch = platform.machine().lower()
	if arch == "aarch64" or arch == "arm64":
		print("aarch64 detected")
		km_bin = "km-aarch64"
	else:
		print("using x86_64 binary")

	# download the binary to bin path
	if os.system(f'curl -f -sSL {args.binary_url}/{args.version}/{km_bin} -o {bin_path}') != 0:
		raise RuntimeError(
			f"Failed to download binary from "
			f"{args.binary_url}/{args.version}/{km_bin}"
			f"\n\nDid you provide a valid tag for '--version'? Check here for valid version tags:"
			f"\nhttps://github.com/moghtech/komodo/tags"
		)

	# add executable permissions
	os.popen(f'chmod +x {bin_path}')
	
def main():
	args = parse_args()

	print("======================")
	print(" KOMODO CLI INSTALLER ")
	print("======================")

	bin_dir = load_bin_dir(args)
 
	print(f'version: {args.version}')
	print(f'user install: {args.user}')
	print(f'bin dir: {bin_dir}')

	download_binary(args, bin_dir)

	print("Finished komodo-cli setup. Try running 'km --help'.\n")

main()
