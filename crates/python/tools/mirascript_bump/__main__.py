import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


VERSION_RE = re.compile(
    r"(__version__\s*=\s*)['\"]([^'\"]+)['\"]"
)


def read_version(init_file: Path) -> str:
    text = init_file.read_text(encoding="utf-8")
    m = VERSION_RE.search(text)
    if not m:
        raise RuntimeError(f"Cannot find __version__ in {init_file}")
    return m.group(2)


def bump(version: str, kind: str) -> str:
    parts = version.split(".")
    try:
        major, minor, patch = (int(p) for p in parts[:3])
    except Exception:
        raise RuntimeError(f"Non-semantic version: {version}")

    if kind == "major":
        return f"{major+1}.0.0"
    if kind == "minor":
        return f"{major}.{minor+1}.0"
    if kind == "patch":
        return f"{major}.{minor}.{patch+1}"

    raise ValueError(kind)


def write_version(init_file: Path, new_version: str) -> None:
    text = init_file.read_text(encoding="utf-8")
    new_text = VERSION_RE.sub(rf"\1'{new_version}'", text)
    init_file.write_text(new_text, encoding="utf-8")


def git(*args):
    subprocess.run(["git", *args], check=True)


def main(argv=None):
    p = argparse.ArgumentParser("mirascript-bump")
    p.add_argument("kind", choices=["major", "minor", "patch"])
    p.add_argument("--package", default="mirascript",
                   help="package directory containing __init__.py")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--no-git", action="store_true")

    args = p.parse_args(argv)

    init_file = Path(args.package) / "__init__.py"
    if not init_file.exists():
        sys.exit(f"❌ {init_file} not found")

    old = read_version(init_file)
    new = bump(old, args.kind)

    print(f"🔖 {old} → {new}")

    if args.dry_run:
        return

    write_version(init_file, new)

    if not args.no_git:
        try:
            git("add", str(init_file))
            git("commit", "-m", f"Bump version to {new}", "--no-verify")
            git("tag", f"python-{new}")
        except Exception as e:
            print("⚠️ git operations failed:", e)


if __name__ == "__main__":
    main()
