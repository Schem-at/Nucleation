import { createTheme, shadow } from "kineglyph";

// Basalt and Vellum belong to Nucleation. Kineglyph only consumes the resulting tokens.
const typography = {
  body: {
    family: 'Inter, "Geist Sans", ui-sans-serif, system-ui, sans-serif',
    size: 14,
    lineHeight: 21,
    weight: 450,
  },
  bodyStrong: {
    family: 'Inter, "Geist Sans", ui-sans-serif, system-ui, sans-serif',
    size: 15,
    lineHeight: 21,
    weight: 650,
  },
  caption: {
    family: 'Inter, "Geist Sans", ui-sans-serif, system-ui, sans-serif',
    size: 12,
    lineHeight: 17,
    weight: 450,
  },
  label: {
    family: '"Geist Mono", ui-monospace, monospace',
    size: 10.5,
    lineHeight: 15,
    weight: 650,
    letterSpacing: 0.55,
  },
  title: {
    family: 'Inter, "Geist Sans", ui-sans-serif, system-ui, sans-serif',
    size: 22,
    lineHeight: 27,
    weight: 650,
    letterSpacing: -0.35,
  },
  display: {
    family: 'Inter, "Geist Sans", ui-sans-serif, system-ui, sans-serif',
    size: 36,
    lineHeight: 40,
    weight: 700,
    letterSpacing: -0.7,
  },
  code: {
    family: '"Geist Mono", ui-monospace, monospace',
    size: 12.5,
    lineHeight: 18,
    weight: 500,
  },
};

export const basaltTheme = createTheme({
  name: "nucleation-dark",
  colors: {
    canvas: "#101216",
    surface: "#16191e",
    surfaceRaised: "#1b1f25",
    surfaceMuted: "#13161a",
    text: "#e8eaed",
    textMuted: "#9299a3",
    accent: "#67cbbb",
    accentContrast: "#101216",
    info: "#7d8fd1",
    success: "#78c9a9",
    warning: "#dfbd79",
    danger: "#dc8c8c",
    connector: "#737b86",
    border: "#303640",
    chart1: "#67cbbb",
    chart2: "#8597d8",
    chart3: "#d59672",
    chart4: "#9fbd78",
    chart5: "#d58da2",
    chart6: "#aa9bd1",
    chartPositive: "#78c9a9",
    chartNegative: "#dc8c8c",
    chartNeutral: "#9299a3",
  },
  radii: { sm: 3, md: 6, lg: 8 },
  typography,
  motion: { fast: 140, normal: 280, slow: 620, easing: "easeInOut" },
  strokes: { hairline: 1, thin: 1.15, regular: 1.5, bold: 2.25 },
  ornament: {
    grid: "none",
    surface: "outlined",
    lineCap: "round",
    eyebrow: true,
  },
  materials: {
    flat: { fill: "canvas" },
    raised: { fill: "surfaceRaised", stroke: "border" },
    floating: {
      fill: "surfaceRaised",
      stroke: "border",
      effects: [
        shadow({ color: "canvas", opacity: 0.22, blur: 12, offset: [0, 4] }),
      ],
    },
    inset: { fill: "surfaceMuted", stroke: "border" },
    glass: { fill: "surfaceRaised", stroke: "border" },
  },
});

export const vellumTheme = createTheme(
  {
    name: "nucleation-light",
    colors: {
      canvas: "#f4f1e9",
      surface: "#faf8f2",
      surfaceRaised: "#fffdf8",
      surfaceMuted: "#ece8de",
      text: "#25282d",
      textMuted: "#6e746f",
      accent: "#237f74",
      accentContrast: "#fffdf8",
      info: "#6475b7",
      success: "#4f9275",
      warning: "#a9792f",
      danger: "#b76060",
      connector: "#858b87",
      border: "#d4cfc4",
      chart1: "#4da99a",
      chart2: "#7f8fc7",
      chart3: "#c48765",
      chart4: "#91ad6c",
      chart5: "#c98297",
      chart6: "#9e90c0",
      chartPositive: "#4f9275",
      chartNegative: "#b76060",
      chartNeutral: "#858b87",
    },
  },
  basaltTheme,
);

export const nucleationThemes = {
  nucleation: basaltTheme,
  "nucleation-dark": basaltTheme,
  "nucleation-light": vellumTheme,
};

export const light = vellumTheme;
export const dark = basaltTheme;
