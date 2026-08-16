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
        (self.skill / "references").mkdir(parents=True)
        (self.skill / "references" / "index.md").write_text(
            "# Introduction\n\nStart here.\n", encoding="utf-8"
        )
        (self.skill / "references" / "change.md").write_text(
            "# Change command\n\nFind changes.\n", encoding="utf-8"
        )
        self.write_manifest()
        self.write_navigation()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_manifest(self, **overrides: object) -> None:
        manifest = {"wavepeek_version": "0.5.0"}
        manifest.update(overrides)
        (self.skill / "manifest.json").write_text(
            json.dumps(manifest), encoding="utf-8"
        )

    def write_navigation(self, navigation: object | None = None) -> None:
        if navigation is None:
            navigation = [
                {"Start here": [{"Introduction": "index.md"}]},
                {"Commands": [{"Change": "change.md"}]},
            ]
        (self.skill / "references" / "docs.json").write_text(
            json.dumps({"navigation": navigation}), encoding="utf-8"
        )

    def prepare(self, *, force: bool = True, version: str = "0.5.0"):
        return prepare_mkdocs.prepare_tree(
            self.skill, self.output, self.config, version, force=force
        )

    def test_prepares_flat_reference_tree_and_explicit_nav(self) -> None:
        version, pages = self.prepare()

        self.assertEqual(version, "0.5.0")
        self.assertEqual(pages, ["index.md", "change.md"])
        self.assertTrue((self.output / "index.md").is_file())
        self.assertTrue((self.output / "change.md").is_file())
        self.assertTrue((self.output / "monochrome.css").is_file())
        self.assertTrue((self.output / "install-strip.js").is_file())
        self.assertEqual(
            (self.output / "wavepeek-icon.svg").read_bytes(),
            prepare_mkdocs.SITE_ICON.read_bytes(),
        )
        self.assertFalse((self.output / "docs.json").exists())
        config = yaml.safe_load(self.config.read_text(encoding="utf-8"))
        self.assertEqual(config["docs_dir"], "mkdocs-src")
        self.assertEqual(config["site_dir"], "mkdocs-site")
        self.assertEqual(
            config["nav"],
            [
                {"Playground": "/wavepeek/"},
                {
                    "Documentation": [
                        {"Start here": [{"Introduction": "index.md"}]},
                        {"Commands": [{"Change": "change.md"}]},
                    ]
                },
            ],
        )
        self.assertEqual(
            config["extra"],
            {"scope": "/wavepeek/", "version": {"provider": "mike"}},
        )

    def test_force_is_required_to_replace_outputs(self) -> None:
        self.prepare()
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "rerun with --force"):
            self.prepare(force=False)

    def test_rejects_mismatched_wavepeek_version(self) -> None:
        self.write_manifest(wavepeek_version="0.6.0")
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "does not match"):
            self.prepare()

    def test_rejects_noncanonical_alias_path(self) -> None:
        self.write_navigation(
            [
                {"Start here": [{"Introduction": "index.md"}]},
                {"Commands": [{"Change": "./change.md"}]},
            ]
        )
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "flat Markdown filename"):
            self.prepare()

    def test_navigation_must_match_flat_markdown_inventory(self) -> None:
        self.write_navigation([{"Start here": [{"Introduction": "index.md"}]}])
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "every flat Markdown file"):
            self.prepare()

        nested = self.skill / "references" / "commands"
        nested.mkdir()
        (nested / "info.md").write_text("# Info\n", encoding="utf-8")
        self.write_navigation()
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "every flat Markdown file"):
            self.prepare()


if __name__ == "__main__":
    unittest.main()
