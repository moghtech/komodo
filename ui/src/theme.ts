import {
  colorsTuple,
  createTheme,
  darken,
  Input,
  lighten,
  MantineColorScheme,
  Table,
  virtualColor,
} from "@mantine/core";

export const DEFAULT_COLOR_SCHEME: MantineColorScheme = "auto";

// Match in ./index.css
const LIGHT_BODY = "#ffffff";
const DARK_BODY = "#131313";

export const theme = createTheme({
  cursorType: "pointer",
  primaryColor: "green",
  breakpoints: {
    xs: "36em",
    sm: "48em",
    md: "62em",
    lg: "75em",
    xl: "88em",
    xl2: "104em",
    xl3: "120em",
    xl4: "136em",
  },
  colors: {
    // Accent background color
    lightAccent: colorsTuple(darken(LIGHT_BODY, 0.04)),
    darkAccent: colorsTuple(lighten(DARK_BODY, 0.04)),
    accent: virtualColor({
      name: "accent",
      light: "lightAccent",
      dark: "darkAccent",
    }),
    // Accent border color
    lightAccentBorder: colorsTuple(darken(LIGHT_BODY, 0.07)),
    darkAccentBorder: colorsTuple(lighten(DARK_BODY, 0.07)),
    "accent-border": virtualColor({
      name: "accent-border",
      light: "lightAccentBorder",
      dark: "darkAccentBorder",
    }),
    // Accent background color on hover
    lightAccentHover: colorsTuple(darken(LIGHT_BODY, 0.02)),
    darkAccentHover: colorsTuple(lighten(DARK_BODY, 0.02)),
    "accent-hover": virtualColor({
      name: "accent-hover",
      light: "lightAccentHover",
      dark: "darkAccentHover",
    }),
  },
  components: {
    Input: Input.extend({
      styles: (theme) => ({
        input: {
          backgroundColor: theme.colors.accent[0],
        },
      }),
    }),
    Table: Table.extend({
      vars: (theme) => ({
        table: {
          "--table-striped-color": theme.colors.accent[0],
          "--table-border-color": theme.colors["accent-border"][0],
          "--table-highlight-on-hover-color": theme.colors["accent-border"][0],
        },
      }),
    }),
  },
});
