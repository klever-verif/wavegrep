import json
import os
import signal
import subprocess
import tempfile
import time
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
class GitHookTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.tmp = Path(self.temp.name)
        self.home = self.tmp / "home"
        self.home.mkdir()
        self.data_home = self.tmp / "data"
        self.main = self.tmp / "main"
        self.linked = self.tmp / "linked"
        self.fake_bin = self.tmp / "bin"
        self.log = self.tmp / "calls.jsonl"
        self.fake_bin.mkdir()
        self._write_fake_pre_commit()
        docker = self.fake_bin / "docker"
        docker.write_text("#!/bin/sh\necho docker-called >>\"$FAKE_LOG\"\nexit 99\n")
        docker.chmod(0o755)
        self._init_repository()

    def _env(self, **updates: str) -> dict[str, str]:
        env = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
        env["HOME"] = str(self.home)
        env["XDG_DATA_HOME"] = str(self.data_home)
        env["PATH"] = f"{self.fake_bin}{os.pathsep}{env['PATH']}"
        env["FAKE_LOG"] = str(self.log)
        env["FAKE_ROOT"] = str(self.main)
        env.update(updates)
        return env

    def _git(
        self, *args: str, cwd: Path | None = None, env: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", *args],
            cwd=cwd or self.main,
            env=env or self._env(),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )

    def _init_repository(self) -> None:
        self.main.mkdir()
        self._git("init", "-q", "-b", "main")
        self._git("config", "user.name", "Test User")
        self._git("config", "user.email", "test@example.com")
        (self.main / "tracked").write_text("base\n")
        self._git("add", "tracked")
        self._git("commit", "-qm", "test: initial")
        self._git("worktree", "add", "-qb", "linked", str(self.linked))
        for root in (self.main, self.linked):
            (root / "tools" / "repo").mkdir(parents=True)
            (root / "dev").write_bytes((REPO_ROOT / "dev").read_bytes())
            (root / "dev").chmod(0o755)
            hook = root / "tools" / "repo" / "git-hook"
            hook.write_bytes((REPO_ROOT / "tools/repo/git-hook").read_bytes())
            hook.chmod(0o755)

    def _write_fake_pre_commit(self) -> None:
        script = self.fake_bin / "pre-commit"
        script.write_text(
            """#!/usr/bin/env python3
import json, os, pathlib, signal, subprocess, sys, time
record = {
    "kind": "pre-commit",
    "args": sys.argv[1:],
    "git_env": {k: v for k, v in os.environ.items() if k.startswith("GIT_")},
    "skip": os.environ.get("SKIP"),
}
if os.environ.get("PROBE_INDEX"):
    record["staged"] = subprocess.run(
        ["git", "show", ":tracked"], check=True, capture_output=True, text=True
    ).stdout
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(record) + "\\n")
marker = os.environ.get("WAIT_MARKER")
if marker:
    pathlib.Path(marker).touch()
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(42))
    time.sleep(30)
data = sys.stdin.read()
print(f"stdout:{data}", end="")
print("stderr", file=sys.stderr)
raise SystemExit(int(os.environ.get("FAKE_STATUS", "0")))
"""
        )
        script.chmod(0o755)

    def _install(self, **env_updates: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(self.main / "dev"), "--install-hooks"],
            cwd=self.main,
            env=self._env(**env_updates),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    @property
    def hooks_dir(self) -> Path:
        configured = self._git("config", "--local", "--get", "core.hooksPath").stdout.strip()
        return Path(configured)

    def _replace_installed_dev(self) -> None:
        installed = self.hooks_dir / "dev"
        installed.write_text(
            """#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
assert args.pop(0) == "--exec-only"
assert args.pop(0) == "env"
assignments = {}
while args and "=" in args[0]:
    key, value = args.pop(0).split("=", 1)
    assignments[key] = value
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps({"kind": "dev", "env": assignments, "args": args}) + "\\n")
env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
workspace = "/workspaces/" + pathlib.Path(os.environ["FAKE_ROOT"]).name
def host_path(value):
    if value == workspace or value.startswith(workspace + "/"):
        return os.environ["FAKE_ROOT"] + value[len(workspace):]
    return value
for key, value in assignments.items():
    env[key] = host_path(value)
args = [host_path(value) for value in args]
os.execvpe(args[0], args, env)
"""
        )
        installed.chmod(0o755)

    def _run_hook(
        self,
        root: Path,
        hook: str,
        *args: str,
        env_updates: dict[str, str] | None = None,
        input_text: str = "",
    ) -> subprocess.CompletedProcess[str]:
        env = self._env(FAKE_ROOT=str(root))
        if env_updates:
            env.update(env_updates)
        return subprocess.run(
            [str(self.hooks_dir / hook), *args],
            cwd=root,
            env=env,
            input=input_text,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def _calls(self) -> list[dict[str, object]]:
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_install_is_idempotent_reviewed_and_refuses_custom_path(self) -> None:
        result = self._install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            self._git("config", "--local", "--get", "core.hooksPath").stdout.strip(),
            str(self.hooks_dir),
        )
        self.assertTrue(self.hooks_dir.is_relative_to(self.data_home))
        self.assertFalse(self.hooks_dir.is_relative_to(self.main))
        self.assertEqual(self.hooks_dir.stat().st_mode & 0o777, 0o700)
        self.assertEqual((self.hooks_dir / "dev").read_bytes(), (self.main / "dev").read_bytes())
        source = (self.main / "tools/repo/git-hook").read_bytes()
        for name in ("pre-commit", "commit-msg"):
            installed = self.hooks_dir / name
            self.assertEqual(installed.read_bytes(), source)
            self.assertTrue(installed.stat().st_mode & 0o111)
        self.assertEqual(self._install().returncode, 0)
        self.assertFalse(self.log.exists(), "installation contacted Docker")

        self._git("config", "--local", "core.hooksPath", "custom-hooks")
        result = self._install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refusing to replace existing core.hooksPath", result.stderr)
        self.assertEqual(
            self._git("config", "--local", "--get", "core.hooksPath").stdout.strip(),
            "custom-hooks",
        )

    def test_install_rejects_data_storage_inside_any_worktree_or_common_dir(self) -> None:
        cases = (self.main, self.linked, self.main / ".git", self.tmp / "worktree-link")
        for data_home in cases:
            if data_home.name == "worktree-link":
                data_home.symlink_to(self.linked, target_is_directory=True)
            result = self._install(XDG_DATA_HOME=str(data_home))
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("must be outside", result.stderr)

    def test_install_respects_effective_global_custom_path(self) -> None:
        self._git("config", "--global", "core.hooksPath", "/custom/hooks")
        result = self._install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("/custom/hooks", result.stderr)
        local = subprocess.run(
            ["git", "config", "--local", "--get", "core.hooksPath"],
            cwd=self.main,
            env=self._env(),
            check=False,
        )
        self.assertNotEqual(local.returncode, 0)

    def test_maps_main_and_linked_worktree_paths_and_filters_environment(self) -> None:
        self.assertEqual(self._install().returncode, 0)
        self._replace_installed_dev()
        host_objects = self.main / ".git" / "objects"
        result = self._run_hook(
            self.main,
            "pre-commit",
            env_updates={
                "GIT_OBJECT_DIRECTORY": str(host_objects),
                "GIT_AUTHOR_NAME": "Host Author",
                "SKIP": "rust-lint",
            },
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        main_call = self._calls()[0]
        self.assertEqual(
            main_call["env"],
            {
                "GIT_WORK_TREE": "/workspaces/main",
                "GIT_DIR": "/workspaces/main/.git",
                "GIT_COMMON_DIR": "/workspaces/main/.git",
                "GIT_INDEX_FILE": "/workspaces/main/.git/index",
                "SKIP": "rust-lint",
            },
        )
        pre_commit_call = self._calls()[1]
        self.assertNotIn("GIT_OBJECT_DIRECTORY", pre_commit_call["git_env"])
        self.assertNotIn("GIT_AUTHOR_NAME", pre_commit_call["git_env"])

        self.log.unlink()
        result = self._run_hook(self.linked, "pre-commit")
        self.assertEqual(result.returncode, 0, result.stderr)
        linked_call = self._calls()[0]
        common = str((self.main / ".git").resolve())
        self.assertEqual(linked_call["env"]["GIT_WORK_TREE"], "/workspaces/linked")
        self.assertEqual(
            linked_call["env"]["GIT_DIR"], f"{common}/worktrees/linked"
        )
        self.assertEqual(linked_call["env"]["GIT_COMMON_DIR"], common)
        self.assertEqual(
            linked_call["env"]["GIT_INDEX_FILE"], f"{common}/worktrees/linked/index"
        )

    def test_temporary_index_exposes_exact_staged_content(self) -> None:
        self.assertEqual(self._install().returncode, 0)
        self._replace_installed_dev()
        temporary_index = self.main / ".git" / "temporary index"
        temporary_index.write_bytes((self.main / ".git" / "index").read_bytes())
        (self.main / "tracked").write_text("staged\n")
        self._git("add", "tracked", env=self._env(GIT_INDEX_FILE=str(temporary_index)))
        (self.main / "tracked").write_text("working\n")

        result = self._run_hook(
            self.main,
            "pre-commit",
            env_updates={"GIT_INDEX_FILE": str(temporary_index), "PROBE_INDEX": "1"},
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self._calls()
        self.assertEqual(
            calls[0]["env"]["GIT_INDEX_FILE"],
            "/workspaces/main/.git/temporary index",
        )
        self.assertEqual(calls[1]["staged"], "staged\n")

    def test_commit_message_path_and_outside_paths(self) -> None:
        self.assertEqual(self._install().returncode, 0)
        self._replace_installed_dev()
        message = self.main / ".git" / "message with spaces"
        message.write_text("chore: valid\n")
        result = self._run_hook(self.main, "commit-msg", str(message))
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self._calls()
        self.assertEqual(
            calls[0]["args"],
            [
                "pre-commit",
                "run",
                "--hook-stage",
                "commit-msg",
                "--commit-msg-filename",
                "/workspaces/main/.git/message with spaces",
            ],
        )
        outside = self.tmp / "outside"
        outside.write_text("chore: valid\n")
        result = self._run_hook(self.main, "commit-msg", str(outside))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the mounted worktree", result.stderr)
        result = self._run_hook(
            self.main,
            "pre-commit",
            env_updates={"GIT_INDEX_FILE": str(outside)},
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the mounted worktree", result.stderr)

    def test_preserves_process_io_and_exit_status(self) -> None:
        self.assertEqual(self._install().returncode, 0)
        self._replace_installed_dev()
        result = self._run_hook(
            self.main,
            "pre-commit",
            env_updates={"FAKE_STATUS": "23"},
            input_text="stdin\n",
        )
        self.assertEqual(result.returncode, 23)
        self.assertEqual(result.stdout, "stdout:stdin\n")
        self.assertEqual(result.stderr, "stderr\n")

    def test_signal_reaches_hook_process(self) -> None:
        self.assertEqual(self._install().returncode, 0)
        self._replace_installed_dev()
        marker = self.tmp / "ready"
        env = self._env(FAKE_ROOT=str(self.main), WAIT_MARKER=str(marker))
        process = subprocess.Popen(
            [str(self.hooks_dir / "pre-commit")],
            cwd=self.main,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            for _ in range(100):
                if marker.exists():
                    break
                time.sleep(0.02)
            self.assertTrue(marker.exists(), "hook process did not start")
            process.send_signal(signal.SIGTERM)
            stdout, stderr = process.communicate(timeout=5)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
        self.assertEqual(process.returncode, 42, (stdout, stderr))


if __name__ == "__main__":
    unittest.main()
