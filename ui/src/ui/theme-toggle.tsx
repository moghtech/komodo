import {
  ActionIcon,
  MantineColorScheme,
  Menu,
  useComputedColorScheme,
  useMantineColorScheme,
} from "@mantine/core";
import { CheckCircle, Moon, Sun } from "lucide-react";

export default function ThemeToggle() {
  const { colorScheme, setColorScheme } = useMantineColorScheme();
  return (
    <Menu offset={16}>
      <Menu.Target>
        <ActionIcon aria-label="ThemeToggle" size="xl" variant="subtle">
          <ThemeIcon />
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown w={150}>
        {["light", "dark", "auto"].map((theme) => (
          <Menu.Item
            key={theme}
            onClick={() => setColorScheme(theme as MantineColorScheme)}
            style={{ textTransform: "capitalize" }}
            rightSection={
              colorScheme === theme ? <CheckCircle size="0.8rem" /> : undefined
            }
          >
            {theme}
          </Menu.Item>
        ))}
      </Menu.Dropdown>
    </Menu>
  );
}

function ThemeIcon() {
  const currentTheme = useComputedColorScheme();
  const dark = currentTheme === "dark";
  return (
    <>
      <Sun
        color="var(--mantine-color-text)"
        size="1.3rem"
        style={{
          display: dark ? "none" : undefined,
        }}
      />
      <Moon
        color="var(--mantine-color-text)"
        size="1.3rem"
        style={{
          display: dark ? undefined : "none",
        }}
      />
    </>
  );
}
