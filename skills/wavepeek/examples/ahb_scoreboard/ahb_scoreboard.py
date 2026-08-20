#!/usr/bin/env python3
"""Combine Wavepeek AHB phase events into complete AHB transfers.

The script reads `wavepeek extract ahb --jsonl` records from stdin or a file.
An accepted address starts one transfer, optional data-stall events count wait
cycles, and the following data-complete event supplies data and the response.
Completed transfers and a final summary are printed as plain text.

This is a readable demonstration, not an AHB protocol checker or burst decoder.
"""

import argparse
import json
import sys
from dataclasses import dataclass
from typing import TextIO


@dataclass
class Transaction:
    start: str
    transfer: str
    direction: str
    payload: dict[str, str]
    wait_cycles: int = 0


class AhbScoreboard:
    def __init__(self, output: TextIO = sys.stdout, errors: TextIO = sys.stderr):
        self.output = output
        self.errors = errors
        self.pending: Transaction | None = None
        self.completed_reads = 0
        self.completed_writes = 0
        self.completed_unknown = 0
        self.unmatched_transfers = 0

    def seed_context(self, context: dict) -> None:
        initial = context.get("initial_data_phase", {})
        if initial.get("state") == "pending":
            self.start(initial["address"])

    def accept(self, row: dict) -> None:
        event = row["event"]
        if event == "address":
            self.start(row)
        elif event == "data-stall":
            if self.pending is None:
                self.warn(f"unmatched data stall at {row['time']}")
            else:
                self.pending.wait_cycles += 1
        elif event == "data-complete":
            self.complete(row)

    def start(self, row: dict) -> None:
        self.pending = Transaction(
            row["time"],
            row["transfer"],
            row["direction"],
            row["payload"],
        )

    def complete(self, row: dict) -> None:
        if self.pending is None:
            self.warn(f"unmatched data completion at {row['time']}")
            return

        transaction = self.pending
        self.pending = None
        request = transaction.payload
        response = row.get("payload", {})
        common = (
            f"start={transaction.start} end={row['time']} "
            f"transfer={transaction.transfer} addr={request.get('haddr', '-')} "
            f"size={request.get('hsize', '-')} burst={request.get('hburst', '-')} "
            f"wait_cycles={transaction.wait_cycles} resp={response.get('hresp', '-')}"
        )

        if transaction.direction == "write":
            self.completed_writes += 1
            print(
                f"WRITE {common} data={response.get('hwdata', '-')} "
                f"strb={response.get('hwstrb', '-')}",
                file=self.output,
            )
        elif transaction.direction == "read":
            self.completed_reads += 1
            print(
                f"READ {common} data={response.get('hrdata', '-')}",
                file=self.output,
            )
        else:
            self.completed_unknown += 1
            print(
                f"UNKNOWN {common} write_data={response.get('hwdata', '-')} "
                f"read_data={response.get('hrdata', '-')}",
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
                f"addr={self.pending.payload.get('haddr', '-')} "
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


def run(stream: TextIO) -> None:
    scoreboard = AhbScoreboard()
    for line in stream:
        record = json.loads(line)
        if record.get("type") == "fatal":
            raise SystemExit(f"wavepeek: {record['message']}")
        if record.get("type") in ("begin", "result"):
            scoreboard.seed_context(record.get("context", {}))
        if record.get("type") == "data":
            scoreboard.accept(record["data"])
        elif record.get("type") == "result":
            for row in record.get("data", []):
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
