#!/usr/bin/env bash
set -euo pipefail

# Keep this script self-contained so downloading it does not execute a second script.
command -v python3 >/dev/null || { echo 'Python 3 is required. Install Python 3, then rerun this command.' >&2; exit 1; }
command -v curl >/dev/null || { echo 'curl is required.' >&2; exit 1; }
python3 - "$@" <<'PY'
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

REPOSITORY = 'GriffinBoris/OpenTranscribe'


def download(url, destination):
    subprocess.run([
        'curl', '--fail', '--location', '--show-error', '--silent',
        '--proto', '=https', '--proto-redir', '=https', '--retry', '3',
        '--output', str(destination), url,
    ], check=True)


def main():
    parser = argparse.ArgumentParser(description='Install an official OpenTranscribe release on macOS or Linux.')
    parser.add_argument('--version', help='Release tag, such as v0.1.14; defaults to the latest stable release')
    parser.add_argument('--directory', type=Path, help='Installation directory; defaults to /Applications or ~/Applications')
    parser.add_argument('--dry-run', action='store_true', help='Resolve and display the release without installing')
    options = parser.parse_args()
    if sys.version_info < (3, 9):
        parser.error('Python 3.9 or newer is required.')
    system = platform.system()
    architecture = platform.machine()
    if system == 'Darwin' and architecture == 'x86_64':
        translated = subprocess.run(['sysctl', '-n', 'sysctl.proc_translated'], capture_output=True, text=True)
        if translated.stdout.strip() == '1':
            architecture = 'arm64'
    if (system, architecture) not in {('Darwin', 'arm64'), ('Darwin', 'x86_64'), ('Linux', 'x86_64')}:
        parser.error('Supported: macOS Apple silicon/Intel and Linux x64. Use the Windows installer on Windows.')
    if options.version and not re.fullmatch(r'v\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?', options.version):
        parser.error('--version must be a release tag such as v0.1.14')
    directory = (options.directory or (Path('/Applications') if system == 'Darwin' else Path.home() / 'Applications'))
    directory = directory.expanduser().resolve()
    destination = directory / ('OpenTranscribe.app' if system == 'Darwin' else 'OpenTranscribe.AppImage')
    release_path = f'tags/{options.version}' if options.version else 'latest'
    with tempfile.TemporaryDirectory(prefix='opentranscribe-download-') as temporary:
        temporary = Path(temporary)
        metadata = temporary / 'release.json'
        download(f'https://api.github.com/repos/{REPOSITORY}/releases/{release_path}', metadata)
        release = json.loads(metadata.read_text())
        version = release['tag_name'].removeprefix('v')
        suffix = 'aarch64.dmg' if architecture == 'arm64' else 'x64.dmg'
        if system == 'Linux':
            suffix = 'amd64.AppImage'
        name = f'OpenTranscribe_{version}_{suffix}'
        assets = [asset for asset in release['assets'] if asset['name'] == name]
        if len(assets) != 1:
            parser.error(f'Release does not contain exactly one {name}')
        asset = assets[0]
        digest = asset.get('digest')
        if not digest or not re.fullmatch(r'sha256:[0-9a-f]{64}', digest):
            parser.error('Release asset has no SHA-256 digest; use a newer release.')
        expected_url = f'https://github.com/{REPOSITORY}/releases/download/{release["tag_name"]}/{name}'
        if asset['browser_download_url'] != expected_url:
            parser.error('Unexpected release download URL')
        print(f'Install {release["tag_name"]}: {name}\nDestination: {destination}', flush=True)
        if options.dry_run:
            return
        if subprocess.run(['pgrep', '-x', 'opentranscribe'], stdout=subprocess.DEVNULL).returncode == 0:
            parser.error('Quit OpenTranscribe completely before installing. Closing its window only hides it.')
        directory.mkdir(parents=True, exist_ok=True)
        if not os.access(directory, os.W_OK):
            parser.error('Installation directory is not writable. Use --directory "$HOME/Applications".')
        if destination.is_symlink():
            parser.error('Destination is a symbolic link; choose another directory.')
        payload = temporary / name
        download(expected_url, payload)
        checksum = hashlib.sha256()
        with payload.open('rb') as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b''):
                checksum.update(chunk)
        if checksum.hexdigest() != digest.removeprefix('sha256:'):
            parser.error('Downloaded asset failed SHA-256 verification; installation canceled.')
        # Stage on the destination filesystem so the final rename never copies a partial install.
        with tempfile.TemporaryDirectory(prefix='.opentranscribe-install-', dir=directory) as staging:
            staging = Path(staging)
            replacement = staging / destination.name
            if system == 'Darwin':
                mount = temporary / 'volume'
                mount.mkdir()
                subprocess.run(['hdiutil', 'attach', '-readonly', '-nobrowse', '-mountpoint', str(mount), str(payload)], check=True)
                try:
                    subprocess.run(['ditto', str(mount / 'OpenTranscribe.app'), str(replacement)], check=True)
                finally:
                    subprocess.run(['hdiutil', 'detach', str(mount)], check=True)
            else:
                shutil.copyfile(payload, replacement)
                replacement.chmod(0o755)
            backup = directory / f'.{destination.name}.previous'
            if backup.exists():
                parser.error(f'A previous installation backup exists at {backup}; recover or move it before retrying.')
            if destination.exists():
                destination.rename(backup)
            try:
                replacement.rename(destination)
            except OSError:
                if backup.exists():
                    backup.rename(destination)
                raise
            if backup.is_dir():
                shutil.rmtree(backup)
            elif backup.exists():
                backup.unlink()
        print(f'Installed {destination}. Open it to finish setup.')
        print('In-app updates remain enabled. Recordings, settings, models, and permissions were not reset.')
        if system == 'Darwin':
            print('For unsigned-build launch or permission recovery, see:')
            print(f'https://github.com/{REPOSITORY}#macos-reset-permissions-after-reinstalling-an-unsigned-build')


if __name__ == '__main__':
    main()
PY
