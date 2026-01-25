import {
  ActionIcon,
  Button,
  Divider,
  Flex,
  Menu,
  SimpleGrid,
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

export const UserDropdown = () => {
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
  const userInvalidate = useUserInvalidate();
  const accounts = MoghAuth.LOGIN_TOKENS.accounts();
  const nav = useNavigate();
  return (
    <Menu position="bottom-end" offset={17} opened={open} onChange={setOpen}>
      <Menu.Target>
        <Button
          variant="subtle"
          c="inherit"
          leftSection={<User size="1.3rem" />}
        >
          <Username username={user?.username} />
        </Button>
      </Menu.Target>
      <Menu.Dropdown>
        <Flex
          direction="column"
          gap="xs"
          m="xs"
          mt="0.3rem"
          mb="0.3rem"
          miw={270}
        >
          <Flex align="center" justify="space-between" gap="md" w="100%">
            <Flex align="center" gap="md" opacity={0.8} fz="sm" lh="sm">
              <ArrowLeftRight size="1rem" />
              Switch accounts
            </Flex>
            <ActionIcon
              variant={viewLogout ? "filled" : "subtle"}
              c="inherit"
              onClick={() => setViewLogout((l) => !l)}
            >
              <Settings size="1rem" />
            </ActionIcon>
          </Flex>

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

          <SimpleGrid cols={2}>
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
              Add account
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
          </SimpleGrid>

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
        </Flex>
      </Menu.Dropdown>
    </Menu>
  );
};

const Account = ({
  login,
  current_id,
  setOpen,
  rerender,
  viewLogout,
}: {
  login: MoghAuth.Types.JwtResponse;
  current_id?: string;
  setOpen: (open: boolean) => void;
  rerender: () => void;
  viewLogout: boolean;
}) => {
  const user_id = useMemo(
    () => MoghAuth.extractUserIdFromJwt(login.jwt),
    [login.jwt],
  );
  const { data: user } = useRead(
    "GetUsername",
    { user_id: user_id! },
    { enabled: !!user_id },
  );
  if (!user_id || !user) return;
  const selected = user_id === current_id;
  return (
    <Flex align="center" gap="md" w="100%">
      <Button
        variant={selected ? "light" : "subtle"}
        c="inherit"
        rightSection={
          <Circle
            stroke="none"
            fill="green"
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
          MoghAuth.LOGIN_TOKENS.change(user_id);
          location.reload();
        }}
      >
        {user.avatar && (
          <img
            src={user.avatar}
            alt="avatar"
            style={{ width: "1.3rem", height: "1.3rem", marginRight: "0.5rem" }}
          />
        )}
        {!user.avatar && (
          <User size="1.3rem" style={{ marginRight: "0.5rem" }} />
        )}
        <Username username={user?.username} />
      </Button>

      {viewLogout && (
        <ActionIcon
          variant="filled"
          color="red"
          onClick={() => {
            MoghAuth.LOGIN_TOKENS.remove(user_id);
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
};

const Username = ({ username }: { username: string | undefined }) => {
  return (
    <Text
      style={{
        overflow: "hidden",
        textOverflow: "ellipsis",
        maxWidth: 140,
      }}
      // visibleFrom="lg"
    >
      {username}
    </Text>
  );
};
