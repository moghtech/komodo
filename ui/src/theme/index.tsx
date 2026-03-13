import {
  ActionIcon,
  Badge,
  Button,
  Code,
  colorsTuple,
  Combobox,
  createTheme,
  CSSVariablesResolver,
  darken,
  Drawer,
  Fieldset,
  HoverCard,
  Input,
  lighten,
  MantineColorScheme,
  MantineProvider,
  MenuDropdown,
  Modal,
  MultiSelect,
  Progress,
  SegmentedControl,
  Select,
  Switch,
  Table,
  Tabs,
  virtualColor,
} from "@mantine/core";
import { Types } from "komodo_client";
import { tagColor } from "@/lib/color";
import { ReactNode } from "react";

// Match in ./index.css
export const LIGHT_BODY = "#f8f9fa";
export const DARK_BODY = "#0f1115";

const DEFAULT_COLOR_SCHEME: MantineColorScheme = "auto";

export default function ThemeProvider({ children }: { children: ReactNode }) {
  return (
    <MantineProvider
      theme={theme}
      cssVariablesResolver={cssVariablesResolver}
      defaultColorScheme={DEFAULT_COLOR_SCHEME}
    >
      {children}
    </MantineProvider>
  );
}

const theme = createTheme({
  cursorType: "pointer",
  primaryColor: "accent",
  breakpoints: {
    xs: "36em",
    sm: "48em",
    md: "66em",
    lg: "82em",
    xl: "96em",
    xl2: "110em",
    xl3: "126em",
    xl4: "142em",
  },
  colors: {
    // Accent background color
    lightAccent: functionColorsTuple(darken(LIGHT_BODY, 0.02), (color) =>
      darken(color, 0.01),
    ),
    darkAccent: functionColorsTuple(lighten(DARK_BODY, 0.025), (color) =>
      lighten(color, 0.02),
    ),
    accent: virtualColor({
      name: "accent",
      light: "lightAccent",
      dark: "darkAccent",
    }),
    // Accent border color
    lightAccentBorder: functionColorsTuple(darken(LIGHT_BODY, 0.1), (color) =>
      darken(color, 0.01),
    ),
    darkAccentBorder: functionColorsTuple(lighten(DARK_BODY, 0.08), (color) =>
      lighten(color, 0.03),
    ),
    "accent-border": virtualColor({
      name: "accent-border",
      light: "lightAccentBorder",
      dark: "darkAccentBorder",
    }),
    lightBw: colorsTuple("#000000"),
    darkBw: colorsTuple("#FFFFFF"),
    bw: virtualColor({
      name: "bw",
      light: "lightBw",
      dark: "darkBw",
    }),
    // Adds the tag colors with increasing opacity
    ...Object.fromEntries(
      Object.values(Types.TagColor).map((color) => {
        return ["Tag" + color, opacityColorsTuple(tagColor(color))];
      }),
    ),
  },
  components: {
    Input: Input.extend({
      styles: (theme) => ({
        input: {
          backgroundColor: theme.colors.accent[1],
          border: "1px solid " + theme.colors["accent-border"][4],
        },
      }),
    }),
    Select: Select.extend({
      styles: (theme) => ({
        input: {
          backgroundColor: theme.colors.accent[1],
          border: "1px solid " + theme.colors["accent-border"][4],
        },
        dropdown: {
          backgroundColor: theme.colors.accent[1],
          borderColor: theme.colors["accent-border"][4],
        },
      }),
    }),
    MultiSelect: MultiSelect.extend({
      styles: (theme) => ({
        input: {
          backgroundColor: theme.colors.accent[1],
          border: "1px solid " + theme.colors["accent-border"][4],
        },
        dropdown: {
          backgroundColor: theme.colors.accent[1],
          borderColor: theme.colors["accent-border"][4],
        },
      }),
      defaultProps: {
        maxDropdownHeight: 250,
      },
    }),
    Combobox: Combobox.extend({
      styles: (theme) => ({
        dropdown: {
          backgroundColor: theme.colors.accent[1],
          border: "1px solid " + theme.colors["accent-border"][4],
        },
      }),
    }),
    MenuDropdown: MenuDropdown.extend({
      defaultProps: {
        bg: "var(--mantine-color-accent-1)",
        bd: "1px solid var(--mantine-color-accent-border-4)",
        bdrs: "md",
      },
    }),
    HoverCard: HoverCard.extend({
      styles: (theme) => ({
        dropdown: {
          backgroundColor: theme.colors.accent[1],
          border: "1px solid " + theme.colors["accent-border"][4],
        },
      }),
    }),
    Button: Button.extend({
      vars: () => ({
        root: {
          "--button-color": "var(--mantine-color-bw-0)",
        },
      }),
      defaultProps: {
        variant: "default",
      },
      classNames: { root: "bordered-heavy-outline" },
    }),
    ActionIcon: ActionIcon.extend({
      vars: () => ({
        root: {
          "--ai-color": "var(--mantine-color-bw-0)",
        },
      }),
    }),
    Switch: Switch.extend({
      defaultProps: {
        color: "green",
      },
    }),
    Badge: Badge.extend({
      defaultProps: {
        bdrs: "sm",
        c: "var(--mantine-color-text)",
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
    Fieldset: Fieldset.extend({
      styles: (theme) => ({
        root: {
          backgroundColor: "var(--mantine-color-body)",
          borderRadius: theme.radius.md,
        },
      }),
      classNames: { root: "bordered-light" },
    }),
    SegmentedControl: SegmentedControl.extend({
      styles: (theme) => ({
        root: {
          backgroundColor: theme.colors.accent[2],
        },
        indicator: {
          backgroundColor: theme.colors.accent[9],
        },
      }),
      classNames: { root: "bordered-heavy" },
    }),
    Progress: Progress.extend({
      styles: (theme) => ({
        root: {
          backgroundColor: theme.colors.accent[4],
        },
      }),
    }),
    Tabs: Tabs.extend({
      styles: (theme) => ({
        list: {
          backgroundColor: theme.colors.accent[1],
          borderRadius: theme.radius.sm,
        },
      }),
    }),
  },
});

const cssVariablesResolver: CSSVariablesResolver = (theme) => ({
  variables: {},
  light: {
    "--mantine-color-default": theme.colors.accent[5],
    "--mantine-color-default-hover": theme.colors.accent[9],
    "--mantine-color-default-border": theme.colors["accent-border"][5],
    "--mantine-color-disabled": theme.colors.accent[1],
  },
  dark: {
    "--mantine-color-default": theme.colors.accent[5],
    "--mantine-color-default-hover": theme.colors.accent[9],
    "--mantine-color-default-border": theme.colors["accent-border"][5],
    "--mantine-color-disabled": theme.colors.accent[1],
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
  return colorsTuple(array);
}
