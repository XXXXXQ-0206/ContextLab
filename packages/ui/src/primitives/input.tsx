import { clsx } from "clsx";
import React, { type InputHTMLAttributes, type ReactNode, useId } from "react";

export type InputProps = InputHTMLAttributes<HTMLInputElement> & {
  label: ReactNode;
  description?: ReactNode;
};

export function Input({ className, description, label, ...props }: InputProps) {
  const generatedId = useId();
  const descriptionId = description ? `${props.id ?? generatedId}-description` : undefined;
  const describedBy = [props["aria-describedby"], descriptionId].filter(Boolean).join(" ") || undefined;

  return (
    <label className="cl-input-field">
      <span className="cl-input-label">{label}</span>
      <input {...props} aria-describedby={describedBy} className={clsx("cl-input", className)} />
      {description ? <span className="cl-input-description" id={descriptionId}>{description}</span> : null}
    </label>
  );
}
