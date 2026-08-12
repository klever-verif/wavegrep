#!/usr/bin/env python3
"""Run one devcontainer command and forward signals to its process group."""

import os
import signal
import subprocess
import sys
from pathlib import Path


def main() -> int:
    token, pidfile_arg, statusfile_arg, cwd, *command = sys.argv[1:]
    pidfile = Path(pidfile_arg)
    statusfile = Path(statusfile_arg)
    child: subprocess.Popen[bytes] | None = None
    received_signal = False
    pending_signal: int | None = None

    def forward(signum: int, _frame: object) -> None:
        nonlocal pending_signal, received_signal
        received_signal = True
        pending_signal = signum
        if child is not None:
            try:
                os.killpg(child.pid, signum)
            except ProcessLookupError:
                pass

    for signum in (signal.SIGHUP, signal.SIGINT, signal.SIGQUIT, signal.SIGTERM):
        signal.signal(signum, forward)

    start_time = Path(f"/proc/{os.getpid()}/stat").read_text().split()[21]
    pidfile.write_text(f"{token} {os.getpid()} {start_time}\n")

    try:
        child = subprocess.Popen(command, cwd=cwd, process_group=0)
        if pending_signal is not None:
            forward(pending_signal, None)
        returncode = child.wait()

        if received_signal:
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass

        status = 128 - returncode if returncode < 0 else returncode
        statusfile.write_text(f"{status}\n")
        return status
    finally:
        pidfile.unlink(missing_ok=True)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"error: dev: {error}", file=sys.stderr)
        raise SystemExit(1) from error
