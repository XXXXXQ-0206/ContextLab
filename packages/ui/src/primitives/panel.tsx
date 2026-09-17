import { clsx } from "clsx";
import type { HTMLAttributes, ReactNode } from "react";

type PanelElement = "section" | "article" | "aside" | "div";

export type PanelProps = HTMLAttributes<HTMLElement> & {
  as?: PanelElement;
};

export function Panel({ as: Element = "section", className, ...props }: PanelProps) {
  return <Element className={clsx("cl-panel", className)} {...props} />;
}

export type PanelHeaderProps = HTMLAttributes<HTMLDivElement> & {
  actions?: ReactNode;
};

export function PanelHeader({ actions, children, className, ...props }: PanelHeaderProps) {
  return (
    <div className={clsx("cl-panel__header", className)} {...props}>
      <div>{children}</div>
      {actions ? <div className="cl-panel__actions">{actions}</div> : null}
    </div>
  );
}
