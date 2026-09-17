import { clsx } from "clsx";
import React, { type HTMLAttributes, type Key, type ReactNode } from "react";

export type DefinitionGridItem = {
  id?: Key;
  label: ReactNode;
  value: ReactNode;
};

export type DefinitionGridProps = HTMLAttributes<HTMLDListElement> & {
  columns?: 1 | 2 | 3 | 4;
  compact?: boolean;
  items: DefinitionGridItem[];
  surface?: "default" | "raised";
  valueFont?: "mono" | "sans";
  valueTone?: "default" | "info";
};

export function DefinitionGrid({
  className,
  columns = 2,
  compact = false,
  items,
  surface = "default",
  valueFont = "mono",
  valueTone = "default",
  ...props
}: DefinitionGridProps) {
  return (
    <dl
      className={clsx(
        "cl-definition-grid",
        `cl-definition-grid--cols-${columns}`,
        compact && "cl-definition-grid--compact",
        surface === "raised" && "cl-definition-grid--raised",
        className
      )}
      {...props}
    >
      {items.map((item, index) => (
        <div className="cl-definition-grid__item" key={item.id ?? index}>
          <dt className="cl-definition-grid__label">{item.label}</dt>
          <dd
            className={clsx(
              "cl-definition-grid__value",
              `cl-definition-grid__value--${valueFont}`,
              valueTone === "info" && "cl-definition-grid__value--info"
            )}
          >
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}
