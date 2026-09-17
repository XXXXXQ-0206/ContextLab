"""Black-box browser smoke for the preview-mode ContextLab Web workspace."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from playwright.sync_api import sync_playwright


VIEWPORTS = (
    ("desktop", 1440, 1100),
    ("mobile", 390, 844),
)


def run_smoke(base_url: str) -> list[dict[str, object]]:
    receipts: list[dict[str, object]] = []

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        try:
            for viewport_name, width, height in VIEWPORTS:
                console_errors: list[str] = []
                page_errors: list[str] = []
                response_failures: list[str] = []
                expected_unavailable_probe = [False]
                page = browser.new_page(viewport={"width": width, "height": height})
                page.on(
                    "console",
                    lambda message: (
                        console_errors.append(message.text)
                        if message.type == "error"
                        and not (
                            expected_unavailable_probe[0]
                            and "status of 503" in message.text
                        )
                        else None
                    ),
                )
                page.on("pageerror", lambda error: page_errors.append(str(error)))
                page.on(
                    "response",
                    lambda response: (
                        response_failures.append(
                            f"{response.status} {response.request.method} {response.url}"
                        )
                        if response.status >= 400
                        and not (
                            viewport_name == "desktop"
                            and response.status == 503
                            and "/workflow-bindings" in response.url
                        )
                        else None
                    ),
                )

                page.goto(f"{base_url}/", wait_until="domcontentloaded")
                page.wait_for_load_state("networkidle")
                page.wait_for_selector("main.app-shell")

                required_selectors = {
                    "workspace": "#workspace",
                    "context_graph": "#graph",
                    "operations": "#operations-heading",
                    "context_bindings_token": (
                        'input[name="workflow-context-bindings-bearer-token"]'
                    ),
                    "context_bindings_button": (
                        'button:has-text("Inspect bindings")'
                    ),
                }
                missing_selectors = [
                    name
                    for name, selector in required_selectors.items()
                    if page.locator(selector).count() == 0
                ]
                geometry = page.evaluate(
                    """() => ({
                      innerWidth: window.innerWidth,
                      scrollWidth: document.documentElement.scrollWidth,
                      bodyScrollWidth: document.body.scrollWidth
                    })"""
                )
                private_read_probe = None
                if viewport_name == "desktop":
                    page.locator(
                        'input[name="workflow-context-bindings-bearer-token"]'
                    ).fill("browser-smoke-token")
                    expected_unavailable_probe[0] = True
                    with page.expect_response(
                        lambda response: "/workflow-bindings" in response.url
                    ) as response_info:
                        page.locator('button:has-text("Inspect bindings")').click()
                    probe_response = response_info.value
                    page.wait_for_timeout(250)
                    private_read_probe = {
                        "status": probe_response.status,
                        "body": probe_response.json(),
                        "notice_count": page.locator(
                            "p.context-benchmark-evidence__notice"
                        ).count(),
                    }
                screenshot_path = (
                    Path.home()
                    / "AppData"
                    / "Local"
                    / "Temp"
                    / f"contextlab-browser-smoke-{viewport_name}.png"
                )
                page.screenshot(path=str(screenshot_path), full_page=True)
                receipts.append(
                    {
                        "viewport": viewport_name,
                        "size": f"{width}x{height}",
                        "title": page.title(),
                        "button_count": page.locator("button").count(),
                        "input_count": page.locator("input").count(),
                        "required_selectors_missing": missing_selectors,
                        "console_errors": console_errors,
                        "page_errors": page_errors,
                        "response_failures": response_failures,
                        "private_read_probe": private_read_probe,
                        "geometry": geometry,
                        "horizontal_overflow": (
                            geometry["scrollWidth"] > geometry["innerWidth"]
                            or geometry["bodyScrollWidth"] > geometry["innerWidth"]
                        ),
                        "screenshot": str(screenshot_path),
                    }
                )
                page.close()
        finally:
            browser.close()

    return receipts


def main() -> int:
    base_url = sys.argv[1] if len(sys.argv) == 2 else "http://127.0.0.1:3000"
    receipts = run_smoke(base_url.rstrip("/"))
    print(json.dumps(receipts, ensure_ascii=True, indent=2))
    failures = [
        receipt
        for receipt in receipts
        if receipt["required_selectors_missing"]
        or receipt["console_errors"]
        or receipt["page_errors"]
        or receipt["response_failures"]
        or (
            receipt["viewport"] == "desktop"
            and (
                receipt["private_read_probe"] is None
                or receipt["private_read_probe"]["status"] != 503
                or receipt["private_read_probe"]["notice_count"] != 1
                or "private" in str(receipt["private_read_probe"]["body"]).lower()
            )
        )
        or receipt["horizontal_overflow"]
    ]
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
