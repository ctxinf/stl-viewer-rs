#!/usr/bin/env python3
"""Package native Windows binaries or a macOS application bundle."""
import hashlib
import os
import plistlib
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

platform, arch = sys.argv[1:]
if platform not in ('windows', 'macos') or arch not in ('x86_64', 'aarch64'):
    raise SystemExit('Expected windows|macos and x86_64|aarch64')
version = tomllib.loads(Path('Cargo.toml').read_text())['package']['version']
# A tag must match the package version on every platform.
if os.environ.get('GITHUB_REF', '').startswith('refs/tags/'):
    assert os.environ['GITHUB_REF_NAME'] == f'v{version}', 'Tag/version mismatch'
dist = Path('dist')
dist.mkdir(exist_ok=True)
stage = Path('target/desktop-package')
if stage.exists():
    shutil.rmtree(stage)
stage.mkdir(parents=True)
name = f'stl-viewer-{version}-{platform}-{arch}'
if platform == 'windows':
    shutil.copy2('target/release/stl-viewer-linux.exe', stage / 'stl-viewer.exe')
    shutil.copy2('LICENSE', stage)
    shutil.copy2('README.md', stage)
    archive = Path(shutil.make_archive(str(dist / name), 'zip', stage))
else:
    contents = stage / 'STL Viewer.app/Contents'
    (contents / 'MacOS').mkdir(parents=True)
    (contents / 'Resources').mkdir()
    shutil.copy2('target/release/stl-viewer-linux', contents / 'MacOS/stl-viewer')
    shutil.copy2('LICENSE', contents / 'Resources/LICENSE')
    iconset = stage / 'AppIcon.iconset'
    iconset.mkdir()
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = '@2x' if scale == 2 else ''
            subprocess.run(['sips', '-z', str(size * scale), str(size * scale),
                            'packaging/stl-viewer-linux.png', '--out',
                            str(iconset / f'icon_{size}x{size}{suffix}.png')], check=True)
    subprocess.run(['iconutil', '-c', 'icns', str(iconset), '-o',
                    str(contents / 'Resources/AppIcon.icns')], check=True)
    info = dict(CFBundleName='STL Viewer', CFBundleDisplayName='STL Viewer',
                CFBundleIdentifier='io.github.ctxinf.stl-viewer',
                CFBundleExecutable='stl-viewer', CFBundlePackageType='APPL',
                CFBundleShortVersionString=version, CFBundleVersion=version,
                CFBundleIconFile='AppIcon', NSHighResolutionCapable=True)
    (contents / 'Info.plist').write_bytes(plistlib.dumps(info))
    subprocess.run(['codesign', '--force', '--deep', '--sign', '-',
                    str(contents.parent)], check=True)
    archive = dist / f'{name}.zip'
    subprocess.run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent',
                    str(contents.parent), str(archive)], check=True)
(dist / 'SHA256SUMS').write_text(f'{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n')
