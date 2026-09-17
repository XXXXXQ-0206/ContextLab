import { clsx } from "clsx";
import React, { type ButtonHTMLAttributes, type ReactNode } from "react";

export type ButtonTone = "solid" | "muted" | "ghost" | "danger";

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  tone?: ButtonTone;
  icon?: ReactNode;
};

export function Button({
  children,
  className,
  icon,
  tone = "solid",
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button className={clsx("cl-button", `cl-button--${tone}`, className)} type={type} {...props}>
      {icon ? <span className="cl-button__icon">{icon}</span> : null}
      <span className="cl-button__label">{children}</span>
    </button>
  );
}
