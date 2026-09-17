import { clsx } from "clsx";
import React, { type ReactNode, type TextareaHTMLAttributes, useId } from "react";

export type TextareaProps = TextareaHTMLAttributes<HTMLTextAreaElement> & {
  label: ReactNode;
  description?: ReactNode;
};

export function Textarea({ className, description, label, ...props }: TextareaProps) {
  const generatedId = useId();
  const descriptionId = description ? `${props.id ?? generatedId}-description` : undefined;
  const describedBy = [props["aria-describedby"], descriptionId].filter(Boolean).join(" ") || undefined;

  return (
    <label className="cl-textarea-field">
      <span className="cl-textarea-label">{label}</span>
      <textarea {...props} aria-describedby={describedBy} className={clsx("cl-textarea", className)} />
      {description ? <span className="cl-textarea-description" id={descriptionId}>{description}</span> : null}
    </label>
  );
}
