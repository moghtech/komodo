import {
  Badge,
  Button,
  Code,
  colorsTuple,
  createTheme,
  darken,
  Drawer,
  Fieldset,
  Input,
  lighten,
  MantineColorScheme,
  MenuDropdown,
  Modal,
  Table,
  virtualColor,
} from "@mantine/core";
import { Types } from "komodo_client";
import { tagColor } from "@/lib/color";

export const DEFAULT_COLOR_SCHEME: MantineColorScheme = "auto";

// Match in ./index.css
export const LIGHT_BODY = "#f8f9fa";
export const DARK_BODY = "#0f1115";

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
    lightAccent: functionColorsTuple(
      darken(LIGHT_BODY, 0.005),
      (color) => darken(color, 0.005),
      true,
    ),
    darkAccent: functionColorsTuple(lighten(DARK_BODY, 0.025), (color) =>
      lighten(color, 0.005),
    ),
    accent: virtualColor({
      name: "accent",
      light: "lightAccent",
      dark: "darkAccent",
    }),
    // Accent border color
    lightAccentBorder: functionColorsTuple(
      darken(LIGHT_BODY, 0.1),
      (color) => darken(color, 0.005),
      true,
    ),
    darkAccentBorder: functionColorsTuple(lighten(DARK_BODY, 0.06), (color) =>
      lighten(color, 0.008),
    ),
    "accent-border": virtualColor({
      name: "accent-border",
      light: "lightAccentBorder",
      dark: "darkAccentBorder",
    }),
    // Accent background color on hover
    // lightAccentHover: functionColorsTuple(
    //   darken(LIGHT_BODY, 0.02),
    //   (color) => darken(color, 0.005),
    //   true,
    // ),
    // darkAccentHover: functionColorsTuple(lighten(DARK_BODY, 0.02), (color) =>
    //   lighten(color, 0.005),
    // ),
    // "accent-hover": virtualColor({
    //   name: "accent-hover",
    //   light: "lightAccentHover",
    //   dark: "darkAccentHover",
    // }),
    // Adds the tag colors with increasing opacity
    ...Object.fromEntries(
      Object.values(Types.TagColor).map((color) => {
        return ["Tag" + color, opacityColorsTuple(tagColor(color))];
      }),
    ),
  },
  components: {
    Button: Button.extend({
      defaultProps: {
        // c: "inherit",
      },
    }),
    Badge: Badge.extend({
      defaultProps: {
        bdrs: "sm",
      },
    }),
    Input: Input.extend({
      defaultProps: {
        bg: "accent",
      },
    }),
    Table: Table.extend({
      vars: (theme) => ({
        table: {
          "--table-striped-color": theme.colors.accent[0],
          "--table-border-color": theme.colors["accent-border"][0],
          "--table-highlight-on-hover-color": theme.colors["accent-border"][0],
        },
      }),
      defaultProps: {
        striped: true,
        highlightOnHover: true,
      },
    }),
    Drawer: Drawer.extend({
      vars: () => ({
        root: {
          "--drawer-flex": "",
        },
      }),
      defaultProps: {
        position: "top",
        radius: "md",
      },
      styles: {
        inner: { justifyContent: "center" },
      },
    }),
    Modal: Modal.extend({
      defaultProps: {
        styles: { content: { borderRadius: "var(--mantine-radius-md)" } },
      },
    }),
    Code: Code.extend({
      defaultProps: {
        bg: "var(--mantine-color-accent-0)",
        bdrs: "sm",
        p: "md",
      },
    }),
    MenuDropdown: MenuDropdown.extend({
      defaultProps: {
        bg: "var(--mantine-color-body)",
        bd: "1px solid var(--mantine-color-accent-border-9)",
        bdrs: "md",
      },
    }),
    Fieldset: Fieldset.extend({
      defaultProps: {
        bg: "var(--mantine-color-body)",
        bd: "1px solid var(--mantine-color-accent-border-0)",
        bdrs: "md",
      },
    }),
  },
});

function opacityColorsTuple(baseHex: string, length = 10) {
  return colorsTuple(
    Array.from({ length }).map(
      (_, i) => baseHex + (i * 10 + 9).toString(16).padStart(2, "0"),
    ),
  );
}

function functionColorsTuple(
  base: string,
  fn: (color: string) => string,
  reverse = false,
  length = 10,
) {
  let b = base;
  const array = [
    base,
    ...Array.from({ length: length - 1 }).map(() => {
      b = fn(b);
      return b;
    }),
  ];
  return colorsTuple(reverse ? array.reverse() : array);
}
