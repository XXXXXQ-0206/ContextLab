import { clsx } from "clsx";
import React, { type HTMLAttributes } from "react";

type CodeChipElement = "code" | "div" | "span";

export type CodeChipProps = HTMLAttributes<HTMLElement> & {
  as?: CodeChipElement;
};

export function CodeChip({ as: Element = "code", className, ...props }: CodeChipProps) {
  return <Element className={clsx("cl-code-chip", className)} {...props} />;
}
