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
    assert entries.count() == before + 1
    entry = entries.first
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
            context = browser.new_context(viewport={"width": 1440, "height": 1000})
            context.grant_permissions(
                ["clipboard-read", "clipboard-write"], origin=origin
            )
            page = context.new_page()
            page.goto(base_url, wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()

            assert page.locator(".md-version").count() == 0
            heading = page.locator(".playground__visually-hidden").bounding_box()
            assert heading and heading["width"] <= 1 and heading["height"] <= 1
            prompt = page.locator("#agent-prompt").get_attribute("data-copy")
            assert page.locator(".playground__install > a").text_content() == (
                "Installation instructions on GitHub Releases ↗"
            )
            assert prompt == (
                "Install the latest release from "
                "https://github.com/kleverhq/wavepeek/releases. "
                "Run 'wavepeek skill' to get the skill."
            )
            assert page.locator("#agent-prompt").evaluate(
                "element => element.scrollWidth <= element.clientWidth"
            )
            install_label = page.locator(".playground__install > a").bounding_box()
            install_prompt = page.locator(".playground__install-prompt").bounding_box()
            assert install_label and install_prompt
            assert abs(
                install_label["y"] + install_label["height"] / 2
                - install_prompt["y"]
                - install_prompt["height"] / 2
            ) < 10
            page.locator("#copy-agent-prompt").click()
            page.locator("#copy-agent-prompt").filter(has_text="Copied").wait_for()
            assert page.locator("#copy-status").text_content() == "Copied"
            assert page.evaluate("navigator.clipboard.readText()") == prompt
            page.evaluate(
                "() => { navigator.clipboard.writeText = () => Promise.reject(new Error('denied')); }"
            )
            page.locator("#copy-agent-prompt").click()
            page.locator("#copy-agent-prompt").filter(has_text="Press Ctrl+C").wait_for()
            assert page.locator("#copy-status").text_content() == "Press Ctrl+C"
            assert "wavepeek" in page.evaluate("getSelection().toString()")
            tagline = page.locator(
                ".md-header__topic:first-child .md-ellipsis"
            ).evaluate("element => getComputedStyle(element, '::after').content")
            assert "deterministic RTL waveform inspection" in tagline
            assert page.locator(".md-search").is_hidden()

            page.evaluate("localStorage.clear()")
            page.reload(wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            for _ in range(2):
                page.locator('[data-md-component="palette"] label:visible').click()
            assert page.locator("body").get_attribute("data-md-color-scheme") == "slate"

            documentation = page.locator("a", has_text="Documentation").first
            assert documentation.get_attribute("href") == "/wavepeek/latest/"
            documentation.click()
            page.wait_for_url(f"{origin}/wavepeek/latest/**")
            assert page.locator("main").text_content().strip()
            assert page.locator("body").get_attribute("data-md-color-scheme") == "slate"
            assert page.locator(".md-search").is_visible()
            page.wait_for_timeout(300)
            documentation_colors = page.evaluate(
                """() => {
                    const style = (selector) => getComputedStyle(document.querySelector(selector));
                    return {
                        canvas: style('body').backgroundColor,
                        text: style('.md-content').color,
                        codeBackground: style('.md-content code').backgroundColor,
                        code: style('.md-content code').color,
                        link: style('.md-content li a').color,
                        footer: style('.md-footer-meta').backgroundColor,
                    };
                }"""
            )
            assert documentation_colors == {
                "canvas": "rgb(16, 17, 20)",
                "text": "rgb(236, 238, 242)",
                "codeBackground": "rgb(30, 33, 38)",
                "code": "rgb(220, 225, 232)",
                "link": "rgb(210, 215, 223)",
                "footer": "rgb(16, 17, 20)",
            }, documentation_colors
            playground = page.locator("a", has_text="Playground").first
            assert playground.get_attribute("href") == "/wavepeek/"
            playground.click()
            page.wait_for_url(base_url)
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            assert page.locator("body").get_attribute("data-md-color-scheme") == "slate"
            assert page.locator(".md-search").is_hidden()

            assert page.locator("#source-name").text_content() == "scr1_axi.fst"
            assert page.locator("#open-local").evaluate("element => element.tagName") == "BUTTON"
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
            assert page.locator("#open-surfer").text_content() == "Open visually in Surfer ↗"
            privacy = page.locator(".playground__source-privacy")
            assert privacy.text_content() == "Waveform data never leaves your browser."
            assert privacy.evaluate("element => getComputedStyle(element).borderTopStyle") == "none"
            assert privacy.evaluate("element => getComputedStyle(element).textAlign") == "left"
            local_source = page.locator(".playground__local-source").bounding_box()
            privacy_box = privacy.bounding_box()
            source_meta = page.locator(".playground__source-meta").bounding_box()
            surfer = page.locator("#open-surfer").bounding_box()
            assert local_source and privacy_box and source_meta and surfer
            assert privacy_box["x"] >= local_source["x"] + local_source["width"]
            assert source_meta["y"] > local_source["y"]
            assert abs(source_meta["y"] - surfer["y"]) < 10
            assert surfer["x"] > source_meta["x"]

            initial_help = page.locator(
                '#transcript .playground__entry[data-status="ok"]'
            ).first
            initial_help.wait_for()
            prefix = page.locator(".playground__command-line label").text_content()
            assert "wavepeek" in prefix
            assert page.locator("#terminal-shortcuts").text_content() == (
                "Enter to run · ↑/↓ for command history"
            )
            assert page.locator("#command-line").input_value() == "help"
            assert initial_help.locator("code").text_content() == "$ wavepeek help"
            assert "Usage: wavepeek" in initial_help.locator(".playground__stdout").text_content()
            assert page.locator(".playground__commands button").all_text_contents() == [
                "Info", "Scope", "Signal", "Value", "Change", "Property", "Extract", "Help"
            ]
            assert page.locator(".playground__command-separator").count() == 2
            assert page.locator('[data-example="help"]').get_attribute("aria-pressed") == "true"
            assert page.locator('[data-example="info"]').get_attribute("aria-pressed") == "false"
            initial_entries = page.locator("#transcript .playground__entry").count()
            assert initial_entries == 1
            page.locator('[data-example="scope"]').click()
            assert page.locator("#command-line").input_value() == "scope --help"
            assert page.locator('[data-example="scope"]').get_attribute("aria-pressed") == "true"
            assert page.locator("#transcript .playground__entry").count() == initial_entries
            page.locator('[data-suggestion="property"]').click()
            assert page.locator("#command-line").input_value().startswith("property ")
            assert page.locator('[data-example="property"]').get_attribute("aria-pressed") == "true"
            assert page.locator("#toggle-suggestions").count() == 0
            assert page.locator("[data-suggestion]").count() == 8
            assert all(
                button.is_visible()
                for button in page.locator("[data-suggestion]").all()
            )
            page.locator('[data-suggestion="generic"]').click()
            assert page.locator("#command-line").input_value().startswith("extract generic ")
            assert page.locator('[data-example="extract"]').get_attribute("aria-pressed") == "true"
            assert page.locator("#suggestions-heading").text_content() == "Demo queries"
            assert page.locator(".playground__commands-more").count() == 0
            assert page.locator("#output-description").count() == 0
            assert page.locator(".playground__shortcuts").count() == 0
            assert page.locator(".playground__sidebar .playground__modes").count() == 0
            assert page.locator(".playground__sidebar > section").count() == 1
            controls_box = page.locator(".playground__command-controls").bounding_box()
            commands_box = page.locator(".playground__commands").bounding_box()
            modes_box = page.locator(".playground__modes").bounding_box()
            source_box = page.locator(".playground__source").bounding_box()
            command_button = page.locator('[data-example="info"]').bounding_box()
            mode_button = page.locator('.playground__modes span').first.bounding_box()
            assert controls_box and commands_box and modes_box and source_box
            assert command_button and mode_button
            assert modes_box["y"] > commands_box["y"]
            assert source_box["x"] > controls_box["x"]
            assert abs(source_box["height"] - controls_box["height"]) < 1
            assert abs(command_button["height"] - mode_button["height"]) < 1
            assert page.locator(".playground__command-controls").evaluate(
                "element => getComputedStyle(element).borderTopStyle"
            ) == "solid"
            page.locator('[data-example="help"]').click()
            assert page.locator("#command-line").input_value() == "help"
            assert page.locator('[data-example="help"]').get_attribute("aria-pressed") == "true"
            assert page.locator("#transcript .playground__entry").count() == initial_entries

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
                assert entry.locator(".playground__duration").get_attribute(
                    "aria-label"
                ).startswith("Succeeded in ")
                assert entry.locator("code").text_content().startswith(
                    f"$ wavepeek {command.split()[0]}"
                )
                assert page.locator("#transcript").evaluate("element => element.scrollTop") == 0
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
            assert page.locator("#open-local").get_attribute("aria-pressed") == "true"
            assert page.locator("#use-demo").get_attribute("aria-pressed") == "false"
            assert "KiB" in page.locator("#source-size").text_content()
            assert page.locator("#open-surfer").is_hidden()
            requests.clear()
            status, stdout, _, _ = run_browser(page, "info --waves local.vcd")
            assert status == 0 and "time_unit: 1ns" in stdout
            assert not [url for url in requests if not url.startswith(origin)]

            page.reload(wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            page.locator('#transcript .playground__entry[data-status="ok"]').wait_for()
            assert page.locator("#transcript .playground__entry").count() == 1

            long_command = commands[-1].replace("--max 3", "--max unlimited")
            command_line = page.locator("#command-line")
            command_header = page.locator(".playground__command-line")
            prompt_before = page.locator(".playground__command-line label").bounding_box()
            run_before = page.locator("#run").bounding_box()
            command_line.fill(" ".join([long_command] * 3))
            focused_line = command_line.bounding_box()
            focused_header = command_header.bounding_box()
            prompt_focused = page.locator(".playground__command-line label").bounding_box()
            run_focused = page.locator("#run").bounding_box()
            assert focused_line and focused_header
            assert prompt_before and run_before and prompt_focused and run_focused
            assert prompt_focused["y"] == prompt_before["y"]
            assert run_focused["y"] == run_before["y"]
            assert focused_line["height"] > 32
            assert focused_line["y"] + focused_line["height"] > (
                focused_header["y"] + focused_header["height"]
            )
            assert command_line.evaluate(
                "element => getComputedStyle(element).whiteSpace"
            ) == "pre-wrap"
            assert command_line.evaluate(
                "element => getComputedStyle(element).boxShadow"
            ) != "none"
            assert command_line.evaluate(
                "element => getComputedStyle(element).overflowY"
            ) == "hidden"
            assert command_line.evaluate(
                "element => element.scrollHeight <= element.clientHeight + 1"
            )
            command_line.blur()
            collapsed_line = command_line.bounding_box()
            collapsed_header = command_header.bounding_box()
            assert collapsed_line and collapsed_header
            assert collapsed_line["height"] < focused_line["height"]
            assert collapsed_line["height"] <= collapsed_header["height"]
            assert collapsed_header["height"] == focused_header["height"]
            assert command_line.evaluate(
                "element => getComputedStyle(element).whiteSpace"
            ) == "nowrap"
            command_line.fill(long_command)
            command_line.blur()
            entries = page.locator("#transcript .playground__entry")
            before = entries.count()
            page.locator("#run").click()
            assert page.locator("#run").text_content() == "Stop"
            assert page.locator("#stop").count() == 0
            page.locator("#command-line").press("Enter")
            assert entries.count() == before + 1
            page.locator("#run").click()
            assert page.locator("#run").text_content() == "Run"
            assert entries.first.get_attribute("data-status") == "error"

            page.locator("#command-line").fill(long_command)
            page.locator("#run").click()
            page.locator("#clear").click()
            page.locator("#run").wait_for(state="visible")
            page.wait_for_function("document.querySelector('#run').textContent === 'Run'")
            assert entries.count() == 0
            assert page.locator("#command-line").input_value() == long_command

            page.locator("#run").click()
            page.locator("#local-file").set_input_files(
                {"name": "replacement.vcd", "mimeType": "text/plain", "buffer": VCD}
            )
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            assert page.locator('[data-status="running"]').count() == 0

            page.locator("#use-demo").click()
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            assert page.locator("#use-demo").get_attribute("aria-pressed") == "true"
            assert page.locator("#open-local").get_attribute("aria-pressed") == "false"
            recovered = run_browser(page, commands[0])
            assert recovered[0] == 0

            terminal = page.locator(".playground__terminal").bounding_box()
            sidebar = page.locator(".playground__sidebar").bounding_box()
            assert terminal and sidebar and 0 < terminal["x"] < sidebar["x"]
            assert terminal["height"] >= 600
            assert sidebar["height"] < terminal["height"]
            vertical = page.evaluate(
                "() => ({viewport: innerHeight, document: document.documentElement.scrollHeight})"
            )
            assert vertical["document"] <= vertical["viewport"], vertical

            colors = page.evaluate(
                """async () => {
                    const style = (selector) => getComputedStyle(document.querySelector(selector));
                    const settle = () => new Promise((resolve) => setTimeout(resolve, 300));
                    document.body.setAttribute('data-md-color-scheme', 'default');
                    await settle();
                    const light = {
                        terminal: style('.playground__terminal').backgroundColor,
                        selectedCommand: style('.playground__commands [aria-pressed="true"]').backgroundColor,
                        selectedDemo: style('#use-demo').backgroundColor,
                        selectedMode: style('.playground__modes input:checked + span').backgroundColor,
                        run: style('#run').backgroundColor,
                        clear: style('#clear').backgroundColor,
                        link: style('#open-surfer').color,
                        ready: style('#source-indicator').backgroundColor,
                    };
                    document.body.setAttribute('data-md-color-scheme', 'slate');
                    await settle();
                    return {
                        light,
                        canvas: style('.playground').backgroundColor,
                        panel: style('.playground__command-controls').backgroundColor,
                        query: style('.playground__suggestions button').backgroundColor,
                        terminal: style('.playground__terminal').backgroundColor,
                        commandBar: style('.playground__command-line').backgroundColor,
                        selectedCommand: style('.playground__commands [aria-pressed="true"]').backgroundColor,
                        selectedDemo: style('#use-demo').backgroundColor,
                        selectedMode: style('.playground__modes input:checked + span').backgroundColor,
                        selectedForeground: style('.playground__commands [aria-pressed="true"]').color,
                        run: style('#run').backgroundColor,
                        clear: style('#clear').backgroundColor,
                        link: style('#open-surfer').color,
                        localBorder: style('#open-local').borderColor,
                        ready: style('#source-indicator').backgroundColor,
                        error: style('.playground').getPropertyValue('--terminal-error').trim(),
                    };
                }"""
            )
            assert colors["light"] == {
                "terminal": "rgb(246, 248, 250)",
                "selectedCommand": "rgb(20, 21, 26)",
                "selectedDemo": "rgb(33, 33, 33)",
                "selectedMode": "rgb(20, 21, 26)",
                "run": "rgb(20, 21, 26)",
                "clear": "rgb(255, 255, 255)",
                "link": "rgb(33, 33, 33)",
                "ready": "rgb(35, 122, 59)",
            }, colors["light"]
            assert colors["canvas"] == "rgb(16, 17, 20)"
            assert colors["panel"] == "rgb(23, 25, 29)"
            assert colors["query"] == "rgb(30, 33, 38)"
            assert colors["terminal"] == "rgb(12, 15, 19)"
            assert colors["commandBar"] == "rgb(19, 23, 28)"
            for key in ("selectedCommand", "selectedDemo", "selectedMode", "run"):
                assert colors[key] == "rgb(236, 238, 242)", colors
            assert colors["selectedForeground"] == "rgb(17, 19, 23)"
            assert colors["clear"] == "rgb(35, 38, 44)"
            assert colors["link"] == "rgb(210, 215, 223)"
            assert colors["localBorder"] == "rgba(0, 0, 0, 0)"
            assert colors["ready"] == "rgb(99, 201, 137)"
            assert colors["error"] == "#ef6b73"
            page.keyboard.press("Tab")
            page.locator('.playground__commands [aria-pressed="true"]').focus()
            page.wait_for_timeout(100)
            focus = page.locator('.playground__commands [aria-pressed="true"]').evaluate(
                "element => ({ color: getComputedStyle(element).outlineColor, "
                "style: getComputedStyle(element).outlineStyle })"
            )
            assert focus == {
                "color": "rgba(236, 238, 242, 0.75)",
                "style": "solid",
            }, focus

            page.set_viewport_size({"width": 390, "height": 844})
            page.wait_for_timeout(100)
            overflow = page.evaluate(
                "() => ({viewport: innerWidth, document: document.documentElement.scrollWidth})"
            )
            assert overflow["document"] <= overflow["viewport"], overflow
            install = page.locator(".playground__install").bounding_box()
            header = page.locator(".md-header").bounding_box()
            source = page.locator(".playground__source").bounding_box()
            controls = page.locator(".playground__command-controls").bounding_box()
            terminal = page.locator(".playground__terminal").bounding_box()
            sidebar = page.locator(".playground__sidebar").bounding_box()
            demo = page.locator("#use-demo").bounding_box()
            local = page.locator("#open-local").bounding_box()
            commands_row = page.locator(".playground__commands").bounding_box()
            command_button = page.locator(".playground__commands button").first.bounding_box()
            assert install and header and source and controls and terminal and sidebar
            assert demo and local and commands_row and command_button
            assert install["y"] - header["y"] - header["height"] < 15
            assert source["y"] < controls["y"] < terminal["y"] < sidebar["y"]
            assert abs(demo["y"] - local["y"]) < 5
            assert commands_row["height"] > command_button["height"] * 1.5
            assert page.locator(".playground__commands").evaluate(
                "element => element.scrollWidth <= element.clientWidth"
            )
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
