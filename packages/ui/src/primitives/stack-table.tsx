import { clsx } from "clsx";
import React, { type CSSProperties, type HTMLAttributes, type Key, type ReactNode } from "react";

export type StackTableRow = {
  id?: Key;
  cells: ReactNode[];
};

export type StackTableProps = HTMLAttributes<HTMLDivElement> & {
  columnTemplate?: string;
  headers: ReactNode[];
  rows: StackTableRow[];
};

export function StackTable({
  className,
  columnTemplate,
  headers,
  rows,
  style,
  ...props
}: StackTableProps) {
  const tableStyle = {
    ...style,
    "--cl-stack-table-columns": columnTemplate ?? `repeat(${headers.length}, minmax(0, 1fr))`
  } as CSSProperties;

  return (
    <div className={clsx("cl-stack-table", className)} role="table" style={tableStyle} {...props}>
      <div className="cl-stack-table__head" role="row">
        {headers.map((header, index) => (
          <span className="cl-stack-table__head-cell" key={index} role="columnheader">
            {header}
          </span>
        ))}
      </div>
      {rows.map((row, rowIndex) => (
        <div className="cl-stack-table__row" key={row.id ?? rowIndex} role="row">
          {row.cells.map((cell, cellIndex) => (
            <div className="cl-stack-table__cell" key={cellIndex} role="cell">
              {cell}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
