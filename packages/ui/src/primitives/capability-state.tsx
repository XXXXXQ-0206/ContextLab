import { clsx } from "clsx";
import type { CSSProperties, HTMLAttributes, ReactNode } from "react";
import React from "react";

export type CapabilityStateKind = "loading" | "error" | "empty" | "available" | "unavailable";

export type CapabilityStateProps = Omit<HTMLAttributes<HTMLElement>, "children" | "role"> & {
  state: CapabilityStateKind;
  stateLabel: ReactNode;
  label: ReactNode;
  description: ReactNode;
  detail?: ReactNode;
  children?: ReactNode;
  ariaLabel: string;
};

type CapabilityStateA11y = {
  role: "alert" | "status";
  live: "assertive" | "polite";
  busy: boolean;
};

type CapabilityStateAppearance = {
  background: string;
  borderColor: string;
  accent: string;
};

const stateA11y: Record<CapabilityStateKind, CapabilityStateA11y> = {
  loading: { role: "status", live: "polite", busy: true },
  error: { role: "alert", live: "assertive", busy: false },
  empty: { role: "status", live: "polite", busy: false },
  available: { role: "status", live: "polite", busy: false },
  unavailable: { role: "status", live: "polite", busy: false }
};

const stateAppearance: Record<CapabilityStateKind, CapabilityStateAppearance> = {
  loading: {
    background: "color-mix(in srgb, var(--cl-color-cobalt-100) 42%, var(--cl-color-panel))",
    borderColor: "color-mix(in srgb, var(--cl-color-cobalt-600) 32%, var(--cl-color-border))",
    accent: "var(--cl-color-info)"
  },
  error: {
    background: "color-mix(in srgb, var(--cl-color-red-100) 48%, var(--cl-color-panel))",
    borderColor: "color-mix(in srgb, var(--cl-color-danger) 34%, var(--cl-color-border))",
    accent: "var(--cl-color-danger)"
  },
  empty: {
    background: "var(--cl-color-panel-raised)",
    borderColor: "var(--cl-color-border)",
    accent: "var(--cl-color-muted)"
  },
  available: {
    background: "color-mix(in srgb, var(--cl-color-signal-100) 44%, var(--cl-color-panel))",
    borderColor: "color-mix(in srgb, var(--cl-color-signal-600) 32%, var(--cl-color-border))",
    accent: "var(--cl-color-signal-700)"
  },
  unavailable: {
    background: "color-mix(in srgb, var(--cl-color-amber-100) 48%, var(--cl-color-panel))",
    borderColor: "color-mix(in srgb, var(--cl-color-amber-600) 32%, var(--cl-color-border))",
    accent: "var(--cl-color-warning)"
  }
};

const containerStyle: CSSProperties = {
  border: "1px solid",
  borderRadius: "var(--cl-radius-md)",
  display: "grid",
  gap: "var(--cl-space-3)",
  maxWidth: "100%",
  minWidth: 0,
  overflowWrap: "anywhere",
  padding: "var(--cl-space-4)",
  width: "100%"
};

const headerStyle: CSSProperties = {
  alignItems: "flex-start",
  display: "flex",
  flexWrap: "wrap",
  gap: "var(--cl-space-2)",
  justifyContent: "space-between",
  minWidth: 0
};

const labelStyle: CSSProperties = {
  color: "var(--cl-color-fg)",
  fontSize: "var(--cl-text-base)",
  fontWeight: 650,
  lineHeight: "var(--cl-leading-tight)",
  margin: 0,
  minWidth: 0
};

const descriptionStyle: CSSProperties = {
  color: "var(--cl-color-muted)",
  fontSize: "var(--cl-text-sm)",
  lineHeight: "var(--cl-leading-copy)",
  margin: 0,
  minWidth: 0
};

const detailStyle: CSSProperties = {
  color: "var(--cl-color-muted)",
  fontFamily: "var(--cl-font-mono)",
  fontSize: "var(--cl-text-xs)",
  lineHeight: "var(--cl-leading-copy)",
  margin: 0,
  minWidth: 0
};

export function CapabilityState({
  ariaLabel,
  children,
  className,
  description,
  detail,
  label,
  state,
  stateLabel,
  style,
  ...props
}: CapabilityStateProps) {
  const accessibility = stateA11y[state];
  const appearance = stateAppearance[state];

  return (
    <section
      {...props}
      aria-atomic="true"
      aria-busy={accessibility.busy || undefined}
      aria-label={ariaLabel}
      aria-live={accessibility.live}
      className={clsx("cl-capability-state", `cl-capability-state--${state}`, className)}
      data-layout="responsive"
      data-state={state}
      role={accessibility.role}
      style={{ ...containerStyle, ...appearance, ...style }}
    >
      <div style={headerStyle}>
        <p style={labelStyle}>{label}</p>
        <span
          style={{
            alignItems: "center",
            background: "var(--cl-color-panel)",
            border: "1px solid currentColor",
            borderRadius: "999px",
            color: appearance.accent,
            display: "inline-flex",
            flex: "0 1 auto",
            fontSize: "var(--cl-text-xs)",
            fontWeight: 650,
            lineHeight: "var(--cl-leading-tight)",
            maxWidth: "100%",
            overflowWrap: "anywhere",
            padding: "0.2rem var(--cl-space-2)"
          }}
        >
          {stateLabel}
        </span>
      </div>
      <p style={descriptionStyle}>{description}</p>
      {detail ? <p style={detailStyle}>{detail}</p> : null}
      {children}
    </section>
  );
}
