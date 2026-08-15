#!/usr/bin/env python3

"""Run browser-level checks against a composed WavePeek Pages preview."""

import argparse
import functools
import http.server
import pathlib
import shlex
import subprocess
import threading
import urllib.parse

from playwright.sync_api import Locator, Page, sync_playwright


VCD = b"""$date today $end
$version browser-test $end
$timescale 1ns $end
$scope module top $end
$var wire 1 ! clk $end
$upscope $end
$enddefinitions $end
#0
0!
#5
1!
"""


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, _format: str, *_args: object) -> None:
        pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--site", required=True, type=pathlib.Path)
    parser.add_argument("--native-bin", required=True, type=pathlib.Path)
    return parser.parse_args()


def run_browser(page: Page, command: str) -> tuple[int, str, str, Locator]:
    entries = page.locator("#transcript .playground__entry")
    before = entries.count()
    page.locator("#command-line").fill(command)
    page.locator("#run").click()
    entry = entries.nth(before)
    entry.locator(".playground__duration").filter(has_not_text="Running").wait_for(
        timeout=30_000
    )
    return (
        int(entry.get_attribute("data-exit-status")),
        entry.locator(".playground__stdout").text_content(),
        entry.locator(".playground__stderr").text_content(),
        entry,
    )


def run_native(binary: pathlib.Path, cwd: pathlib.Path, command: str) -> tuple[int, str, str]:
    completed = subprocess.run(
        [str(binary), *shlex.split(command)],
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
    )
    return completed.returncode, completed.stdout, completed.stderr


def check(site: pathlib.Path, native_bin: pathlib.Path) -> None:
    handler = functools.partial(QuietHandler, directory=site)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    base_url = f"{origin}/wavepeek/"
    demo_dir = site / "wavepeek" / "assets" / "playground"

    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            page = browser.new_page(viewport={"width": 1440, "height": 1000})
            page.goto(base_url, wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()

            assert page.locator(".md-version").count() == 0
            documentation = page.locator("a", has_text="Documentation").first
            assert documentation.get_attribute("href") == "/wavepeek/latest/"
            documentation.click()
            page.wait_for_url(f"{origin}/wavepeek/latest/**")
            assert page.locator("main").text_content().strip()
            playground = page.locator("a", has_text="Playground").first
            assert playground.get_attribute("href") == "/wavepeek/"
            playground.click()
            page.wait_for_url(base_url)
            page.locator("#source-status").filter(has_text="Ready").wait_for()

            assert page.locator("#source-name").text_content() == "scr1_axi.fst"
            assert page.locator("#source-format").text_content() == "FST"
            assert "MiB" in page.locator("#source-size").text_content()
            assert page.locator("#source-indicator").get_attribute("data-status") == "ready"
            surfer_url = urllib.parse.urlparse(
                page.locator("#open-surfer").get_attribute("href")
            )
            assert surfer_url.netloc == "app.surfer-project.org"
            assert urllib.parse.parse_qs(surfer_url.query)["load_url"] == [
                f"{base_url}assets/playground/scr1_axi.fst"
            ]

            prefix = page.locator(".playground__command-line label").text_content()
            assert "wavepeek" in prefix and page.locator("#command-line").input_value().startswith(
                "info "
            )
            assert page.locator(".playground__commands button").all_text_contents() == [
                "Info", "Scope", "Signal", "Value", "Change", "Property", "Extract AXI"
            ]
            initial_entries = page.locator("#transcript .playground__entry").count()
            page.locator('[data-example="scope"]').click()
            assert page.locator("#command-line").input_value().startswith("scope ")
            assert page.locator("#transcript .playground__entry").count() == initial_entries
            page.locator('[data-suggestion="property"]').click()
            assert page.locator("#command-line").input_value().startswith("property ")
            page.locator("#toggle-suggestions").click()
            assert page.locator("#more-suggestions").is_visible()
            page.locator('[data-suggestion="generic"]').click()
            assert page.locator("#command-line").input_value().startswith("extract generic ")

            page.locator('input[name="output-mode"][value="json"]').check()
            assert page.locator("#command-line").input_value().endswith("--json")
            page.locator('input[name="output-mode"][value="human"]').check()
            assert "--json" not in page.locator("#command-line").input_value()

            commands = [
                "info --waves scr1_axi.fst",
                "info --waves scr1_axi.fst --json",
                "info --waves scr1_axi.fst --jsonl",
                (
                    "extract axi --waves scr1_axi.fst "
                    "--scope TOP.scr1_top_tb_axi.i_top --include '^io_axi_dmem_' "
                    "--map aclk=clk --map aresetn=axi_rst_n "
                    "--from 1ps --to 1880182ps --max 3"
                ),
            ]
            for command in commands:
                status, stdout, stderr, entry = run_browser(page, command)
                assert (status, stdout, stderr) == run_native(native_bin, demo_dir, command)
                assert entry.get_attribute("data-status") == "ok"
                assert "Exit" not in entry.text_content()

            status, _, stderr, failed = run_browser(
                page,
                "extract axi --waves scr1_axi.fst --source config.json",
            )
            assert status == 1
            assert "--source is not supported in the browser" in stderr
            assert failed.get_attribute("data-status") == "error"
            assert failed.locator(".playground__stderr").is_visible()
            scroll = page.locator("#transcript").evaluate(
                "element => ({height: element.clientHeight, content: element.scrollHeight})"
            )
            assert scroll["content"] > scroll["height"]

            page.locator("#command-line").fill("info --waves scr1_axi.fst")
            page.locator("#clear").click()
            assert page.locator("#transcript .playground__entry").count() == 0
            assert page.locator("#command-line").input_value() == "info --waves scr1_axi.fst"
            page.locator("#command-line").press("ArrowUp")
            assert "--source config.json" in page.locator("#command-line").input_value()
            page.locator("#command-line").press("Control+k")
            assert page.locator("#transcript .playground__entry").count() == 0

            tokenized = page.evaluate(
                "async () => (await import('/wavepeek/assets/playground/playground.js')).tokenize("
                '"property --eval \\"a && b\\"")'
            )
            assert tokenized[-1] == "a && b"

            requests: list[str] = []
            page.on("request", lambda request: requests.append(request.url))
            page.locator("#local-file").set_input_files(
                {"name": "local.vcd", "mimeType": "text/plain", "buffer": VCD}
            )
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            assert page.locator("#source-name").text_content() == "local.vcd"
            assert page.locator("#source-format").text_content() == "VCD"
            assert "KiB" in page.locator("#source-size").text_content()
            assert page.locator("#open-surfer").is_hidden()
            requests.clear()
            status, stdout, _, _ = run_browser(page, "info --waves local.vcd")
            assert status == 0 and "time_unit: 1ns" in stdout
            assert not [url for url in requests if not url.startswith(origin)]

            page.reload(wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            assert page.locator("#transcript .playground__entry").count() == 0

            page.locator("#command-line").fill(commands[-1].replace("--max 3", "--max unlimited"))
            page.locator("#run").click()
            page.locator("#stop").click()
            assert page.locator("#transcript .playground__entry").last.get_attribute(
                "data-status"
            ) == "error"
            recovered = run_browser(page, commands[0])
            assert recovered[0] == 0

            terminal = page.locator(".playground__terminal").bounding_box()
            sidebar = page.locator(".playground__sidebar").bounding_box()
            assert terminal and sidebar and terminal["x"] < sidebar["x"]
            assert terminal["height"] >= 600

            colors = page.evaluate(
                """() => {
                    const root = document.querySelector('.playground');
                    document.body.setAttribute('data-md-color-scheme', 'default');
                    const light = getComputedStyle(root).getPropertyValue('--terminal-bg');
                    document.body.setAttribute('data-md-color-scheme', 'slate');
                    const dark = getComputedStyle(root).getPropertyValue('--terminal-bg');
                    return {light, dark};
                }"""
            )
            assert colors["light"] != colors["dark"]

            page.set_viewport_size({"width": 390, "height": 844})
            page.wait_for_timeout(100)
            overflow = page.evaluate(
                "() => ({viewport: innerWidth, document: document.documentElement.scrollWidth})"
            )
            assert overflow["document"] <= overflow["viewport"], overflow
            terminal = page.locator(".playground__terminal").bounding_box()
            sidebar = page.locator(".playground__sidebar").bounding_box()
            assert terminal and sidebar and sidebar["y"] > terminal["y"]
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def main() -> int:
    args = parse_args()
    check(args.site.resolve(), args.native_bin.resolve())
    print("Playground browser checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
