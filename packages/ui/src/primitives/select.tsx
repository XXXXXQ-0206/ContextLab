import { clsx } from "clsx";
import React, { type ReactNode, type SelectHTMLAttributes, useId } from "react";

export type SelectProps = SelectHTMLAttributes<HTMLSelectElement> & {
  label: ReactNode;
  description?: ReactNode;
};

export function Select({ children, className, description, label, ...props }: SelectProps) {
  const generatedId = useId();
  const descriptionId = description ? `${props.id ?? generatedId}-description` : undefined;
  const describedBy = [props["aria-describedby"], descriptionId].filter(Boolean).join(" ") || undefined;

  return (
    <label className="cl-select-field">
      <span className="cl-select-label">{label}</span>
      <select {...props} aria-describedby={describedBy} className={clsx("cl-select", className)}>
        {children}
      </select>
      {description ? <span className="cl-select-description" id={descriptionId}>{description}</span> : null}
    </label>
  );
}
