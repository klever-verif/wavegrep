#!/usr/bin/env python3

"""Stage the current root Playground for a Material for MkDocs build."""

import argparse
import hashlib
import os
import pathlib
import shutil
import sys
import tempfile
import tomllib

import yaml


DEMO_SHA256 = "aad73e9b0d2b244b67a96b254371ff29a2ac2e54077176376f6361570789e884"
DOCUMENTATION_URL = "/wavepeek/latest/"


class PrepareError(Exception):
    pass


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_root", type=pathlib.Path)
    parser.add_argument("--wasm-dir", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--config-output", required=True, type=pathlib.Path)
    parser.add_argument("--site-output", required=True, type=pathlib.Path)
    parser.add_argument("--version")
    parser.add_argument("--force", action="store_true")
    return parser.parse_args(argv)


def fail(message: str) -> None:
    raise PrepareError(message)


def relative(path: pathlib.Path, parent: pathlib.Path) -> str:
    return pathlib.Path(os.path.relpath(path.resolve(), parent.resolve())).as_posix()


def source_version(source_root: pathlib.Path, requested: str | None) -> str:
    cargo = tomllib.loads((source_root / "Cargo.toml").read_text(encoding="utf-8"))
    version = cargo["package"]["version"]
    if requested is not None and requested != version:
        fail(f"requested version {requested} does not match Cargo.toml version {version}")
    return version


def prepare_tree(
    source_root: pathlib.Path,
    wasm_dir: pathlib.Path,
    output: pathlib.Path,
    config_output: pathlib.Path,
    site_output: pathlib.Path,
    version: str | None,
    *,
    force: bool,
) -> str:
    source_root = source_root.resolve()
    wasm_dir = wasm_dir.resolve()
    output = output.resolve()
    config_output = config_output.resolve()
    site_output = site_output.resolve()
    playground = source_root / "web" / "playground"
    demo = playground / "assets" / "scr1_axi.fst"

    if not playground.is_dir():
        fail(f"playground source does not exist: {playground}")
    if not (wasm_dir / "wavepeek.js").is_file() or not (wasm_dir / "wavepeek_bg.wasm").is_file():
        fail(f"wasm-bindgen output is incomplete: {wasm_dir}")
    if hashlib.sha256(demo.read_bytes()).hexdigest() != DEMO_SHA256:
        fail(f"bundled demo digest does not match {DEMO_SHA256}: {demo}")
    if output.exists() and not force:
        fail(f"output directory already exists: {output}; rerun with --force")
    if config_output.exists() and not force:
        fail(f"config output already exists: {config_output}; rerun with --force")

    wavepeek_version = source_version(source_root, version)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=f".{output.name}-", dir=output.parent) as temporary:
        temporary_output = pathlib.Path(temporary)
        index = temporary_output / "index.md"
        rendered = (playground / "index.md").read_text(encoding="utf-8").replace(
            "@WAVEPEEK_VERSION@", wavepeek_version
        )
        index.write_text(rendered, encoding="utf-8")
        playground_assets = temporary_output / "assets" / "playground"
        shutil.copytree(playground / "assets", playground_assets)
        shutil.copytree(
            wasm_dir,
            playground_assets / "wasm",
            dirs_exist_ok=True,
        )
        shutil.copyfile(
            source_root / "tools" / "docs" / "monochrome.css",
            temporary_output / "monochrome.css",
        )
        shutil.copyfile(
            source_root / "tools" / "docs" / "install-strip.js",
            temporary_output / "install-strip.js",
        )
        if output.exists():
            shutil.rmtree(output)
        temporary_output.replace(output)

    config_parent = config_output.parent
    generated = {
        "INHERIT": relative(source_root / "mkdocs.yml", config_parent),
        "docs_dir": relative(output, config_parent),
        "site_dir": relative(site_output, config_parent),
        "nav": [
            {"Playground": "index.md"},
            {"Documentation": DOCUMENTATION_URL},
        ],
        "extra_css": [
            "monochrome.css",
            "assets/playground/playground.css",
        ],
    }
    config_output.parent.mkdir(parents=True, exist_ok=True)
    config_output.write_text(
        yaml.safe_dump(generated, sort_keys=False, allow_unicode=True), encoding="utf-8"
    )
    return wavepeek_version


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        version = prepare_tree(
            args.source_root,
            args.wasm_dir,
            args.output,
            args.config_output,
            args.site_output,
            args.version,
            force=args.force,
        )
    except (OSError, KeyError, PrepareError, tomllib.TOMLDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"prepared WavePeek {version} Playground at {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
