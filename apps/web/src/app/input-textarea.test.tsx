import assert from "node:assert/strict";
import test from "node:test";
import { Input, Textarea } from "@contextlab/ui";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";

test("Input and Textarea render accessible shared form fields", () => {
  const markup = renderToStaticMarkup(
    <form>
      <Input
        aria-label="Bearer token / 访问令牌"
        description="In-memory only / 仅内存"
        label="Bearer token / 访问令牌"
        name="bearer-token"
        type="password"
      />
      <Textarea
        aria-label="Component body / 组件正文"
        disabled
        label="Component body / 组件正文"
        name="component-content"
        rows={5}
      />
    </form>
  );

  assert.match(markup, /class="cl-input-field"/);
  assert.match(markup, /type="password"/);
  assert.match(markup, /In-memory only/);
  const descriptionId = markup.match(/aria-describedby="([^"]+)"/)?.[1];
  assert.ok(descriptionId);
  assert.match(markup, new RegExp(`id="${descriptionId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"`));
  assert.match(markup, /class="cl-textarea-field"/);
  assert.match(markup, /rows="5"/);
  assert.match(markup, /disabled=""/);
});
