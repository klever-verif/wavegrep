"""Tests for the APB scoreboard example shipped with the Wavepeek skill."""

import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "skills" / "wavepeek" / "examples" / "apb_scoreboard" / "apb_scoreboard.py"


def row(event: str, time: str, direction: str, payload: dict[str, str]) -> dict:
    return {
        "time": time,
        "sample_time": time,
        "profile": "apb4",
        "event": event,
        "direction": direction,
        "payload": payload,
    }


class ApbScoreboardTests(unittest.TestCase):
    def run_scoreboard(self, input_text: str, *arguments: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, "-B", SCRIPT, *arguments],
            input=input_text,
            check=True,
            capture_output=True,
            text=True,
        )

    def test_combines_setup_wait_and_completion_events(self) -> None:
        rows = [
            row("setup", "5ns", "write", {"paddr": "8'h40", "pprot": "3'h2", "pwdata": "8'hde", "pstrb": "1'h1"}),
            row("access-wait", "10ns", "write", {"paddr": "8'h40"}),
            row("access-wait", "15ns", "write", {"paddr": "8'h40"}),
            row("access-complete", "20ns", "write", {"pslverr": "1'h0"}),
            row("setup", "25ns", "read", {"paddr": "8'h44", "pprot": "3'h1"}),
            row("access-complete", "30ns", "read", {"prdata": "8'ha5", "pslverr": "1'h1"}),
        ]
        stream = "\n".join(
            json.dumps({"type": "data", "seq": index, "data": item})
            for index, item in enumerate(rows, 1)
        )

        result = self.run_scoreboard(stream)

        self.assertIn(
            "WRITE start=5ns end=20ns addr=8'h40 pprot=3'h2 wait_cycles=2 "
            "error=1'h0 data=8'hde strb=1'h1",
            result.stdout,
        )
        self.assertIn(
            "READ start=25ns end=30ns addr=8'h44 pprot=3'h1 wait_cycles=0 "
            "error=1'h1 data=8'ha5",
            result.stdout,
        )
        self.assertIn(
            "SUMMARY completed_reads=1 completed_writes=1 completed_unknown=0 "
            "incomplete=0 unmatched_transfers=0",
            result.stdout,
        )
        self.assertEqual(result.stderr, "")

    def test_preserves_unknown_direction(self) -> None:
        rows = [
            row("setup", "40ns", "unknown", {"paddr": "8'h4c", "pwdata": "8'hcc"}),
            row("access-complete", "45ns", "unknown", {"prdata": "8'ha5"}),
        ]
        stream = "\n".join(
            json.dumps({"type": "data", "seq": index, "data": item})
            for index, item in enumerate(rows, 1)
        )

        result = self.run_scoreboard(stream)

        self.assertIn(
            "UNKNOWN start=40ns end=45ns addr=8'h4c pprot=- wait_cycles=0 "
            "error=- write_data=8'hcc read_data=8'ha5",
            result.stdout,
        )
        self.assertIn("completed_unknown=1", result.stdout)

    def test_fatal_input_exits_nonzero(self) -> None:
        record = json.dumps({"type": "fatal", "message": "mapping failed"})

        with self.assertRaises(subprocess.CalledProcessError) as failure:
            self.run_scoreboard(record)

        self.assertIn("wavepeek: mapping failed", failure.exception.stderr)

    def test_reads_json_file_and_reports_unmatched_and_incomplete_events(self) -> None:
        envelope = {
            "type": "result",
            "command": "extract apb",
            "data": [
                row("access-complete", "20ns", "read", {"prdata": "8'h00"}),
                row("setup", "25ns", "write", {"paddr": "8'h48"}),
            ],
            "diagnostics": [],
        }
        with tempfile.TemporaryDirectory() as temporary:
            input_file = pathlib.Path(temporary) / "apb.json"
            input_file.write_text(json.dumps(envelope) + "\n", encoding="utf-8")
            result = self.run_scoreboard("", str(input_file))

        self.assertIn("WARNING unmatched Access completion at 20ns", result.stderr)
        self.assertIn("WARNING incomplete WRITE start=25ns addr=8'h48", result.stderr)
        self.assertIn(
            "SUMMARY completed_reads=0 completed_writes=0 completed_unknown=0 "
            "incomplete=1 unmatched_transfers=1",
            result.stdout,
        )


if __name__ == "__main__":
    unittest.main()
