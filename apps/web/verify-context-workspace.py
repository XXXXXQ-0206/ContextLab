from pathlib import Path
from playwright.sync_api import sync_playwright


APP_DIR = Path("apps/web/src/app")


def inspect_source_boundaries():
    page_source = (APP_DIR / "page.tsx").read_text(encoding="utf-8")
    data_source = (APP_DIR / "context-workspace-data.ts").read_text(encoding="utf-8")
    presenter_source = (APP_DIR / "context-workspace-presenter.ts").read_text(encoding="utf-8")
    screen_source = (APP_DIR / "context-workspace-screen.tsx").read_text(encoding="utf-8")
    graph_inspector_source = (APP_DIR / "context-graph-inspector.tsx").read_text(encoding="utf-8")
    graph_review_source = (APP_DIR / "context-graph-review.tsx").read_text(encoding="utf-8")
    lifecycle_data_source = (APP_DIR / "context-lifecycle-data.ts").read_text(encoding="utf-8")
    lifecycle_editor_source = (APP_DIR / "context-lifecycle-editor.tsx").read_text(encoding="utf-8")
    lifecycle_proxy_source = (APP_DIR / "context-lifecycle-proxy.ts").read_text(encoding="utf-8")
    benchmark_workspace_data_source = (APP_DIR / "local-benchmark-workspace-data.ts").read_text(encoding="utf-8")
    benchmark_workspace_presenter_source = (APP_DIR / "local-benchmark-workspace-presenter.ts").read_text(encoding="utf-8")
    benchmark_workspace_inspector_source = (APP_DIR / "local-benchmark-workspace-inspector.tsx").read_text(encoding="utf-8")

    assert "dangerouslySetInnerHTML" not in page_source
    assert "ContextLabClient" not in page_source
    assert "loadContextWorkspace" not in screen_source
    assert "@contextlab/ui" not in presenter_source
    assert "ContextLabClient" not in presenter_source
    assert "ContextLabClient" not in screen_source
    assert "ReactNode" not in presenter_source
    assert "const graphEdges" not in screen_source
    assert "answer only from retrieved snippets" not in screen_source
    assert "semanticDiffPreview" not in screen_source
    assert "semanticDiffPreview" not in presenter_source
    assert "CommitGraphReview" in screen_source
    assert '"use client"' in graph_review_source
    assert "presentCommitGraphDiff" in graph_review_source
    assert "/api/contexts/" in graph_review_source
    assert "presentContextWorkspaceScreen" in page_source
    assert page_source.count("getEvaluationRun") == 0
    assert data_source.count("getEvaluationRun") == 1
    assert data_source.count("getWorkspaceContextGraph") == 1
    assert "selectedEvaluationRun" in data_source
    assert "workspaceContextGraph" in presenter_source
    assert '"use client"' in graph_inspector_source
    assert "presentGraphNodeInspector" in graph_inspector_source
    assert "ContextGraphInspector" in screen_source
    assert ".filter((relationship)" not in screen_source
    assert "ContextLifecycleEditor" in screen_source
    assert "@contextlab/local-sdk" not in screen_source
    assert 'credentials: "omit"' in lifecycle_data_source
    assert "ContextLabLocalClient" not in lifecycle_editor_source
    assert "ContextLabLocalClient" in lifecycle_proxy_source
    assert "@contextlab/local-sdk" in benchmark_workspace_data_source
    assert "ContextLabLocalClient" not in benchmark_workspace_presenter_source
    assert "parseLocalBenchmarkWorkspace" not in benchmark_workspace_presenter_source
    assert "loadLocalBenchmarkWorkspace" in benchmark_workspace_inspector_source
    assert "evaluate_runs" not in benchmark_workspace_presenter_source


def inspect_viewport(page, width, height, name):
    page.set_viewport_size({"width": width, "height": height})
    page.goto("http://localhost:3000", wait_until="networkidle", timeout=60000)
    page.get_by_role("heading", name="Support Resolution Agent").wait_for(timeout=15000)

    for text in [
        "Support Resolution Agent",
        "No API base URL configured",
        "Context Detail",
        "Preview data",
        "Preview fallback",
        "Live API",
        "Needs URL",
        "Recovery path",
        "Commit History",
        "Component Inventory",
        "Component Detail",
        "Evaluation Runs",
        "Evaluation Run Detail",
        "Evaluation Scorecard",
        "Local Benchmark Workspace",
        "No server projection loaded",
        "Commit Graph Review",
        "Preview fixture",
        "Added Nodes",
        "memory:timeline",
        "Selected Node / 已选节点",
        "Incoming Relationships / 入边",
        "Outgoing Relationships / 出边",
        "Graph Relationships",
        "Source / 来源",
        "Relationship / 关系",
        "Target / 目标",
        "Default Workspace",
        "Owns",
        "Support AI Project",
        "Retrieves",
        "Refund Policy Knowledge",
        "Graph nodes",
        "9 relationships",
        "/api/v1/workspaces/default/context-graph",
        "/api/v1/contexts/support-resolution-agent/components",
        "/api/v1/contexts/support-resolution-agent/components/refund-policy",
        "/api/v1/contexts/support-resolution-agent/evaluation-runs",
        "/api/v1/contexts/support-resolution-agent/evaluation-scorecard?suite_name=Safety+Regression+Suite&model_version=deepseek-chat",
        "/api/v1/contexts/support-resolution-agent/evaluation-runs/safety-regression",
        "metadata only",
        "metrics JSON",
        "accuracy",
        "latency_ms",
        "policy-handbook",
        "sha256:preview",
    ]:
        assert page.get_by_text(text, exact=False).count() > 0, f"missing text: {text}"

    overflow = page.evaluate(
        "() => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1"
    )
    assert not overflow, f"{name} viewport has horizontal overflow"

    if name == "desktop":
        component_summary = page.locator(".component-summary").first.bounding_box()
        assert component_summary is not None, "missing component summary layout box"
        assert (
            component_summary["width"] >= 110
        ), f"component inventory summary column is too narrow: {component_summary['width']}"

    benchmark_workspace_token = page.locator('input[name="benchmark-workspace-bearer-token"]')
    benchmark_workspace_cohort = page.locator('input[name="benchmark-workspace-revised-cohort"]')
    benchmark_workspace_button = page.get_by_role(
        "button", name="Load exact workspace / 加载精确工作台"
    )
    assert benchmark_workspace_token.count() == 1
    assert benchmark_workspace_cohort.count() == 1
    assert benchmark_workspace_token.get_attribute("type") == "password"
    assert benchmark_workspace_token.get_attribute("aria-describedby")
    assert benchmark_workspace_button.is_disabled()
    benchmark_workspace_token.fill("preview-benchmark-token")
    benchmark_workspace_cohort.fill("preview-cohort")
    assert not benchmark_workspace_button.is_disabled()
    page.reload(wait_until="networkidle", timeout=60000)
    assert benchmark_workspace_token.input_value() == ""
    assert benchmark_workspace_button.is_disabled()

    project_node = page.get_by_role("button", name="Select Support AI Project (Project)")
    assert project_node.get_attribute("aria-pressed") == "false"
    project_node.focus()
    page.keyboard.press("Enter")
    assert project_node.get_attribute("aria-pressed") == "true"
    assert page.get_by_label("Selected context graph node detail").get_by_text(
        "project:support-ai", exact=True
    ).count() == 1

    graph_node_select = page.get_by_label("Select context graph node")
    graph_node_select.select_option("workspace:default")
    workspace_node = page.get_by_role("button", name="Select Default Workspace (Workspace)")
    assert workspace_node.get_attribute("aria-pressed") == "true"

    base_commit_select = page.get_by_label("Base commit / 基线提交")
    compare_commit_select = page.get_by_label("Compare commit / 对比提交")
    assert base_commit_select.is_disabled()
    assert compare_commit_select.is_disabled()

    lifecycle_editor = page.locator("#context-lifecycle-editor-heading")
    if lifecycle_editor.count() == 1:
        for text in ["Bearer token", "Load state", "Create component", "No credential is persisted"]:
            assert page.get_by_text(text, exact=False).count() > 0, f"missing text: {text}"
        lifecycle_token = page.get_by_label("Bearer token / 访问令牌")
        assert lifecycle_token.get_attribute("type") == "password"
        assert lifecycle_token.get_attribute("aria-describedby")
        create_button = page.get_by_role("button", name="Create component / 创建组件")
        assert create_button.is_disabled()
        lifecycle_token.fill("preview-local-token")
        assert not page.get_by_role("button", name="Load state / 加载状态").is_disabled()
        assert create_button.is_disabled()
        page.get_by_label("Component content / 组件正文").fill("Preview lifecycle content")
        assert not create_button.is_disabled()

        page.get_by_label("Operation / 操作").select_option("update")
        update_button = page.get_by_role("button", name="Update component / 更新组件")
        assert update_button.count() == 1
        assert update_button.is_disabled()
        assert page.get_by_label("Component / 组件").count() == 1

        page.get_by_label("Operation / 操作").select_option("remove")
        remove_button = page.get_by_role("button", name="Remove component / 移除组件")
        assert remove_button.count() == 1
        assert remove_button.is_disabled()
        assert "cl-button--danger" in (remove_button.get_attribute("class") or "")
        assert page.get_by_role("checkbox", name="Confirm removal", exact=False).is_disabled()
        assert page.get_by_label("Component content / 组件正文").count() == 0

        page.reload(wait_until="networkidle", timeout=60000)
        assert page.get_by_label("Bearer token / 访问令牌").input_value() == ""
        assert page.get_by_role("button", name="Create component / 创建组件").is_disabled()
    else:
        assert lifecycle_editor.count() == 0

    screenshot_dir = Path("target")
    screenshot_dir.mkdir(exist_ok=True)
    page.screenshot(path=str(screenshot_dir / f"context-workspace-{name}.png"), full_page=True)


inspect_source_boundaries()

with sync_playwright() as playwright:
    browser = playwright.chromium.launch(headless=True)
    page = browser.new_page()
    console_errors = []
    page.on("console", lambda message: console_errors.append(message.text) if message.type == "error" else None)

    inspect_viewport(page, 1440, 1000, "desktop")
    inspect_viewport(page, 390, 900, "mobile")

    browser.close()

assert not console_errors, f"console errors: {console_errors}"
