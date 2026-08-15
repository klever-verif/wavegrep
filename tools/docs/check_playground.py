#!/usr/bin/env python3

"""Run browser-level checks against a built WavePeek Playground site."""

import argparse
import functools
import http.server
import pathlib
import shlex
import subprocess
import threading
import urllib.parse

from playwright.sync_api import Page, sync_playwright


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


def run_browser(page: Page, command: str) -> tuple[int, str, str]:
    page.locator("#command-line").fill(command)
    page.locator("#run").click()
    page.locator("#exit-status").filter(has_text="Exit").wait_for(timeout=30_000)
    status = int(page.locator("#exit-status").text_content().removeprefix("Exit "))
    return (
        status,
        page.locator("#stdout").text_content(),
        page.locator("#stderr").text_content(),
    )


def run_native(binary: pathlib.Path, cwd: pathlib.Path, command: str) -> tuple[int, str, str]:
    completed = subprocess.run(
        [str(binary), *shlex.split(command)[1:]],
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
    base_url = f"http://127.0.0.1:{server.server_port}/"
    demo_dir = site / "assets" / "playground"

    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            page = browser.new_page(viewport={"width": 1440, "height": 1000})
            page.goto(base_url, wait_until="networkidle")
            page.locator("#source-status").filter(has_text="bundled demo").wait_for()

            assert page.locator(".md-version").count() == 0
            assert page.locator("a", has_text="Documentation").first.get_attribute("href") == (
                "https://kleverhq.github.io/wavepeek/latest/"
            )
            surfer_url = urllib.parse.urlparse(page.locator("#open-surfer").get_attribute("href"))
            assert surfer_url.netloc == "app.surfer-project.org"
            assert urllib.parse.parse_qs(surfer_url.query)["load_url"] == [
                f"{base_url}assets/playground/scr1_axi.fst"
            ]

            commands = [
                "wavepeek info --waves scr1_axi.fst",
                "wavepeek info --waves scr1_axi.fst --json",
                "wavepeek info --waves scr1_axi.fst --jsonl",
                (
                    "wavepeek extract axi --waves scr1_axi.fst "
                    "--scope TOP.scr1_top_tb_axi.i_top --include '^io_axi_dmem_' "
                    "--map aclk=clk --map aresetn=axi_rst_n "
                    "--from 1ps --to 1880182ps --max 3"
                ),
            ]
            for command in commands:
                assert run_browser(page, command) == run_native(native_bin, demo_dir, command)

            unsupported = run_browser(
                page,
                "wavepeek extract axi --waves scr1_axi.fst --source config.json",
            )
            assert unsupported[0] == 1
            assert "--source is not supported in the browser" in unsupported[2]

            custom = "wavepeek signal --waves scr1_axi.fst --scope TOP --abs"
            page.locator("#command-line").fill(custom)
            page.locator("#max").fill("7")
            page.locator("#max").blur()
            synchronized = page.locator("#command-line").input_value()
            assert "--abs" in synchronized and "--max 7" in synchronized
            tokenized = page.evaluate(
                "async () => (await import('/assets/playground/playground.js')).tokenize("
                '"wavepeek property --eval \\\"a && b\\\"")'
            )
            assert tokenized[-1] == "a && b"

            requests: list[str] = []
            page.on("request", lambda request: requests.append(request.url))
            page.locator("#local-file").set_input_files(
                {"name": "local.vcd", "mimeType": "text/plain", "buffer": VCD}
            )
            page.locator("#source-status").filter(has_text="local file").wait_for()
            assert page.locator("#open-surfer").is_hidden()
            requests.clear()
            local = run_browser(page, "wavepeek info --waves local.vcd")
            assert local[0] == 0 and "time_unit: 1ns" in local[1]
            assert not [url for url in requests if not url.startswith(base_url)]

            page.reload(wait_until="networkidle")
            page.locator("#source-status").filter(has_text="bundled demo").wait_for()
            assert page.locator("#history li").count() == 1
            assert "Commands run in this tab" in page.locator("#history").text_content()

            page.locator("#command-line").fill(commands[-1].replace("--max 3", "--max unlimited"))
            page.locator("#run").click()
            page.locator("#stop").click()
            assert page.locator("#exit-status").text_content() == "Stopped"
            recovered = run_browser(page, commands[0])
            assert recovered[0] == 0

            workspace = page.locator(".playground__workspace").bounding_box()
            controls = page.locator(".playground__controls").bounding_box()
            result = page.locator(".playground__result").bounding_box()
            assert workspace and controls and result and controls["x"] < result["x"]

            page.set_viewport_size({"width": 390, "height": 844})
            page.wait_for_timeout(100)
            overflow = page.evaluate(
                """() => ({
                    viewport: window.innerWidth,
                    document: document.documentElement.scrollWidth,
                    widest: [...document.querySelectorAll('*')]
                      .map((element) => ({tag: element.tagName, id: element.id, class: element.className, width: element.scrollWidth, box: element.getBoundingClientRect().width, min: getComputedStyle(element).minWidth, cssWidth: getComputedStyle(element).width, grid: getComputedStyle(element).gridTemplateColumns}))
                      .sort((left, right) => right.width - left.width)
                      .slice(0, 20),
                })"""
            )
            assert overflow["document"] <= overflow["viewport"], overflow
            controls = page.locator(".playground__controls").bounding_box()
            result = page.locator(".playground__result").bounding_box()
            assert controls and result and result["y"] > controls["y"]
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
