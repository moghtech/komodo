import { AppShell, Burger, Button, Flex, Text } from "@mantine/core";
import { useNavigate } from "react-router-dom";
import { ThemeToggle } from "@/components/common/theme-toggle";
import { UserDropdown } from "@/components/user-dropdown";

const Topbar = ({
  opened,
  toggle,
}: {
  opened: boolean;
  toggle: () => void;
}) => {
  const nav = useNavigate();
  return (
    <AppShell.Header
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
      }}
      pl="1.3rem"
      pr="2rem"
      py="0rem"
    >
      <Flex align="center" gap="md">
        <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
        <Button
          variant="subtle"
          c="inherit"
          leftSection={
            <img src="/mogh-512x512.png" width={32} alt="moghtech" />
          }
          onClick={() => nav("/")}
        >
          <Text fz="h2" fw="450" lts="0.1rem">
            KOMODO
          </Text>
        </Button>
      </Flex>
      <Flex align="center" gap="0.3rem">
        <ThemeToggle />
        <UserDropdown />
      </Flex>
    </AppShell.Header>
  );
};

export default Topbar;
