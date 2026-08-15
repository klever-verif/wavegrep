import importlib.util
import pathlib
import sys
import tempfile
import unittest

import yaml

MODULE_PATH = pathlib.Path(__file__).with_name("prepare_playground.py")
SPEC = importlib.util.spec_from_file_location("prepare_playground", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
prepare_playground = importlib.util.module_from_spec(SPEC)
sys.modules["prepare_playground"] = prepare_playground
SPEC.loader.exec_module(prepare_playground)

ROOT = pathlib.Path(__file__).resolve().parents[2]


class PreparePlaygroundTests(unittest.TestCase):
    def test_stages_current_version_demo_and_wasm(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            wasm = root / "wasm"
            wasm.mkdir()
            (wasm / "wavepeek.js").write_text("export default {};", encoding="utf-8")
            (wasm / "wavepeek_bg.wasm").write_bytes(b"wasm")
            output = root / "source"
            config = root / "mkdocs.yml"
            site = root / "site"

            version = prepare_playground.prepare_tree(
                ROOT,
                wasm,
                output,
                config,
                site,
                None,
                force=False,
            )

            index = (output / "index.md").read_text(encoding="utf-8")
            self.assertIn(f"WavePeek {version}", index)
            self.assertNotIn("@WAVEPEEK_VERSION@", index)
            self.assertTrue((output / "assets/playground/scr1_axi.fst").is_file())
            self.assertEqual(
                (output / "assets/playground/wasm/wavepeek_bg.wasm").read_bytes(),
                b"wasm",
            )
            generated = yaml.safe_load(config.read_text(encoding="utf-8"))
            self.assertEqual(
                generated["nav"],
                [
                    {"Playground": "index.md"},
                    {
                        "Documentation": "https://kleverhq.github.io/wavepeek/latest/"
                    },
                ],
            )
            self.assertNotIn("extra", generated)


if __name__ == "__main__":
    unittest.main()
