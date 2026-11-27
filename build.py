#!/usr/bin/env python3

import contextlib
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path

is_windows = platform.system() == "Windows"

root_path = Path(__file__).resolve().parent
dist_path = root_path.joinpath("dist")
staging_root_path = root_path.joinpath("staging")

app_name = "stool"
bin_name = "stool"


def get_version() -> str:
    with open(root_path.joinpath("version.json"), "r", encoding="utf-8") as f:
        vobj = json.load(f)

    return vobj["FullVersion"]


version = get_version()


@contextlib.contextmanager
def cd(path: Path | str):
    old_cwd = Path.cwd()

    os.chdir(path)

    try:
        yield
    finally:
        os.chdir(old_cwd)


def rmdir(path: Path | str):
    path = Path(path)

    with contextlib.suppress(FileNotFoundError):
        shutil.rmtree(path)


def rmfile(path: Path | str):
    path = Path(path)

    path.unlink(missing_ok=True)


def cmd(
    description: str, args: Sequence[str | Path], env: dict[str, str] | None = None
):
    print(f"--- {description} ---")

    if env is None:
        cmd_env = os.environ
    else:
        cmd_env = os.environ.copy()
        cmd_env.update(env)

    subprocess.run(args, env=cmd_env, check=True)


def create_hash_file(path: Path | str) -> Path:
    path = Path(path)
    hash_path = path.with_name(f"{path.name}.sha256.txt")

    with open(path, "rb") as f:
        checksum = hashlib.file_digest(f, "sha256")

    with open(hash_path, "w", encoding="utf-8") as f:
        f.write(checksum.hexdigest())

    return hash_path


def create_tarball(archive_name: str, path: Path | str):
    path = Path(path)

    archive_file = f"{archive_name}.tar.zst"
    archive_path = dist_path.joinpath(archive_file)

    # Delete old archive
    rmfile(archive_path)

    with cd(path):
        cmd("Compressing archive", ["tar", "acf", archive_path, "."])

    create_hash_file(archive_path)


def create_zip(archive_name: str, path: Path | str):
    path = Path(path)

    archive_file = f"{archive_name}.zip"
    archive_path = dist_path.joinpath(dist_path, archive_file)

    # Delete old archive
    rmfile(archive_path)

    with cd(path):
        cmd("Compressing archive", ["7z", "a", "-mx9", archive_path, "*"])

    create_hash_file(archive_path)


def build(name: str, target: str, cross: bool = False):
    staging_path = staging_root_path.joinpath(name)

    # Delte old staging directory
    rmdir(staging_path)

    # Create staging directory
    staging_path.mkdir(parents=True, exist_ok=True)

    if cross:
        cargo_bin = "cross"
    else:
        cargo_bin = "cargo"

    # Build
    print("--- BUILDING ---")
    cmd("Cargo build", [cargo_bin, "build", "--release", "--target", target])

    return staging_path


def bin_path(target: str, bin_name: str) -> Path:
    return root_path.joinpath("target", target, "release", bin_name)


def copy_bin(staging_path: Path | str, target: str, bin_name: str):
    staging_path = Path(staging_path)

    print("--- COPYING BIN ---")
    shutil.copy2(bin_path(target, bin_name), staging_path)


def build_linux(name: str, target: str):
    staging_path = build(name, target, cross=True)

    # Copy files
    copy_bin(staging_path, target, bin_name)

    # Create tarball
    create_tarball(f"{app_name}-{version}-{name}", staging_path)


def build_windows(name: str, target: str):
    staging_path = build(name, target)

    # Copy files
    copy_bin(staging_path, target, f"{bin_name}.exe")

    # Create archive
    create_zip(f"{app_name}-{version}-{name}", staging_path)


def build_linux64():
    build_linux("linux-x86_64", "x86_64-unknown-linux-gnu")


def build_linux32():
    build_linux("linux-i686", "i686-unknown-linux-gnu")


def build_appimage(name: str, target: str):
    staging_path = build(name, target, cross=True)

    appimage_path = dist_path.joinpath(f"{name}.AppImage")

    desktop_path = root_path.joinpath(f"{bin_name}.desktop")
    icon_path = root_path.joinpath(f"{app_name}.svg")

    cmd(
        "Build AppImage",
        [
            "linuxdeploy-x86_64.AppImage",
            "--appdir",
            staging_path,
            "--executable",
            bin_path(target, bin_name),
            "--desktop-file",
            desktop_path,
        ]
        + (["--icon-file", icon_path] if icon_path.is_file() else [])
        + [
            "--output",
            "appimage",
        ],
        env={"OUTPUT": str(appimage_path)},
    )

    create_hash_file(appimage_path)


def build_appimage64():
    build_appimage(app_name, "x86_64-unknown-linux-gnu")


def build_win64():
    if is_windows:
        target = "x86_64-pc-windows-msvc"
    else:
        target = "x86_64-pc-windows-gnu"

    build_windows("windows-x86_64", target)


def build_win32():
    if is_windows:
        target = "i686-pc-windows-msvc"
    else:
        target = "i686-pc-windows-gnu"

    build_windows("windows-i686", target)


COMMON_REQUIRED_BINS = ["cargo"]
LINUX_REQUIRED_BINS = ["cross", "tar", "zstd"]
WINDOWS_REQUIRED_BINS = ["7z"]

CONFIGURATIONS = {
    "linux64": {
        "build": build_linux64,
        "required_bins": COMMON_REQUIRED_BINS + LINUX_REQUIRED_BINS,
    },
    "linux32": {
        "build": build_linux32,
        "required_bins": COMMON_REQUIRED_BINS + LINUX_REQUIRED_BINS,
    },
    "appimage": {
        "build": build_appimage64,
        "required_bins": COMMON_REQUIRED_BINS
        + LINUX_REQUIRED_BINS
        + ["linuxdeploy-x86_64.AppImage"],
    },
    "win64": {
        "build": build_win64,
        "required_bins": COMMON_REQUIRED_BINS + WINDOWS_REQUIRED_BINS,
    },
    "win32": {
        "build": build_win32,
        "required_bins": COMMON_REQUIRED_BINS + WINDOWS_REQUIRED_BINS,
    },
}


def main() -> int:
    missing_bins = set()
    to_build = []

    def add_to_build(cfg):
        def build():
            cfg["build"]()

        to_build.append(build)

    for cfg_name in sys.argv[1:]:
        cfg = CONFIGURATIONS.get(cfg_name)
        if cfg is None:
            print(f"Configuration '{cfg_name}' does not exist.")
            continue

        for bin in cfg["required_bins"]:
            if not shutil.which(bin):
                missing_bins.add(bin)

        add_to_build(cfg)

    if missing_bins:
        print(f"Missing required binaries:\n\n{'\n'.join(missing_bins)}")
        return 2

    if to_build:
        dist_path.mkdir(parents=True, exist_ok=True)

        for build in to_build:
            build()

    return 0


if __name__ == "__main__":
    sys.exit(main())
