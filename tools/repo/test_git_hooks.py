import json
import os
import subprocess
import tempfile
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
        self.main = self.tmp / "main"
        self.linked = self.tmp / "linked"
        self.fake_bin = self.tmp / "bin"
        self.log = self.tmp / "calls.jsonl"
        self.fake_bin.mkdir()
        self._write_fake_pre_commit()
        self._init_repository()

    def _env(self, **updates: str) -> dict[str, str]:
        env = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
        env.update(HOME=str(self.home), PATH=f"{self.fake_bin}{os.pathsep}{env['PATH']}", FAKE_LOG=str(self.log))
        env.update(updates)
        return env

    def _git(self, root: Path, *args: str, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", *args], cwd=root, env=env or self._env(), text=True,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
        )

    def _init_repository(self) -> None:
        self.main.mkdir()
        subprocess.run(["git", "init", "-q", "-b", "main"], cwd=self.main, env=self._env(), check=True)
        self._git(self.main, "config", "user.name", "Test User")
        self._git(self.main, "config", "user.email", "test@example.com")
        (self.main / "tracked").write_text("base\n")
        self._git(self.main, "add", "tracked")
        self._git(self.main, "commit", "-qm", "test: initial")
        self._git(self.main, "worktree", "add", "-qb", "linked", str(self.linked))
        for root in (self.main, self.linked):
            (root / "tools/repo").mkdir(parents=True)
            (root / "dev").write_bytes((REPO_ROOT / "dev").read_bytes())
            (root / "dev").chmod(0o755)
            hook = root / "tools/repo/git-hook"
            hook.write_bytes((REPO_ROOT / "tools/repo/git-hook").read_bytes())
            hook.chmod(0o755)

    def _write_fake_pre_commit(self) -> None:
        script = self.fake_bin / "pre-commit"
        script.write_text(
            """#!/usr/bin/env python3
import json, os, subprocess, sys
record = {"args": sys.argv[1:], "git_env": {k: v for k, v in os.environ.items() if k.startswith("GIT_")}}
if os.environ.get("PROBE_INDEX"):
    record["staged"] = subprocess.run(["git", "show", ":tracked"], check=True, capture_output=True, text=True).stdout
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(record) + "\\n")
raise SystemExit(int(os.environ.get("FAKE_STATUS", "0")))
"""
        )
        script.chmod(0o755)

    def _install(self, root: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(root / "dev"), "--install-hooks"], cwd=root, env=self._env(),
            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )

    def _hooks_dir(self, root: Path) -> Path:
        return Path(self._git(root, "config", "--worktree", "--get", "core.hooksPath").stdout.strip())

    def _replace_dev(self, root: Path) -> None:
        installed = self._hooks_dir(root) / "dev"
        installed.write_text(
            """#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
assert args.pop(0) == "--exec-only" and args.pop(0) == "env"
assignments = {}
while args and "=" in args[0]:
    key, value = args.pop(0).split("=", 1); assignments[key] = value
workspace = "/workspaces/" + pathlib.Path(os.environ["FAKE_ROOT"]).name
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps({"dev_env": assignments, "args": args}) + "\\n")
def host(value):
    return os.environ["FAKE_ROOT"] + value[len(workspace):] if value == workspace or value.startswith(workspace + "/") else value
env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
env.update({key: host(value) for key, value in assignments.items()})
os.execvpe(args[0], [host(value) for value in args], env)
"""
        )
        installed.chmod(0o755)

    def _run_hook(self, root: Path, hook: str, *args: str, **env_updates: str) -> subprocess.CompletedProcess[str]:
        env = self._env(FAKE_ROOT=str(root), **env_updates)
        return subprocess.run(
            [str(self._hooks_dir(root) / hook), *args], cwd=root, env=env,
            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )

    def _calls(self) -> list[dict[str, object]]:
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_install_is_idempotent_worktree_local_and_refuses_custom_path(self) -> None:
        for root in (self.main, self.linked):
            result = self._install(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(self._hooks_dir(root), Path(self._git(root, "rev-parse", "--path-format=absolute", "--absolute-git-dir").stdout.strip()) / "wavepeek-hooks")
            self.assertEqual(self._install(root).returncode, 0)
        self.assertNotEqual(self._hooks_dir(self.main), self._hooks_dir(self.linked))

        self._git(self.linked, "config", "--worktree", "core.hooksPath", "/custom/hooks")
        result = self._install(self.linked)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refusing to replace", result.stderr)
        self.assertNotEqual(self._hooks_dir(self.main), Path("/custom/hooks"))

    def test_maps_linked_paths_filters_environment_and_uses_exact_index(self) -> None:
        self.assertEqual(self._install(self.linked).returncode, 0)
        self._replace_dev(self.linked)
        common = Path(self._git(self.linked, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.strip())
        index = common / "temporary index"
        git_dir = Path(self._git(self.linked, "rev-parse", "--path-format=absolute", "--absolute-git-dir").stdout.strip())
        index.write_bytes((git_dir / "index").read_bytes())
        (self.linked / "tracked").write_text("staged\n")
        self._git(self.linked, "add", "tracked", env=self._env(GIT_INDEX_FILE=str(index)))
        (self.linked / "tracked").write_text("working\n")

        result = self._run_hook(
            self.linked, "pre-commit", GIT_INDEX_FILE=str(index),
            GIT_OBJECT_DIRECTORY=str(common / "objects"), PROBE_INDEX="1",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        dev_call, call = self._calls()
        self.assertEqual(call["staged"], "staged\n")
        self.assertEqual(dev_call["dev_env"]["GIT_WORK_TREE"], "/workspaces/linked")
        self.assertEqual(dev_call["dev_env"]["GIT_DIR"], str(git_dir))
        self.assertEqual(dev_call["dev_env"]["GIT_COMMON_DIR"], str(common))
        self.assertEqual(dev_call["dev_env"]["GIT_INDEX_FILE"], str(index))
        self.assertNotIn("GIT_OBJECT_DIRECTORY", call["git_env"])

    def test_commit_message_path_and_status(self) -> None:
        self.assertEqual(self._install(self.main).returncode, 0)
        self._replace_dev(self.main)
        message = self.main / ".git/message with spaces"
        message.write_text("chore: valid\n")
        result = self._run_hook(self.main, "commit-msg", str(message), FAKE_STATUS="23")
        self.assertEqual(result.returncode, 23)
        self.assertEqual(
            self._calls()[0]["args"],
            ["pre-commit", "run", "--hook-stage", "commit-msg", "--commit-msg-filename", "/workspaces/main/.git/message with spaces"],
        )

        outside = self.tmp / "outside"
        outside.write_text("chore: valid\n")
        result = self._run_hook(self.main, "commit-msg", str(outside))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the mounted worktree", result.stderr)


if __name__ == "__main__":
    unittest.main()
