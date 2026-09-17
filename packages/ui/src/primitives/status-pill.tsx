import { clsx } from "clsx";
import React, { type ReactNode } from "react";

export type StatusPillTone = "neutral" | "success" | "warning" | "info";

export type StatusPillProps = {
  children: ReactNode;
  tone?: StatusPillTone;
  className?: string;
};

export function StatusPill({ children, className, tone = "neutral" }: StatusPillProps) {
  return <span className={clsx("cl-status-pill", `cl-status-pill--${tone}`, className)}>{children}</span>;
}
