export const primitiveTokens = {
  color: {
    ink: {
      950: "#10110f",
      900: "#191b18",
      800: "#252922",
      700: "#353b31",
      500: "#68705f",
      300: "#a8afa1",
      100: "#eceee8",
      50: "#f8f8f4"
    },
    signal: {
      700: "#065f46",
      600: "#047857",
      500: "#0f9f79",
      300: "#66d7bd",
      100: "#d6f6ec"
    },
    cobalt: {
      700: "#224c83",
      600: "#2d68ae",
      500: "#3f7fc8",
      100: "#dcecff"
    },
    amber: {
      600: "#ad6615",
      500: "#d4821f",
      100: "#fdecc8"
    },
    red: {
      600: "#c53b3b",
      100: "#ffe2df"
    }
  },
  radius: {
    xs: "3px",
    sm: "5px",
    md: "8px"
  },
  space: {
    1: "0.25rem",
    2: "0.5rem",
    3: "0.75rem",
    4: "1rem",
    5: "1.25rem",
    6: "1.5rem",
    8: "2rem",
    10: "2.5rem",
    12: "3rem"
  }
} as const;

export const semanticTokens = {
  color: {
    background: "var(--cl-color-bg)",
    foreground: "var(--cl-color-fg)",
    panel: "var(--cl-color-panel)",
    border: "var(--cl-color-border)",
    accent: "var(--cl-color-accent)"
  },
  radius: {
    control: "var(--cl-radius-control)",
    panel: "var(--cl-radius-panel)"
  }
} as const;

export const componentTokens = {
  button: {
    background: "var(--cl-button-bg)",
    foreground: "var(--cl-button-fg)",
    radius: "var(--cl-button-radius)"
  },
  panel: {
    background: "var(--cl-panel-bg)",
    border: "var(--cl-panel-border)",
    radius: "var(--cl-panel-radius)"
  }
} as const;
