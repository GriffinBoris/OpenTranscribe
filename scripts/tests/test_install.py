import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

INSTALLER = Path(__file__).resolve().parents[1] / 'install.sh'
SOURCE = INSTALLER.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0]
NAMESPACE = {'__name__': 'installer_tests'}
exec(compile(SOURCE, str(INSTALLER), 'exec'), NAMESPACE)


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.payload = b'AppImage fixture'
        self.asset_names = []

    def download(self, url, destination):
        if 'api.github.com' not in url:
            self.asset_names.append(url.rsplit('/', 1)[1])
            destination.write_bytes(self.payload)
            return
        assets = []
        for suffix in ['aarch64.dmg', 'x64.dmg', 'amd64.AppImage']:
            name = f'OpenTranscribe_0.1.14_{suffix}'
            assets.append({
                'name': name,
                'digest': f'sha256:{hashlib.sha256(self.payload).hexdigest()}',
                'browser_download_url': f'https://github.com/GriffinBoris/OpenTranscribe/releases/download/v0.1.14/{name}',
            })
        destination.write_text(json.dumps({'tag_name': 'v0.1.14', 'assets': assets}))

    def run_installer(self, system='Linux', architecture='x86_64', dry_run=False, download=None):
        arguments = ['install', '--directory', str(self.directory)]
        if dry_run:
            arguments.append('--dry-run')
        with patch.object(sys, 'argv', arguments), \
                patch('platform.system', return_value=system), \
                patch('platform.machine', return_value=architecture), \
                patch('subprocess.run', return_value=subprocess.CompletedProcess([], 1, stdout='')), \
                patch.dict(NAMESPACE, {'download': download or self.download}):
            NAMESPACE['main']()

    def test_linux_replacement_is_executable_and_keeps_unrelated_files(self):
        destination = self.directory / 'OpenTranscribe.AppImage'
        destination.write_bytes(b'previous app')
        unrelated = self.directory / 'notes.txt'
        unrelated.write_text('keep')
        self.run_installer()
        self.assertEqual(destination.read_bytes(), self.payload)
        self.assertEqual(destination.stat().st_mode & 0o777, 0o755)
        self.assertEqual(unrelated.read_text(), 'keep')
        self.assertEqual(self.asset_names, ['OpenTranscribe_0.1.14_amd64.AppImage'])
        self.assertFalse((self.directory / '.OpenTranscribe.AppImage.previous').exists())

    def test_bad_checksum_preserves_existing_installation(self):
        destination = self.directory / 'OpenTranscribe.AppImage'
        destination.write_bytes(b'previous app')

        def corrupt_download(url, path):
            self.download(url, path)
            if 'api.github.com' not in url:
                path.write_bytes(b'corrupt')

        with self.assertRaises(SystemExit):
            self.run_installer(download=corrupt_download)
        self.assertEqual(destination.read_bytes(), b'previous app')

    def test_dry_run_resolves_macos_architectures_without_installing(self):
        for architecture in ['arm64', 'x86_64']:
            with self.subTest(architecture=architecture):
                self.run_installer('Darwin', architecture, dry_run=True)
        self.assertEqual(self.asset_names, [])
        self.assertEqual(list(self.directory.iterdir()), [])

    def test_unsupported_architecture_does_not_download(self):
        with self.assertRaises(SystemExit):
            self.run_installer('Linux', 'aarch64')
        self.assertEqual(self.asset_names, [])

    def test_failed_replacement_restores_previous_installation(self):
        destination = self.directory / 'OpenTranscribe.AppImage'
        destination.write_bytes(b'previous app')
        rename = Path.rename

        def fail_replacement(path, target):
            if path.parent.name.startswith('.opentranscribe-install-'):
                raise OSError('replacement failed')
            return rename(path, target)

        with patch.object(Path, 'rename', fail_replacement), self.assertRaises(OSError):
            self.run_installer()
        self.assertEqual(destination.read_bytes(), b'previous app')


if __name__ == '__main__':
    unittest.main()
