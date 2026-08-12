from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest

import yaml

MODULE_PATH = pathlib.Path(__file__).with_name("prepare_mkdocs.py")
SPEC = importlib.util.spec_from_file_location("prepare_mkdocs", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
prepare_mkdocs = importlib.util.module_from_spec(SPEC)
sys.modules["prepare_mkdocs"] = prepare_mkdocs
SPEC.loader.exec_module(prepare_mkdocs)


class PrepareMkdocsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp.name)
        self.skill = self.root / "skill"
        self.output = self.root / "mkdocs-src"
        self.config = self.root / "mkdocs.yml"
        (self.skill / "references" / "commands").mkdir(parents=True)
        (self.skill / "examples").mkdir()
        (self.skill / "SKILL.md").write_text("# Skill\n", encoding="utf-8")
        (self.skill / "references" / "intro.md").write_text(
            "# Introduction\n\nStart here.\n", encoding="utf-8"
        )
        (self.skill / "references" / "commands" / "change.md").write_text(
            "# Change command\n\nFind changes.\n", encoding="utf-8"
        )
        self.write_manifest()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_manifest(self, **overrides: object) -> None:
        manifest = {"wavepeek_version": "0.5.0", "bundle_format_version": 1}
        manifest.update(overrides)
        (self.skill / "manifest.json").write_text(
            json.dumps(manifest), encoding="utf-8"
        )

    def prepare(self, *, force: bool = True, version: str = "0.5.0"):
        return prepare_mkdocs.prepare_tree(
            self.skill, self.output, self.config, version, force=force
        )

    def test_prepares_reference_tree_and_nav(self) -> None:
        version, topics = self.prepare()

        self.assertEqual(version, "0.5.0")
        self.assertEqual(len(topics), 2)
        self.assertEqual(
            (self.output / "index.md").read_text(encoding="utf-8"),
            "# Introduction\n\nStart here.\n",
        )
        self.assertTrue((self.output / "commands" / "change.md").is_file())
        self.assertFalse((self.output / "manifest.json").exists())
        config = yaml.safe_load(self.config.read_text(encoding="utf-8"))
        self.assertEqual(config["docs_dir"], "mkdocs-src")
        self.assertEqual(config["site_dir"], "mkdocs-site")
        self.assertIn({"Introduction": "index.md"}, config["nav"])
        self.assertIn(
            {"Commands": [{"Change command": "commands/change.md"}]},
            config["nav"],
        )

    def test_force_is_required_to_replace_outputs(self) -> None:
        self.prepare()
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "rerun with --force"):
            self.prepare(force=False)

    def test_rejects_unsupported_bundle_version(self) -> None:
        self.write_manifest(bundle_format_version=999)
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "bundle_format_version"):
            self.prepare()

    def test_rejects_mismatched_wavepeek_version(self) -> None:
        self.write_manifest(wavepeek_version="0.6.0")
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "does not match"):
            self.prepare()

    def test_rejects_missing_required_bundle_paths(self) -> None:
        (self.skill / "examples").rmdir()
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "missing examples"):
            self.prepare()

    def test_rejects_reference_without_h1(self) -> None:
        (self.skill / "references" / "intro.md").write_text(
            "No title.\n", encoding="utf-8"
        )
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "missing an H1"):
            self.prepare()


if __name__ == "__main__":
    unittest.main()
