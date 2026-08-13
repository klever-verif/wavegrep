#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import pathlib
import shutil
import sys
import tempfile
from dataclasses import dataclass
from typing import Any

import yaml

SECTION_LABELS = {
    "commands": "Commands",
    "workflows": "Workflows",
    "troubleshooting": "Troubleshooting",
    "reference": "Reference",
}
SECTION_ORDER = tuple(SECTION_LABELS)


class PrepareError(Exception):
    pass


@dataclass(frozen=True)
class Topic:
    title: str
    source: pathlib.Path
    page: pathlib.PurePosixPath


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


def load_manifest(skill_dir: pathlib.Path, version: str | None) -> str:
    manifest_path = skill_dir / "manifest.json"
    if not manifest_path.is_file():
        fail(f"missing skill manifest: {manifest_path}")
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"skill manifest is not valid JSON: {error}")
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


def markdown_title(path: pathlib.Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    fail(f"reference is missing an H1 title: {path}")


def load_topics(skill_dir: pathlib.Path) -> list[Topic]:
    for required in ("SKILL.md", "references", "examples"):
        if not (skill_dir / required).exists():
            fail(f"skill bundle is missing {required}: {skill_dir / required}")
    references = skill_dir / "references"
    if not references.is_dir():
        fail(f"skill references path is not a directory: {references}")

    topics: list[Topic] = []
    for source in sorted(references.rglob("*.md")):
        relative = source.relative_to(references)
        page = pathlib.PurePosixPath(*relative.parts)
        if page == pathlib.PurePosixPath("intro.md"):
            page = pathlib.PurePosixPath("index.md")
        topics.append(Topic(markdown_title(source), source, page))
    if not any(topic.page == pathlib.PurePosixPath("index.md") for topic in topics):
        fail(f"skill references are missing intro.md: {references}")
    return topics


def build_nav(topics: list[Topic]) -> list[dict[str, Any]]:
    intro = next(topic for topic in topics if topic.page == pathlib.PurePosixPath("index.md"))
    nav: list[dict[str, Any]] = [{intro.title: "index.md"}]
    by_section: dict[str, list[Topic]] = {}
    for topic in topics:
        if topic is intro:
            continue
        section = topic.page.parts[0] if len(topic.page.parts) > 1 else "other"
        by_section.setdefault(section, []).append(topic)
    for section in SECTION_ORDER:
        section_topics = by_section.pop(section, [])
        if section_topics:
            nav.append(
                {
                    SECTION_LABELS[section]: [
                        {topic.title: topic.page.as_posix()} for topic in section_topics
                    ]
                }
            )
    for section in sorted(by_section):
        nav.append(
            {
                section.replace("-", " ").replace("_", " ").title(): [
                    {topic.title: topic.page.as_posix()}
                    for topic in by_section[section]
                ]
            }
        )
    return nav


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
        "nav": nav,
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
) -> tuple[str, list[Topic]]:
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
    topics = load_topics(skill_dir)
    output.parent.mkdir(parents=True, exist_ok=True)
    temp_dir = pathlib.Path(tempfile.mkdtemp(prefix=f".{output.name}-", dir=output.parent))
    try:
        for topic in topics:
            destination = temp_dir / pathlib.Path(*topic.page.parts)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(topic.source, destination)
        if output.exists():
            shutil.rmtree(output)
        temp_dir.replace(output)
        write_generated_config(config_output, output, build_nav(topics))
    except Exception:
        if temp_dir.exists():
            shutil.rmtree(temp_dir)
        raise
    return wavepeek_version, topics


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        version, topics = prepare_tree(
            args.skill_dir,
            args.output,
            args.config_output,
            args.version,
            force=args.force,
        )
    except PrepareError as error:
        print(f"error: docs-site: {error}", file=sys.stderr)
        return 1
    print(f"prepared {len(topics)} reference(s) for wavepeek {version} at {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
