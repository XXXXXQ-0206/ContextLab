import { clsx } from "clsx";
import React, { type HTMLAttributes, type Key, type ReactNode } from "react";

export type StatGridItem = {
  detail?: ReactNode;
  id?: Key;
  label: ReactNode;
  value: ReactNode;
};

export type StatGridProps = HTMLAttributes<HTMLDivElement> & {
  columns?: 1 | 2 | 3 | 4;
  items: StatGridItem[];
};

export function StatGrid({ className, columns = 4, items, ...props }: StatGridProps) {
  return (
    <div className={clsx("cl-stat-grid", `cl-stat-grid--cols-${columns}`, className)} {...props}>
      {items.map((item, index) => (
        <div className="cl-stat-card" key={item.id ?? index}>
          <span className="cl-stat-card__label">{item.label}</span>
          <strong className="cl-stat-card__value">{item.value}</strong>
          {item.detail ? <small className="cl-stat-card__detail">{item.detail}</small> : null}
        </div>
      ))}
    </div>
  );
}
