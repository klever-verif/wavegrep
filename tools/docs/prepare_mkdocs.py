#!/usr/bin/env python3

import argparse
import json
import os
import pathlib
import shutil
import sys
import tempfile
from typing import Any

import yaml


PLAYGROUND_URL = "/wavepeek/"


class PrepareError(Exception):
    pass


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prepare a generated MkDocs source tree from an extracted wavepeek skill."
    )
    parser.add_argument("skill_dir", type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--config-output", required=True, type=pathlib.Path)
    parser.add_argument("--version")
    parser.add_argument("--force", action="store_true")
    return parser.parse_args(argv)


def fail(message: str) -> None:
    raise PrepareError(message)


def load_json(path: pathlib.Path, description: str) -> Any:
    if not path.is_file():
        fail(f"missing {description}: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"{description} is not valid JSON: {error}")


def load_manifest(skill_dir: pathlib.Path, version: str | None) -> str:
    manifest = load_json(skill_dir / "manifest.json", "skill manifest")
    if not isinstance(manifest, dict):
        fail("skill manifest root must be an object")
    wavepeek_version = manifest.get("wavepeek_version")
    if not isinstance(wavepeek_version, str) or not wavepeek_version:
        fail("skill manifest wavepeek_version must be a non-empty string")
    if version is not None and wavepeek_version != version:
        fail(
            f"skill manifest wavepeek_version {wavepeek_version!r} does not match {version!r}"
        )
    return wavepeek_version


def load_navigation(references: pathlib.Path) -> tuple[list[dict[str, Any]], list[str]]:
    manifest = load_json(references / "docs.json", "reference navigation")
    if not isinstance(manifest, dict) or not isinstance(manifest.get("navigation"), list):
        fail("reference navigation must contain a navigation array")

    nav = manifest["navigation"]
    pages: list[str] = []
    try:
        for group in nav:
            if not isinstance(group, dict) or len(group) != 1:
                raise ValueError
            items = next(iter(group.values()))
            if not isinstance(items, list) or not items:
                raise ValueError
            for item in items:
                if not isinstance(item, dict) or len(item) != 1:
                    raise ValueError
                path = next(iter(item.values()))
                if not isinstance(path, str):
                    raise ValueError
                relative = pathlib.PurePosixPath(path)
                if (
                    path != relative.as_posix()
                    or relative.is_absolute()
                    or len(relative.parts) != 1
                    or relative.suffix != ".md"
                ):
                    fail(f"reference navigation path must be a flat Markdown filename: {path}")
                pages.append(path)
    except ValueError:
        fail("reference navigation must use MkDocs group and page objects")

    discovered = sorted(path.relative_to(references).as_posix() for path in references.rglob("*.md"))
    if sorted(pages) != discovered:
        fail("reference navigation must list every flat Markdown file exactly once")
    return nav, pages


def write_generated_config(
    config_output: pathlib.Path,
    output: pathlib.Path,
    nav: list[dict[str, Any]],
) -> None:
    config_parent = config_output.parent.resolve()
    generated = {
        "INHERIT": pathlib.Path(
            os.path.relpath(pathlib.Path("mkdocs.yml").resolve(), config_parent)
        ).as_posix(),
        "docs_dir": pathlib.Path(os.path.relpath(output.resolve(), config_parent)).as_posix(),
        "site_dir": "mkdocs-site",
        "nav": [
            {"Playground": PLAYGROUND_URL},
            {"Documentation": nav},
        ],
        "extra": {"version": {"provider": "mike"}},
    }
    config_output.parent.mkdir(parents=True, exist_ok=True)
    config_output.write_text(
        yaml.safe_dump(generated, sort_keys=False, allow_unicode=True), encoding="utf-8"
    )


def prepare_tree(
    skill_dir: pathlib.Path,
    output: pathlib.Path,
    config_output: pathlib.Path,
    version: str | None,
    *,
    force: bool,
) -> tuple[str, list[str]]:
    skill_dir = skill_dir.resolve()
    output = output.resolve()
    config_output = config_output.resolve()
    if not skill_dir.is_dir():
        fail(f"skill directory does not exist: {skill_dir}")
    if output.exists() and not force:
        fail(f"output directory already exists: {output}; rerun with --force")
    if config_output.exists() and not force:
        fail(f"config output already exists: {config_output}; rerun with --force")

    wavepeek_version = load_manifest(skill_dir, version)
    references = skill_dir / "references"
    nav, pages = load_navigation(references)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=f".{output.name}-", dir=output.parent) as temporary:
        temp_dir = pathlib.Path(temporary)
        for page in pages:
            shutil.copyfile(references / page, temp_dir / page)
        shutil.copyfile(pathlib.Path(__file__).with_name("monochrome.css"), temp_dir / "monochrome.css")
        if output.exists():
            shutil.rmtree(output)
        temp_dir.replace(output)
    write_generated_config(config_output, output, nav)
    return wavepeek_version, pages


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        version, pages = prepare_tree(
            args.skill_dir,
            args.output,
            args.config_output,
            args.version,
            force=args.force,
        )
    except PrepareError as error:
        print(f"error: docs-site: {error}", file=sys.stderr)
        return 1
    print(f"prepared {len(pages)} reference(s) for wavepeek {version} at {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
