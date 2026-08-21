#!/usr/bin/env python3
"""Convert waveform time values without manual decimal-place counting.

The script recognizes zs, as, fs, ps, ns, us, ms, and s suffixes and uses
exact rational arithmetic instead of floating point. With no --to option it
chooses the largest unit that produces an integer token suitable for Wavepeek.
An explicit --to unit may produce a decimal value for display or further math.

Examples:
    python3 scripts/time_convert.py 1.5ns
    1500ps

    python3 scripts/time_convert.py 1500ps --to ns
    1.5ns
"""

import argparse
import re
from fractions import Fraction

UNITS = {
    "s": 10**21,
    "ms": 10**18,
    "us": 10**15,
    "ns": 10**12,
    "ps": 10**9,
    "fs": 10**6,
    "as": 10**3,
    "zs": 1,
}
TOKEN = re.compile(r"([0-9]+(?:\.[0-9]+)?)(zs|as|fs|ps|ns|us|ms|s)")


def parse_time(token: str) -> Fraction:
    """Return a time token as an exact number of zeptoseconds."""
    match = TOKEN.fullmatch(token)
    if not match:
        raise ValueError("expected <number><unit>, for example 10ns or 1.5ps")
    magnitude, unit = match.groups()
    return Fraction(magnitude) * UNITS[unit]


def format_number(value: Fraction) -> str:
    """Format a fraction that has a finite decimal representation."""
    numerator, denominator = value.numerator, value.denominator
    twos = fives = 0
    while denominator % 2 == 0:
        denominator //= 2
        twos += 1
    while denominator % 5 == 0:
        denominator //= 5
        fives += 1
    if denominator != 1:
        raise ValueError("result has no finite decimal representation")

    places = max(twos, fives)
    numerator *= 2 ** (places - twos) * 5 ** (places - fives)
    if places == 0:
        return str(numerator)
    digits = str(numerator).zfill(places + 1)
    return f"{digits[:-places]}.{digits[-places:]}".rstrip("0").rstrip(".")


def format_time(value_zs: Fraction, unit: str | None = None) -> str:
    """Format zeptoseconds in a requested or automatically selected unit."""
    if unit is not None:
        return f"{format_number(value_zs / UNITS[unit])}{unit}"

    for candidate, scale in UNITS.items():
        magnitude = value_zs / scale
        if magnitude.denominator == 1:
            return f"{magnitude.numerator}{candidate}"
    raise ValueError("value is smaller than 1zs; choose an explicit --to unit")


def convert(token: str, unit: str | None = None) -> str:
    """Convert one time token."""
    return format_time(parse_time(token), unit)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("time", help="time value with a unit suffix, such as 1500ps")
    parser.add_argument("--to", choices=UNITS, metavar="UNIT", help="output unit")
    args = parser.parse_args()
    try:
        print(convert(args.time, args.to))
    except ValueError as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
