"""Tests for the AXI scoreboard example shipped with the Wavepeek skill."""

import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = (
    ROOT
    / "skills"
    / "wavepeek"
    / "examples"
    / "axi_scoreboard"
    / "axi_scoreboard.py"
)


def row(channel: str, time: str, payload: dict[str, str]) -> dict:
    return {
        "time": time,
        "sample_time": time,
        "profile": "axi4",
        "channel": channel,
        "payload": payload,
    }


class AxiScoreboardTests(unittest.TestCase):
    def run_scoreboard(self, input_text: str, *arguments: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, "-B", SCRIPT, *arguments],
            input=input_text,
            check=True,
            capture_output=True,
            text=True,
        )

    def test_reconstructs_interleaved_jsonl_reads_and_writes(self) -> None:
        rows = [
            row("ar", "10ns", {"arid": "4'h2", "araddr": "32'h1000", "arlen": "8'h01"}),
            row("ar", "11ns", {"arid": "4'h3", "araddr": "32'h1800", "arlen": "8'h00"}),
            row("aw", "12ns", {"awid": "4'h1", "awaddr": "32'h2000", "awlen": "8'h01"}),
            row("aw", "13ns", {"awid": "4'h4", "awaddr": "32'h2800", "awlen": "8'h00"}),
            row("w", "14ns", {"wdata": "32'haa", "wstrb": "4'hf", "wlast": "1'h0"}),
            row("r", "15ns", {"rid": "4'h2", "rdata": "32'h11", "rresp": "2'h0", "rlast": "1'h0"}),
            row("w", "16ns", {"wdata": "32'hbb", "wstrb": "4'hf", "wlast": "1'h1"}),
            row("w", "17ns", {"wdata": "32'hcc", "wstrb": "4'h3", "wlast": "1'h1"}),
            row("r", "18ns", {"rid": "4'h3", "rdata": "32'h33", "rresp": "2'h0", "rlast": "1'h1"}),
            row("r", "19ns", {"rid": "4'h2", "rdata": "32'h22", "rresp": "2'h0", "rlast": "1'h1"}),
            row("b", "20ns", {"bid": "4'h4", "bresp": "2'h0"}),
            row("b", "21ns", {"bid": "4'h1", "bresp": "2'h0"}),
        ]
        stream = "\n".join(
            json.dumps({"type": "data", "seq": index, "data": item})
            for index, item in enumerate(rows, 1)
        )

        result = self.run_scoreboard(stream)

        self.assertIn(
            "READ start=10ns end=19ns id=4'h2 addr=32'h1000 len=8'h01 "
            "size=- burst=- beats=2 resp=[2'h0,2'h0] data=[32'h11,32'h22]",
            result.stdout,
        )
        self.assertIn(
            "READ start=11ns end=18ns id=4'h3 addr=32'h1800 len=8'h00 "
            "size=- burst=- beats=1 resp=[2'h0] data=[32'h33]",
            result.stdout,
        )
        self.assertIn(
            "WRITE start=12ns end=21ns id=4'h1 addr=32'h2000 len=8'h01 "
            "size=- burst=- beats=2 resp=2'h0 data=[32'haa,32'hbb] strb=[4'hf,4'hf]",
            result.stdout,
        )
        self.assertIn(
            "WRITE start=13ns end=20ns id=4'h4 addr=32'h2800 len=8'h00 "
            "size=- burst=- beats=1 resp=2'h0 data=[32'hcc] strb=[4'h3]",
            result.stdout,
        )
        self.assertIn(
            "SUMMARY completed_reads=2 completed_writes=2 "
            "incomplete_reads=0 incomplete_writes=0 unmatched_transfers=0",
            result.stdout,
        )
        self.assertEqual(result.stderr, "")

    def test_fatal_input_exits_nonzero(self) -> None:
        record = json.dumps({"type": "fatal", "message": "mapping failed"})

        with self.assertRaises(subprocess.CalledProcessError) as failure:
            self.run_scoreboard(record)

        self.assertIn("wavepeek: mapping failed", failure.exception.stderr)

    def test_reads_json_file_and_reports_incomplete_transactions(self) -> None:
        envelope = {
            "type": "result",
            "command": "extract axi",
            "data": [
                row("r", "20ns", {"rid": "4'h9", "rdata": "32'h0", "rlast": "1'h1"}),
                row("aw", "21ns", {"awid": "4'h3", "awaddr": "32'h3000"}),
            ],
            "diagnostics": [],
        }
        with tempfile.TemporaryDirectory() as temporary:
            input_file = pathlib.Path(temporary) / "axi.json"
            input_file.write_text(json.dumps(envelope) + "\n", encoding="utf-8")
            result = self.run_scoreboard("", str(input_file))

        self.assertIn("WARNING unmatched R transfer at 20ns id=4'h9", result.stderr)
        self.assertIn("WARNING incomplete WRITE start=21ns id=4'h3", result.stderr)
        self.assertIn(
            "SUMMARY completed_reads=0 completed_writes=0 "
            "incomplete_reads=0 incomplete_writes=1 unmatched_transfers=1",
            result.stdout,
        )


if __name__ == "__main__":
    unittest.main()
