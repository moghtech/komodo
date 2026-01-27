import { Button } from "@mantine/core";
import { ChevronDown, ChevronUp } from "lucide-react";

export default function ShowHideButton({
  show,
  setShow,
}: {
  show: boolean;
  setShow: (show: boolean) => void;
}) {
  return (
    <Button
      variant="outline"
      onClick={() => setShow(!show)}
      rightSection={
        show ? <ChevronUp className="w-4" /> : <ChevronDown className="w-4" />
      }
    >
      {show ? "Hide" : "Show"}
    </Button>
  );
}
