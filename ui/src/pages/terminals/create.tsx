import { useShiftKeyListener } from "@/lib/hooks";
import { useState } from "react";

const TERMINAL_TYPES = ["Server", "Container", "Stack", "Deployment"] as const;
type TerminalType = (typeof TERMINAL_TYPES)[number];



export default function CreateTerminal() {
  const [open, setOpen] = useState(false);
  const [type, setType] = useState<TerminalType>("Server");
  useShiftKeyListener("N", () => !open && setOpen(true));

  
  return <></>;
}
