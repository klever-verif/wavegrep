#!/usr/bin/env python3
"""Reconstruct simple AXI transactions from Wavepeek JSON or JSONL output.

The script reads `wavepeek extract axi --jsonl` records from stdin or a file.
It groups read data by RID until RLAST, assigns AXI4 write data in address
order, and matches write responses to AWID through BID. Completed transactions
and a final summary are printed as plain text.

This is a readable demonstration, not an AXI protocol checker. It assumes the
selected trace contains each AW transfer before its W transfers and uses WLAST
and RLAST to end bursts.
"""

import argparse
import json
import sys
from collections import defaultdict, deque
from dataclasses import dataclass, field
from typing import TextIO


@dataclass
class Transaction:
    start: str
    payload: dict[str, str]
    beats: list[dict[str, str]] = field(default_factory=list)
    data_complete: bool = False


class AxiScoreboard:
    def __init__(self, output: TextIO = sys.stdout, errors: TextIO = sys.stderr):
        self.output = output
        self.errors = errors
        self.reads = defaultdict(deque)
        self.writes = defaultdict(deque)
        self.write_data = deque()
        self.completed_reads = 0
        self.completed_writes = 0
        self.unmatched_transfers = 0

    def accept(self, row: dict) -> None:
        channel = row["channel"]
        if channel == "ar":
            self.on_ar(row)
        elif channel == "r":
            self.on_r(row)
        elif channel == "aw":
            self.on_aw(row)
        elif channel == "w":
            self.on_w(row)
        elif channel == "b":
            self.on_b(row)

    def on_ar(self, row: dict) -> None:
        transaction_id = row["payload"].get("arid", "0")
        self.reads[transaction_id].append(Transaction(row["time"], row["payload"]))

    def on_r(self, row: dict) -> None:
        transaction_id = row["payload"].get("rid", "0")
        if not self.reads[transaction_id]:
            self.warn(f"unmatched R transfer at {row['time']} id={transaction_id}")
            return

        transaction = self.reads[transaction_id][0]
        transaction.beats.append(row["payload"])
        if row["payload"].get("rlast") == "1'h1":
            self.reads[transaction_id].popleft()
            self.completed_reads += 1
            self.print_read(transaction_id, transaction, row)

    def on_aw(self, row: dict) -> None:
        transaction_id = row["payload"].get("awid", "0")
        transaction = Transaction(row["time"], row["payload"])
        self.writes[transaction_id].append(transaction)
        self.write_data.append(transaction)

    def on_w(self, row: dict) -> None:
        if not self.write_data:
            self.warn(f"unmatched W transfer at {row['time']}")
            return

        transaction = self.write_data[0]
        transaction.beats.append(row["payload"])
        if row["payload"].get("wlast") == "1'h1":
            transaction.data_complete = True
            self.write_data.popleft()

    def on_b(self, row: dict) -> None:
        transaction_id = row["payload"].get("bid", "0")
        if not self.writes[transaction_id]:
            self.warn(f"unmatched B transfer at {row['time']} id={transaction_id}")
            return

        transaction = self.writes[transaction_id][0]
        if not transaction.data_complete:
            self.warn(f"B transfer before final W at {row['time']} id={transaction_id}")
            return

        self.writes[transaction_id].popleft()
        self.completed_writes += 1
        self.print_write(transaction_id, transaction, row)

    def print_read(self, transaction_id: str, transaction: Transaction, row: dict) -> None:
        request = transaction.payload
        data = ",".join(beat.get("rdata", "-") for beat in transaction.beats)
        responses = ",".join(beat.get("rresp", "-") for beat in transaction.beats)
        print(
            f"READ start={transaction.start} end={row['time']} id={transaction_id} "
            f"addr={request.get('araddr', '-')} len={request.get('arlen', '-')} "
            f"size={request.get('arsize', '-')} burst={request.get('arburst', '-')} "
            f"beats={len(transaction.beats)} resp=[{responses}] data=[{data}]",
            file=self.output,
        )

    def print_write(self, transaction_id: str, transaction: Transaction, row: dict) -> None:
        request = transaction.payload
        data = ",".join(beat.get("wdata", "-") for beat in transaction.beats)
        strobes = ",".join(beat.get("wstrb", "-") for beat in transaction.beats)
        print(
            f"WRITE start={transaction.start} end={row['time']} id={transaction_id} "
            f"addr={request.get('awaddr', '-')} len={request.get('awlen', '-')} "
            f"size={request.get('awsize', '-')} burst={request.get('awburst', '-')} "
            f"beats={len(transaction.beats)} resp={row['payload'].get('bresp', '-')} "
            f"data=[{data}] strb=[{strobes}]",
            file=self.output,
        )

    def warn(self, message: str) -> None:
        self.unmatched_transfers += 1
        print(f"WARNING {message}", file=self.errors)

    def finish(self) -> None:
        incomplete_reads = sum(len(queue) for queue in self.reads.values())
        incomplete_writes = sum(len(queue) for queue in self.writes.values())

        for transaction_id, queue in self.reads.items():
            for transaction in queue:
                print(
                    f"WARNING incomplete READ start={transaction.start} id={transaction_id} "
                    f"received_beats={len(transaction.beats)}",
                    file=self.errors,
                )
        for transaction_id, queue in self.writes.items():
            for transaction in queue:
                print(
                    f"WARNING incomplete WRITE start={transaction.start} id={transaction_id} "
                    f"received_beats={len(transaction.beats)}",
                    file=self.errors,
                )

        print(
            f"SUMMARY completed_reads={self.completed_reads} "
            f"completed_writes={self.completed_writes} "
            f"incomplete_reads={incomplete_reads} "
            f"incomplete_writes={incomplete_writes} "
            f"unmatched_transfers={self.unmatched_transfers}",
            file=self.output,
        )


def transfer_rows(stream: TextIO):
    """Yield AXI transfer rows from Wavepeek JSON or JSONL records."""
    for line in stream:
        record = json.loads(line)
        if record.get("type") == "fatal":
            raise SystemExit(f"wavepeek: {record['message']}")
        if record.get("type") == "data":
            yield record["data"]
        elif record.get("type") == "result":
            yield from record.get("data", [])


def run(stream: TextIO) -> None:
    scoreboard = AxiScoreboard()
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
