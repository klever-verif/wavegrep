"""Tests for the Wavepeek output diff example."""

import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "skills" / "wavepeek_v3" / "examples" / "output_diff" / "output_diff.py"


def row(time: str, value: str) -> dict:
    return {
        "time": time,
        "sample_time": time,
        "signals": [{"path": "tb.dut.state", "value": value}],
    }


def result(rows: list[dict], complete: bool = True) -> dict:
    return {
        "type": "result",
        "command": "change",
        "data": rows,
        "summary": {"complete": complete, "returned": len(rows), "limit": None},
        "diagnostics": [],
    }


class OutputDiffTests(unittest.TestCase):
    def run_diff(self, left: pathlib.Path, right: pathlib.Path, *arguments: str):
        return subprocess.run(
            [sys.executable, "-B", SCRIPT, left, right, *arguments],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_reports_first_or_all_differing_blocks(self) -> None:
        left_rows = [row("10ns", "2'h0"), row("20ns", "2'h1"), row("30ns", "2'h2"), row("40ns", "2'h3")]
        right_rows = [row("10ns", "2'h0"), row("20ns", "2'h3"), row("30ns", "2'h2"), row("40ns", "2'h0")]

        with tempfile.TemporaryDirectory() as temporary:
            directory = pathlib.Path(temporary)
            left = directory / "before.json"
            right = directory / "after.json"
            left.write_text(json.dumps(result(left_rows)) + "\n", encoding="utf-8")
            right.write_text(json.dumps(result(right_rows)) + "\n", encoding="utf-8")

            first = self.run_diff(left, right)
            all_blocks = self.run_diff(left, right, "--all")

        self.assertEqual(first.returncode, 1)
        self.assertIn("DIFF 1 replace", first.stdout)
        self.assertNotIn("DIFF 2", first.stdout)
        self.assertIn("differing_blocks=2", first.stdout)
        self.assertIn("shown_blocks=1", first.stdout)
        self.assertEqual(all_blocks.returncode, 1)
        self.assertIn("DIFF 2 replace", all_blocks.stdout)
        self.assertIn("shown_blocks=2", all_blocks.stdout)

    def test_ignores_jsonl_sequence_numbers(self) -> None:
        data = row("10ns", "2'h1")
        left_records = [
            {"type": "begin", "seq": 0, "command": "change", "context": {}},
            {"type": "data", "seq": 1, "data": data},
            {"type": "end", "seq": 2, "summary": {"complete": True}},
        ]
        right_records = [
            {"type": "begin", "seq": 10, "command": "change", "context": {}},
            {"type": "data", "seq": 20, "data": data},
            {"type": "end", "seq": 30, "summary": {"complete": True}},
        ]

        with tempfile.TemporaryDirectory() as temporary:
            directory = pathlib.Path(temporary)
            left = directory / "before.jsonl"
            right = directory / "after.jsonl"
            left.write_text("\n".join(map(json.dumps, left_records)) + "\n", encoding="utf-8")
            right.write_text("\n".join(map(json.dumps, right_records)) + "\n", encoding="utf-8")
            comparison = self.run_diff(left, right)

        self.assertEqual(comparison.returncode, 0)
        self.assertEqual(comparison.stdout, "EQUAL rows=1\n")
        self.assertEqual(comparison.stderr, "")

    def test_rejects_incomplete_results(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = pathlib.Path(temporary)
            left = directory / "before.json"
            right = directory / "after.json"
            left.write_text(json.dumps(result([row("10ns", "1'h0")], False)) + "\n", encoding="utf-8")
            right.write_text(json.dumps(result([row("10ns", "1'h0")])) + "\n", encoding="utf-8")
            comparison = self.run_diff(left, right)

        self.assertEqual(comparison.returncode, 1)
        self.assertIn("result is incomplete", comparison.stderr)
        self.assertEqual(comparison.stdout, "")


if __name__ == "__main__":
    unittest.main()
