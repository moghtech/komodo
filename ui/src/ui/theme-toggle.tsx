import {
  ActionIcon,
  MantineColorScheme,
  Menu,
  useComputedColorScheme,
  useMantineColorScheme,
} from "@mantine/core";
import { CheckCircle, Moon, Sun } from "lucide-react";

export const ThemeToggle = () => {
  const { colorScheme, setColorScheme } = useMantineColorScheme();
  return (
    <Menu position="bottom-end" offset={19}>
      <Menu.Target>
        <ActionIcon
          aria-label="ThemeToggle"
          variant="subtle"
          size="lg"
          c="inherit"
        >
          <ThemeIcon />
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown>
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
};

const ThemeIcon = () => {
  const currentTheme = useComputedColorScheme();
  const dark = currentTheme === "dark";
  return (
    <>
      <Sun
        color="black"
        size="1.3rem"
        style={{
          display: dark ? "none" : undefined,
        }}
      />
      <Moon
        color="white"
        size="1.3rem"
        style={{
          display: dark ? undefined : "none",
        }}
      />
    </>
  );
};
