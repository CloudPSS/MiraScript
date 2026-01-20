from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = "MiraScript 的 Python 绑定允许在 Python 中使用 MiraScript 编译器。它提供了一个简单的接口来编译 MiraScript 脚本，并返回编译结果和诊断信息。"
import os
import re
import subprocess

# Read current version from package without importing it
pkg_init = os.path.join(os.path.dirname(__file__), 'mirascript', '__init__.py')
with open(pkg_init, 'r', encoding='utf-8') as f:
    init_src = f.read()

m = re.search(r"__version__\s*=\s*['\"]([^'\"]+)['\"]", init_src)
if not m:
    raise RuntimeError('Cannot find __version__ in mirascript/__init__.py')

current_version = m.group(1)

# Bump strategy: use env BUMP_TYPE (major/minor/patch), default patch
bump_type = os.environ.get('BUMP_TYPE', 'patch')
parts = current_version.split('.')
try:
    parts = [int(p) for p in parts]
except Exception:
    # fallback: if not semantic, don't change
    parts = None

if parts is not None:
    major, minor, patch = (parts + [0, 0, 0])[:3]
    if bump_type == 'major':
        major += 1
        minor = 0
        patch = 0
    elif bump_type == 'minor':
        minor += 1
        patch = 0
    else:
        patch += 1

    new_version = f"{major}.{minor}.{patch}"
    # Update file only if version changed
    if new_version != current_version:
        new_src = re.sub(r"(__version__\s*=\s*)['\"][^'\"]+['\"]",
                         rf"\1'{new_version}'",
                         init_src)
        with open(pkg_init, 'w', encoding='utf-8') as f:
            f.write(new_src)

        # commit and tag
        try:
            subprocess.run(['git', 'add', pkg_init], check=True)
            subprocess.run(['git', 'commit', '-m', f'Bump version to {new_version}','--no-verify'], check=True)
            subprocess.run(['git', 'tag', f'python_{new_version}'], check=True)
        except Exception:
            # If git commands fail, continue but surface a warning
            print('Warning: git operations failed; version file updated but not tagged')
else:
    new_version = current_version

__version__ = new_version

setup(
    name='cloudpss-mirascript',
    version=__version__,
    keywords=["cloudpss", "cloudpss-mirascript", "mirascript"],
    description='cloudpss mirascript',
    long_description=long_description,
    long_description_content_type="text/markdown",
    license="MIT Licence",
    url='https://www.cloudpss.net',
    author='cloudpss',
    author_email='zhangdaming@cloudpss.net',
    packages=find_packages( include=("mirascript*",)),
    include_package_data=True,
    platforms="any",
    python_requires='>=3.7',
    install_requires=[],
)
