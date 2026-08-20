"""Tests for the time helpers shipped with the Wavepeek skill."""

import importlib.util
import pathlib
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "skills" / "wavepeek_v3" / "scripts"


def load_module(name: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


time_convert = load_module("time_convert")
time_math = load_module("time_math")


class TimeConvertTests(unittest.TestCase):
    def test_all_unit_pairs_round_trip_exactly(self) -> None:
        for source in time_convert.UNITS:
            token = f"1{source}"
            for target in time_convert.UNITS:
                with self.subTest(source=source, target=target):
                    converted = time_convert.convert(token, target)
                    self.assertEqual(
                        time_convert.parse_time(converted),
                        time_convert.parse_time(token),
                    )

    def test_automatic_output_is_an_integer_wavepeek_token(self) -> None:
        self.assertEqual(time_convert.convert("1.5ns"), "1500ps")
        self.assertEqual(time_convert.convert("1000000ps"), "1us")
        self.assertEqual(time_convert.convert("0.000001fs"), "1zs")

    def test_invalid_and_too_small_values_are_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected <number><unit>"):
            time_convert.convert("10")
        with self.assertRaisesRegex(ValueError, "smaller than 1zs"):
            time_convert.convert("0.1zs")

    def test_convert_cli(self) -> None:
        result = subprocess.run(
            [sys.executable, "-B", SCRIPTS / "time_convert.py", "1500ps", "--to", "ns"],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.stdout, "1.5ns\n")
        self.assertEqual(result.stderr, "")


class TimeMathTests(unittest.TestCase):
    def test_adds_and_subtracts_mixed_units(self) -> None:
        self.assertEqual(time_math.calculate("1us", "+", "500ns"), "1500ns")
        self.assertEqual(time_math.calculate("1us", "-", "500ns", "us"), "0.5us")

    def test_negative_result_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "result is negative"):
            time_math.calculate("1ns", "-", "2ns")

    def test_math_cli(self) -> None:
        result = subprocess.run(
            [sys.executable, "-B", SCRIPTS / "time_math.py", "10ns", "+", "250ps"],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.stdout, "10250ps\n")
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
