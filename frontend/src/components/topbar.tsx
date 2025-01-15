import { useRead, useResourceParamType, useShiftKeyListener } from "@lib/hooks";
import { ResourceComponents } from "./resources";
import {
  AlertTriangle,
  Bell,
  Box,
  Boxes,
  FileQuestion,
  FolderTree,
  Keyboard,
  LayoutDashboard,
  Settings,
  User,
  Users,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { Button } from "@ui/button";
import { Link } from "react-router-dom";
import { RESOURCE_TARGETS, usableResourcePath } from "@lib/utils";
import { OmniSearch, OmniDialog } from "./omnibar";
import { WsStatusIndicator } from "@lib/socket";
import { TopbarUpdates } from "./updates/topbar";
import { Logout } from "./util";
import { ThemeToggle } from "@ui/theme";
import { useAtom } from "jotai";
import { ReactNode, useState } from "react";
import { HomeView, homeViewAtom } from "@main";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Badge } from "@ui/badge";
import { TopbarAlerts } from "./alert/topbar";

export const Topbar = () => {
  const [omniOpen, setOmniOpen] = useState(false);
  useShiftKeyListener("S", () => setOmniOpen(true));

  return (
    <div className="fixed top-0 w-full bg-accent z-50 border-b shadow-sm">
      <div className="container h-16 flex items-center justify-between md:grid md:grid-cols-[auto_1fr] lg:grid-cols-3">
        {/* Logo */}
        <Link
          to="/"
          className="flex gap-3 items-center text-2xl tracking-widest md:mx-2"
        >
          <img src="/greengrey-outline.svg" className="w-[32px]" />
          <div className="hidden lg:block">KOMODO</div>
        </Link>

        {/* Searchbar */}
        <div className="hidden lg:flex justify-center">
          <OmniSearch setOpen={setOmniOpen} />
        </div>

        {/* Shortcuts */}
        <div className="flex justify-end items-center gap-1">
          <MobileDropdown />
          <OmniSearch setOpen={setOmniOpen} className="lg:hidden" />
          <div className="flex gap-0">
            <Docs />
            <Version />
          </div>
          <WsStatusIndicator />
          <KeyboardShortcuts />
          <TopbarAlerts />
          <TopbarUpdates />
          <ThemeToggle />
          <Logout />
        </div>
      </div>
      <OmniDialog open={omniOpen} setOpen={setOmniOpen} />
    </div>
  );
};

const Docs = () => (
  <a
    href="https://komo.do/docs/intro"
    target="_blank"
    className="hidden lg:block"
  >
    <Button variant="link" size="sm" className="px-2">
      <div>Docs</div>
    </Button>
  </a>
);

const Version = () => {
  const version = useRead("GetVersion", {}).data?.version;

  if (!version) return null;
  return (
    <a
      href="https://github.com/mbecker20/komodo/releases"
      target="_blank"
      className="hidden lg:block"
    >
      <Button variant="link" size="sm" className="px-2">
        <div>v{version}</div>
      </Button>
    </a>
  );
};

const MobileDropdown = () => {
  const type = useResourceParamType();
  const Components = type && ResourceComponents[type];
  const [view, setView] = useAtom<HomeView>(homeViewAtom);

  const [icon, title] = Components
    ? [
        <Components.Icon />,
        (type === "ServerTemplate"
          ? "Template"
          : type === "ResourceSync"
          ? "Sync"
          : type) + "s",
      ]
    : location.pathname === "/" && view === "Dashboard"
    ? [<LayoutDashboard className="w-4 h-4" />, "Dashboard"]
    : location.pathname === "/" && view === "Resources"
    ? [<Boxes className="w-4 h-4" />, "Resources"]
    : location.pathname === "/" && view === "Tree"
    ? [<FolderTree className="w-4 h-4" />, "Tree"]
    : location.pathname === "/containers"
    ? [<Box className="w-4 h-4" />, "Containers"]
    : location.pathname === "/settings"
    ? [<Settings className="w-4 h-4" />, "Settings"]
    : location.pathname === "/alerts"
    ? [<AlertTriangle className="w-4 h-4" />, "Alerts"]
    : location.pathname === "/updates"
    ? [<Bell className="w-4 h-4" />, "Updates"]
    : location.pathname.split("/")[1] === "user-groups"
    ? [<Users className="w-4 h-4" />, "User Groups"]
    : location.pathname.split("/")[1] === "users"
    ? [<User className="w-4 h-4" />, "Users"]
    : [<FileQuestion className="w-4 h-4" />, "Unknown"];

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild className="lg:hidden justify-self-end">
        <Button
          variant="ghost"
          className="flex justify-start items-center gap-2 w-36 px-3"
        >
          {icon}
          {title}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-36" side="bottom" align="start">
        <DropdownMenuGroup>
          <DropdownLinkItem
            label="Dashboard"
            icon={<LayoutDashboard className="w-4 h-4" />}
            to="/"
            onClick={() => setView("Dashboard")}
          />
          <DropdownLinkItem
            label="Resources"
            icon={<Boxes className="w-4 h-4" />}
            to="/"
            onClick={() => setView("Resources")}
          />
          <DropdownLinkItem
            label="Containers"
            icon={<Box className="w-4 h-4" />}
            to="/containers"
          />

          <DropdownMenuSeparator />

          {RESOURCE_TARGETS.map((type) => {
            const RTIcon = ResourceComponents[type].Icon;
            const name =
              type === "ServerTemplate"
                ? "Template"
                : type === "ResourceSync"
                ? "Sync"
                : type;
            return (
              <DropdownLinkItem
                key={type}
                label={`${name}s`}
                icon={<RTIcon />}
                to={`/${usableResourcePath(type)}`}
              />
            );
          })}

          <DropdownMenuSeparator />

          <DropdownLinkItem
            label="Alerts"
            icon={<AlertTriangle className="w-4 h-4" />}
            to="/alerts"
          />

          <DropdownLinkItem
            label="Updates"
            icon={<Bell className="w-4 h-4" />}
            to="/updates"
          />

          <DropdownMenuSeparator />

          <DropdownLinkItem
            label="Settings"
            icon={<Settings className="w-4 h-4" />}
            to="/settings"
          />
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const DropdownLinkItem = ({
  label,
  icon,
  to,
  onClick,
}: {
  label: string;
  icon: ReactNode;
  to: string;
  onClick?: () => void;
}) => {
  return (
    <Link to={to} onClick={onClick}>
      <DropdownMenuItem className="flex items-center gap-2 cursor-pointer">
        {icon}
        {label}
      </DropdownMenuItem>
    </Link>
  );
};

const KeyboardShortcuts = () => {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="ghost" size="icon" className="hidden md:flex">
          <Keyboard className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Keyboard Shortcuts</DialogTitle>
        </DialogHeader>
        <div className="grid gap-3 grid-cols-2 pt-8">
          <KeyboardShortcut label="Save" keys={["Ctrl / Cmd", "Enter"]} />
          <KeyboardShortcut label="Go Home" keys={["Shift", "H"]} />

          <KeyboardShortcut label="Go to Servers" keys={["Shift", "G"]} />
          <KeyboardShortcut label="Go to Stacks" keys={["Shift", "Z"]} />
          <KeyboardShortcut label="Go to Deployments" keys={["Shift", "D"]} />
          <KeyboardShortcut label="Go to Builds" keys={["Shift", "B"]} />
          <KeyboardShortcut label="Go to Repos" keys={["Shift", "R"]} />
          <KeyboardShortcut label="Go to Procedures" keys={["Shift", "P"]} />

          <KeyboardShortcut label="Search" keys={["Shift", "S"]} />
          <KeyboardShortcut label="Add Filter Tag" keys={["Shift", "T"]} />
          <KeyboardShortcut
            label="Clear Filter Tags"
            keys={["Shift", "C"]}
            divider={false}
          />
        </div>
      </DialogContent>
    </Dialog>
  );
};

const KeyboardShortcut = ({
  label,
  keys,
  divider = true,
}: {
  label: string;
  keys: string[];
  divider?: boolean;
}) => {
  return (
    <>
      <div>{label}</div>
      <div className="flex items-center gap-2">
        {keys.map((key) => (
          <Badge variant="secondary" key={key}>
            {key}
          </Badge>
        ))}
      </div>

      {divider && (
        <div className="col-span-full bg-gray-600 h-[1px] opacity-40" />
      )}
    </>
  );
};
