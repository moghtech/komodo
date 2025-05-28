import { Section } from "@components/layouts";
import { komodo_client, useLocalStorage } from "@lib/hooks";
import { Button } from "@ui/button";
import { CardTitle } from "@ui/card";
import { Input } from "@ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { RefreshCcw } from "lucide-react";
import { ReactNode, useCallback, useState } from "react";
import { Terminal } from ".";
import { ContainerExecQuery, TerminalCallbacks } from "komodo_client";

const BASE_SHELLS = ["sh", "bash"];

export const ContainerTerminal = ({
  query: { type, query },
  titleOther,
}: {
  query: ContainerExecQuery;
  titleOther?: ReactNode;
}) => {
  const [_reconnect, _setReconnect] = useState(false);
  const triggerReconnect = () => _setReconnect((r) => !r);
  const [_clear, _setClear] = useState(false);

  const storageKey =
    type === "container"
      ? `server-${query.server}-${query.container}-shell-v1`
      : type === "deployment"
        ? `deployment-${query.deployment}-shell-v1`
        : `stack-${query.stack}-${query.service}-shell-v1`;

  const [shell, setShell] = useLocalStorage(storageKey, "sh");
  const [otherShell, setOtherShell] = useState("");

  const make_ws = useCallback(
    (callbacks: TerminalCallbacks) =>
      komodo_client().connect_container_exec({
        query: { type, query: { ...query, shell } } as any,
        ...callbacks,
      }),
    [query, shell]
  );

  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex items-center gap-4 mr-[16px]">
          <CardTitle className="text-muted-foreground flex items-center gap-2">
            docker exec -it container
            <Select value={shell} onValueChange={setShell}>
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {[
                    ...BASE_SHELLS,
                    ...(!BASE_SHELLS.includes(shell) ? [shell] : []),
                  ].map((shell) => (
                    <SelectItem key={shell} value={shell}>
                      {shell}
                    </SelectItem>
                  ))}
                  <Input
                    placeholder="other"
                    value={otherShell}
                    onChange={(e) => setOtherShell(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        setShell(otherShell);
                        setOtherShell("");
                      } else {
                        e.stopPropagation();
                      }
                    }}
                  />
                </SelectGroup>
              </SelectContent>
            </Select>
          </CardTitle>
          <Button
            className="flex items-center gap-2"
            variant="secondary"
            onClick={() => triggerReconnect()}
          >
            Reconnect
            <RefreshCcw className="w-4 h-4" />
          </Button>
        </div>
      }
    >
      <div className="min-h-[65vh]">
        <Terminal
          make_ws={make_ws}
          selected={true}
          _clear={_clear}
          _reconnect={_reconnect}
        />
      </div>
    </Section>
  );
};
