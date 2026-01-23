import { useShiftKeyListener } from "@lib/hooks";
import { Link } from "react-router-dom";
import { OmniSearch, OmniDialog } from "../omnibar";
import { ThemeToggle } from "@ui/theme";
import { useState } from "react";
import {
  Api,
  CopyCorePubkey,
  Docs,
  KeyboardShortcuts,
  MobileDropdown,
  TopbarAlerts,
  TopbarUpdates,
  UserDropdown,
  Version,
  WsStatusIndicator,
} from "./components";

export const Topbar = () => {
  const [omniOpen, setOmniOpen] = useState(false);
  useShiftKeyListener("S", () => setOmniOpen(true));

  return (
    <div className="fixed top-0 w-full bg-accent z-50 border-b shadow-sm">
      <div className="container px-[1.2rem] h-16 flex items-center justify-between md:grid md:grid-cols-[auto_1fr] lg:grid-cols-3">
        {/* Logo */}
        <div className="flex items-center gap-1">
          <Link
            to="/"
            className="flex gap-3 items-center text-2xl tracking-widest md:mx-2"
          >
            <img src="/komodo-512x512.png" className="w-[32px]" />
            <div className="hidden lg:block">KOMODO</div>
          </Link>
          <MobileDropdown />
        </div>

        {/* Searchbar */}
        <div className="hidden lg:flex justify-center">
          <OmniSearch setOpen={setOmniOpen} />
        </div>

        {/* Shortcuts */}
        <div className="flex justify-end items-center gap-1">
          <OmniSearch setOpen={setOmniOpen} className="lg:hidden" />
          <div className="flex gap-0">
            <Api />
            <Docs />
            <Version />
          </div>
          <WsStatusIndicator />
          <CopyCorePubkey />
          <KeyboardShortcuts />
          <TopbarAlerts />
          <TopbarUpdates />
          <ThemeToggle />
          <UserDropdown />
        </div>
      </div>
      <OmniDialog open={omniOpen} setOpen={setOmniOpen} />
    </div>
  );
};
