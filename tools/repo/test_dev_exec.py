import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path


HELPER = Path(__file__).with_name("dev_exec.py")


class DevExecTests(unittest.TestCase):
    def test_forwards_terminal_signal_to_command_group(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ready = root / "ready"
            child_pid = root / "child"
            pidfile = root / "pid"
            statusfile = root / "status"
            code = (
                "import os,pathlib,signal,subprocess,time; "
                f"child=subprocess.Popen(['sleep','30']); pathlib.Path({str(child_pid)!r}).write_text(str(child.pid)); "
                f"pathlib.Path({str(ready)!r}).touch(); "
                "signal.signal(signal.SIGINT, lambda *_: raise_(43)); time.sleep(30)"
            )
            runner = subprocess.Popen(
                [
                    sys.executable,
                    str(HELPER),
                    "token",
                    str(pidfile),
                    str(statusfile),
                    str(root),
                    sys.executable,
                    "-c",
                    "import os; raise_ = lambda status: os._exit(status); " + code,
                ],
                start_new_session=True,
            )
            try:
                for _ in range(100):
                    if ready.exists():
                        break
                    time.sleep(0.02)
                self.assertTrue(ready.exists())
                os.killpg(runner.pid, signal.SIGINT)
                self.assertEqual(runner.wait(timeout=5), 43)
                pid = int(child_pid.read_text())
                with self.assertRaises(ProcessLookupError):
                    os.kill(pid, 0)
                self.assertEqual(statusfile.read_text(), "43\n")
                self.assertFalse(pidfile.exists())
            finally:
                if runner.poll() is None:
                    runner.kill()
                    runner.wait()


if __name__ == "__main__":
    unittest.main()
