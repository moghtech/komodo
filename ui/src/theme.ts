import {
  colorsTuple,
  createTheme,
  darken,
  lighten,
  MantineColorScheme,
  virtualColor,
} from "@mantine/core";

export const DEFAULT_COLOR_SCHEME: MantineColorScheme = "auto";

export const theme = createTheme({
  cursorType: "pointer",
  primaryColor: "green",
  breakpoints: {
    xs: "36em",
    sm: "48em",
    md: "62em",
    lg: "75em",
    xl: "88em",
    xxl: "100em",
  },
  colors: {
    // main background color
    lightMain: colorsTuple("#ffffff"),
    // dark.8
    darkMain: colorsTuple("#1f1f1f"),
    main: virtualColor({
      name: "main",
      light: "lightMain",
      dark: "darkMain",
    }),
    // Accent background color
    lightAccent: colorsTuple(darken("#ffffff", 0.03)),
    darkAccent: colorsTuple(lighten("#1f1f1f", 0.03)),
    accent: virtualColor({
      name: "accent",
      light: "lightAccent",
      dark: "darkAccent",
    }),
  },
});
