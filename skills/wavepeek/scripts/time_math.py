#!/usr/bin/env python3
"""Add or subtract waveform time values expressed in different units.

This script is useful for deriving Wavepeek --from, --to, and --at values
without converting units by hand. It reuses time_convert.py, keeps all math
exact, and rejects negative results because Wavepeek timestamps are unsigned.
With no --to option it emits the largest exact integer unit.

Examples:
    python3 scripts/time_math.py 10ns + 250ps
    10250ps

    python3 scripts/time_math.py 10ns - 250ps --to ns
    9.75ns
"""

import argparse

from time_convert import UNITS, format_time, parse_time


def calculate(left: str, operator: str, right: str, unit: str | None = None) -> str:
    """Calculate one timestamp addition or subtraction."""
    left_zs = parse_time(left)
    right_zs = parse_time(right)
    result = left_zs + right_zs if operator == "+" else left_zs - right_zs
    if result < 0:
        raise ValueError("result is negative; Wavepeek timestamps cannot be negative")
    return format_time(result, unit)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("left", help="first time value, such as 10ns")
    parser.add_argument("operator", choices=("+", "-"))
    parser.add_argument("right", help="second time value, such as 250ps")
    parser.add_argument("--to", choices=UNITS, metavar="UNIT", help="output unit")
    args = parser.parse_args()
    try:
        print(calculate(args.left, args.operator, args.right, args.to))
    except ValueError as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
