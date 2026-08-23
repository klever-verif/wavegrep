#!/usr/bin/env python3
"""Compare data rows from two complete Wavepeek JSON or JSONL results.

The script ignores machine-output envelopes and JSONL sequence numbers. It
aligns canonical data rows with difflib, prints the first differing block by
default, and prints every block with --all. It does not read waveform files.
"""

import argparse
import difflib
import json
from pathlib import Path


def load_result(path: Path) -> tuple[str, list[dict]]:
    records = [
        json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()
    ]
    for record in records:
        if record.get("type") == "fatal":
            raise SystemExit(f"{path}: {record['message']}")

    if len(records) == 1 and records[0].get("type") == "result":
        result = records[0]
        require_complete(path, result.get("summary"))
        return result["command"], result.get("data", [])

    end_positions = [
        index for index, record in enumerate(records) if record.get("type") == "end"
    ]
    if not end_positions:
        raise SystemExit(f"{path}: incomplete JSONL stream: missing end record")
    if len(end_positions) != 1 or end_positions[0] != len(records) - 1:
        raise SystemExit(f"{path}: invalid JSONL stream: end record must be final")
    end = records[end_positions[0]]
    require_complete(path, end.get("summary"))

    begin = next((record for record in records if record.get("type") == "begin"), {})
    rows = [record["data"] for record in records if record.get("type") == "data"]
    return begin.get("command", "unknown"), rows


def require_complete(path: Path, summary: dict | None) -> None:
    if summary is not None and not summary.get("complete", False):
        raise SystemExit(f"{path}: result is incomplete; rerun Wavepeek with --max unlimited")


def canonical(row: dict) -> str:
    return json.dumps(row, sort_keys=True, separators=(",", ":"))


def location(rows: list[dict], start: int, end: int) -> str:
    if start == end:
        return f"rows={start}:{end} time=-"
    row = rows[start]
    result = f"rows={start}:{end} time={row.get('time', '-')}"
    if "sample_time" in row:
        result += f" sample_time={row['sample_time']}"
    return result


def compare(left: list[dict], right: list[dict], show_all: bool) -> bool:
    matcher = difflib.SequenceMatcher(
        None,
        [canonical(row) for row in left],
        [canonical(row) for row in right],
        autojunk=False,
    )
    blocks = [opcode for opcode in matcher.get_opcodes() if opcode[0] != "equal"]
    if not blocks:
        print(f"EQUAL rows={len(left)}")
        return False

    shown = blocks if show_all else blocks[:1]
    for number, (operation, left_start, left_end, right_start, right_end) in enumerate(shown, 1):
        print(f"DIFF {number} {operation}")
        print(f"  left  {location(left, left_start, left_end)}")
        print(f"  right {location(right, right_start, right_end)}")
        for row in left[left_start:left_end]:
            print(f"- {canonical(row)}")
        for row in right[right_start:right_end]:
            print(f"+ {canonical(row)}")

    removed = sum(left_end - left_start for _, left_start, left_end, _, _ in blocks)
    added = sum(right_end - right_start for _, _, _, right_start, right_end in blocks)
    print(
        f"SUMMARY equal=false left_rows={len(left)} right_rows={len(right)} "
        f"differing_blocks={len(blocks)} removed_rows={removed} added_rows={added} "
        f"shown_blocks={len(shown)}"
    )
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("left", type=Path, help="baseline Wavepeek JSON or JSONL")
    parser.add_argument("right", type=Path, help="comparison Wavepeek JSON or JSONL")
    parser.add_argument("--all", action="store_true", help="show every differing block")
    args = parser.parse_args()

    left_command, left_rows = load_result(args.left)
    right_command, right_rows = load_result(args.right)
    if left_command != right_command:
        raise SystemExit(f"commands differ: {left_command!r} != {right_command!r}")
    raise SystemExit(1 if compare(left_rows, right_rows, args.all) else 0)


if __name__ == "__main__":
    main()
