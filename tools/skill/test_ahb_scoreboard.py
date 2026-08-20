"""Tests for the AHB scoreboard example shipped with the Wavepeek skill."""

import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "skills" / "wavepeek" / "examples" / "ahb_scoreboard" / "ahb_scoreboard.py"


def row(
    event: str,
    time: str,
    direction: str | None = None,
    payload: dict[str, str] | None = None,
    transfer: str | None = None,
) -> dict:
    result = {
        "time": time,
        "sample_time": time,
        "profile": "ahb-lite",
        "event": event,
        "payload": payload or {},
    }
    if direction is not None:
        result["direction"] = direction
    if transfer is not None:
        result["transfer"] = transfer
    return result


class AhbScoreboardTests(unittest.TestCase):
    def run_scoreboard(self, input_text: str, *arguments: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, "-B", SCRIPT, *arguments],
            input=input_text,
            check=True,
            capture_output=True,
            text=True,
        )

    def test_combines_pipelined_address_stall_and_completion_events(self) -> None:
        rows = [
            row("address", "15ns", "read", {"haddr": "32'h1000", "hsize": "3'h2", "hburst": "3'h0"}, "nonseq"),
            row("data-stall", "20ns", "read", {"hresp": "1'h0"}),
            row("data-stall", "25ns", "read", {"hresp": "1'h0"}),
            row("data-complete", "30ns", "read", {"hresp": "1'h1", "hrdata": "32'hdeadbeef"}),
            row("address", "30ns", "write", {"haddr": "32'h2000", "hsize": "3'h2", "hburst": "3'h0"}, "nonseq"),
            row("data-complete", "35ns", "write", {"hresp": "1'h0", "hwdata": "32'ha5a55a5a", "hwstrb": "4'h5"}),
        ]
        records = [{"type": "begin", "seq": 0, "context": {"initial_data_phase": {"state": "desynchronized"}}}]
        records.extend(
            {"type": "data", "seq": index, "data": item}
            for index, item in enumerate(rows, 1)
        )
        stream = "\n".join(json.dumps(record) for record in records)

        result = self.run_scoreboard(stream)

        self.assertIn(
            "READ start=15ns end=30ns transfer=nonseq addr=32'h1000 "
            "size=3'h2 burst=3'h0 wait_cycles=2 resp=1'h1 data=32'hdeadbeef",
            result.stdout,
        )
        self.assertIn(
            "WRITE start=30ns end=35ns transfer=nonseq addr=32'h2000 "
            "size=3'h2 burst=3'h0 wait_cycles=0 resp=1'h0 "
            "data=32'ha5a55a5a strb=4'h5",
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
            row("address", "40ns", "unknown", {"haddr": "32'h3000"}, "nonseq"),
            row(
                "data-complete",
                "45ns",
                "unknown",
                {"hresp": "1'h0", "hwdata": "32'hcafef00d", "hrdata": "32'h0badc0de"},
            ),
        ]
        stream = "\n".join(
            json.dumps({"type": "data", "seq": index, "data": item})
            for index, item in enumerate(rows, 1)
        )

        result = self.run_scoreboard(stream)

        self.assertIn(
            "UNKNOWN start=40ns end=45ns transfer=nonseq addr=32'h3000 "
            "size=- burst=- wait_cycles=0 resp=1'h0 "
            "write_data=32'hcafef00d read_data=32'h0badc0de",
            result.stdout,
        )
        self.assertIn("completed_unknown=1", result.stdout)

    def test_fatal_input_exits_nonzero(self) -> None:
        record = json.dumps({"type": "fatal", "message": "mapping failed"})

        with self.assertRaises(subprocess.CalledProcessError) as failure:
            self.run_scoreboard(record)

        self.assertIn("wavepeek: mapping failed", failure.exception.stderr)

    def test_uses_initial_data_phase_from_json_context(self) -> None:
        completion = row("data-complete", "10ns", "read")
        completion.pop("payload")
        envelope = {
            "type": "result",
            "command": "extract ahb",
            "context": {
                "initial_data_phase": {
                    "state": "pending",
                    "address": {
                        "time": "5ns",
                        "sample_time": "4ns",
                        "transfer": "nonseq",
                        "direction": "read",
                        "payload": {"haddr": "32'h3000", "hsize": "3'h2"},
                    },
                }
            },
            "data": [completion],
            "diagnostics": [],
        }
        with tempfile.TemporaryDirectory() as temporary:
            input_file = pathlib.Path(temporary) / "ahb.json"
            input_file.write_text(json.dumps(envelope) + "\n", encoding="utf-8")
            result = self.run_scoreboard("", str(input_file))

        self.assertIn(
            "READ start=5ns end=10ns transfer=nonseq addr=32'h3000 "
            "size=3'h2 burst=- wait_cycles=0 resp=- data=-",
            result.stdout,
        )
        self.assertIn("completed_reads=1", result.stdout)
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
