#!/usr/bin/env python3
"""Combine Wavepeek APB phase events into complete APB transactions.

The script reads `wavepeek extract apb --jsonl` records from stdin or a file.
A Setup event starts one transaction, optional Access wait events count wait
cycles, and the Access completion supplies read data and the error response.
Completed transactions and a final summary are printed as plain text.

This is a readable demonstration, not an APB protocol checker.
"""

import argparse
import json
import sys
from dataclasses import dataclass
from typing import TextIO


@dataclass
class Transaction:
    start: str
    direction: str
    payload: dict[str, str]
    wait_cycles: int = 0


class ApbScoreboard:
    def __init__(self, output: TextIO = sys.stdout, errors: TextIO = sys.stderr):
        self.output = output
        self.errors = errors
        self.pending: Transaction | None = None
        self.completed_reads = 0
        self.completed_writes = 0
        self.completed_unknown = 0
        self.unmatched_transfers = 0

    def accept(self, row: dict) -> None:
        event = row["event"]
        if event == "setup":
            self.pending = Transaction(row["time"], row["direction"], row["payload"])
        elif event == "access-wait":
            if self.pending is None:
                self.warn(f"unmatched Access wait at {row['time']}")
            else:
                self.pending.wait_cycles += 1
        elif event == "access-complete":
            self.complete(row)

    def complete(self, row: dict) -> None:
        if self.pending is None:
            self.warn(f"unmatched Access completion at {row['time']}")
            return

        transaction = self.pending
        self.pending = None
        request = transaction.payload
        response = row["payload"]
        common = (
            f"start={transaction.start} end={row['time']} "
            f"addr={request.get('paddr', '-')} pprot={request.get('pprot', '-')} "
            f"wait_cycles={transaction.wait_cycles} "
            f"error={response.get('pslverr', '-')}"
        )

        if transaction.direction == "write":
            self.completed_writes += 1
            print(
                f"WRITE {common} data={request.get('pwdata', '-')} "
                f"strb={request.get('pstrb', '-')}",
                file=self.output,
            )
        elif transaction.direction == "read":
            self.completed_reads += 1
            print(
                f"READ {common} data={response.get('prdata', '-')}",
                file=self.output,
            )
        else:
            self.completed_unknown += 1
            print(
                f"UNKNOWN {common} write_data={request.get('pwdata', '-')} "
                f"read_data={response.get('prdata', '-')}",
                file=self.output,
            )

    def warn(self, message: str) -> None:
        self.unmatched_transfers += 1
        print(f"WARNING {message}", file=self.errors)

    def finish(self) -> None:
        incomplete = int(self.pending is not None)
        if self.pending is not None:
            print(
                f"WARNING incomplete {self.pending.direction.upper()} "
                f"start={self.pending.start} "
                f"addr={self.pending.payload.get('paddr', '-')} "
                f"wait_cycles={self.pending.wait_cycles}",
                file=self.errors,
            )

        print(
            f"SUMMARY completed_reads={self.completed_reads} "
            f"completed_writes={self.completed_writes} "
            f"completed_unknown={self.completed_unknown} "
            f"incomplete={incomplete} "
            f"unmatched_transfers={self.unmatched_transfers}",
            file=self.output,
        )


def transfer_rows(stream: TextIO):
    """Yield APB events from Wavepeek JSON or JSONL records."""
    for line in stream:
        record = json.loads(line)
        if record.get("type") == "fatal":
            raise SystemExit(f"wavepeek: {record['message']}")
        if record.get("type") == "data":
            yield record["data"]
        elif record.get("type") == "result":
            yield from record.get("data", [])


def run(stream: TextIO) -> None:
    scoreboard = ApbScoreboard()
    for row in transfer_rows(stream):
        scoreboard.accept(row)
    scoreboard.finish()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("input", nargs="?", help="Wavepeek JSON or JSONL file; default: stdin")
    args = parser.parse_args()

    if args.input:
        with open(args.input, encoding="utf-8") as stream:
            run(stream)
    else:
        run(sys.stdin)


if __name__ == "__main__":
    main()
