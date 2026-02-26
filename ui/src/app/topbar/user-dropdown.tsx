import {
  ActionIcon,
  Box,
  Button,
  Divider,
  Flex,
  Group,
  Menu,
  Stack,
  Text,
} from "@mantine/core";
import {
  ArrowLeftRight,
  Circle,
  LogOut,
  Plus,
  Settings,
  User,
} from "lucide-react";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { MoghAuth } from "komodo_client";
import { useRead, useUser, useUserInvalidate } from "@/lib/hooks";
import { hexColorByIntention } from "@/lib/color";

export default function UserDropdown() {
  const [_, setRerender] = useState(false);
  const rerender = () => setRerender((r) => !r);
  const [viewLogout, setViewLogout] = useState(false);
  const [open, _setOpen] = useState(false);
  const setOpen = (open: boolean) => {
    _setOpen(open);
    if (open) {
      setViewLogout(false);
    }
  };
  const user = useUser().data;
  const avatar = (user?.config.data as { avatar?: string }).avatar;
  const userInvalidate = useUserInvalidate();
  const accounts = MoghAuth.LOGIN_TOKENS.accounts();
  const nav = useNavigate();
  return (
    <Menu offset={20} opened={open} onChange={setOpen}>
      <Menu.Target>
        <Button
          variant="subtle"
          size="lg"
          leftSection={
            avatar ? (
              <img
                src={avatar}
                alt="avatar"
                style={{
                  width: "1.2rem",
                  height: "1.2rem",
                }}
              />
            ) : (
              <User size="1.3rem" />
            )
          }
          pl="0.5rem"
          pr={{ base: "-20", lg: "0.5rem" }}
        >
          <Username username={user?.username} />
        </Button>
      </Menu.Target>
      <Menu.Dropdown w={350} maw="96vw">
        <Stack gap="xs" m="xs" mt="0.3rem" mb="0.3rem">
          <Group justify="space-between">
            <Group opacity={0.8} fz="sm" lh="sm">
              <ArrowLeftRight size="1rem" />
              Switch accounts
            </Group>
            <ActionIcon
              variant={viewLogout ? "filled" : "subtle"}
              c="inherit"
              onClick={() => setViewLogout((l) => !l)}
            >
              <LogOut size="1rem" />
            </ActionIcon>
          </Group>

          <Divider />

          {accounts.map((login) => (
            <Account
              key={login.user_id}
              login={login}
              current_id={user?._id?.$oid}
              setOpen={setOpen}
              rerender={rerender}
              viewLogout={viewLogout}
            />
          ))}

          <Divider />

          <Group grow>
            <Button
              variant="subtle"
              c="inherit"
              leftSection={<Plus size="1rem" />}
              onClick={() => {
                setOpen(false);
                nav(
                  `/login?${new URLSearchParams({ backto: `${location.pathname}${location.search}` })}`,
                );
              }}
            >
              <Box component="span">
                Add
                <Box component="span" pl="0.25rem" visibleFrom="xs">
                  Account
                </Box>
              </Box>
            </Button>

            <Button
              leftSection={<Settings size="1rem" />}
              onClick={() => {
                setOpen(false);
                nav("/profile");
              }}
            >
              Profile
            </Button>
          </Group>

          {viewLogout && (
            <Button
              variant="filled"
              color="red"
              rightSection={<LogOut size="1rem" />}
              fullWidth
              onClick={() => {
                MoghAuth.LOGIN_TOKENS.remove_all();
                userInvalidate();
              }}
            >
              Log Out All
            </Button>
          )}
        </Stack>
      </Menu.Dropdown>
    </Menu>
  );
}

function Account({
  login,
  current_id: currentId,
  setOpen,
  rerender,
  viewLogout,
}: {
  login: MoghAuth.Types.JwtResponse;
  current_id?: string;
  setOpen: (open: boolean) => void;
  rerender: () => void;
  viewLogout: boolean;
}) {
  const userId = useMemo(
    () => MoghAuth.extractUserIdFromJwt(login.jwt),
    [login.jwt],
  );
  const { data: user } = useRead(
    "GetUsername",
    { user_id: userId! },
    { enabled: !!userId },
  );
  if (!userId || !user) return;
  const selected = userId === currentId;
  return (
    <Flex align="center" gap="md" w="100%">
      <Button
        variant={selected ? "default" : "subtle"}
        rightSection={
          <Circle
            stroke="none"
            fill={hexColorByIntention("Good")}
            size="0.8rem"
            style={{ display: selected ? undefined : "none" }}
          />
        }
        justify="space-between"
        fullWidth
        onClick={() => {
          if (selected) {
            // Noop
            setOpen(false);
            return;
          }
          MoghAuth.LOGIN_TOKENS.change(userId);
          location.reload();
        }}
      >
        {user.avatar && (
          <img
            src={user.avatar}
            alt="avatar"
            style={{ width: "1.2rem", height: "1.2rem", marginRight: "0.5rem" }}
          />
        )}
        {!user.avatar && (
          <User size="1.3rem" style={{ marginRight: "0.5rem" }} />
        )}
        <Username username={user?.username} alwaysShowUsername />
      </Button>

      {viewLogout && (
        <ActionIcon
          color="red"
          onClick={() => {
            MoghAuth.LOGIN_TOKENS.remove(userId);
            if (selected) {
              location.reload();
            } else {
              rerender();
            }
          }}
        >
          <LogOut size="1rem" />
        </ActionIcon>
      )}
    </Flex>
  );
}

function Username({
  username,
  alwaysShowUsername,
}: {
  username: string | undefined;
  alwaysShowUsername?: boolean;
}) {
  return (
    <Text
      style={{
        overflow: "hidden",
        textOverflow: "ellipsis",
        maxWidth: 140,
      }}
      visibleFrom={alwaysShowUsername ? undefined : "lg"}
    >
      {username}
    </Text>
  );
}
