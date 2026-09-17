import assert from "node:assert/strict";
import test from "node:test";
import { Select } from "@contextlab/ui";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

test("Select wraps a native control with its bilingual label and disabled state", () => {
  const markup = renderToStaticMarkup(
    createElement(
      Select,
      {
        description: "Protected selection / 受保护选择",
        id: "base-commit",
        label: "Base / 基线提交",
        disabled: true,
        value: "baseline",
        onChange() {}
      },
      createElement("option", { value: "baseline" }, "Establish baseline")
    )
  );

  assert.match(markup, /Base \/ 基线提交/);
  assert.match(markup, /<select[^>]*id="base-commit"[^>]*disabled/);
  assert.match(markup, /<option value="baseline" selected="">Establish baseline<\/option>/);
  const descriptionId = markup.match(/aria-describedby="([^"]+)"/)?.[1];
  assert.ok(descriptionId);
  assert.match(markup, new RegExp(`id="${descriptionId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"`));
});
