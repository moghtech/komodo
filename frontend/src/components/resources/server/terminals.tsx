import { Section } from "@components/layouts";
import { StatusBadge } from "@components/util";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { terminal_history_manager } from "@lib/terminal";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@ui/card";
import { Input } from "@ui/input";
import { useToast } from "@ui/use-toast";
import { X } from "lucide-react";
import { ReactNode, useEffect, useMemo, useRef, useState } from "react";

export const ServerTerminals = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther?: ReactNode;
}) => {
  const { data: terminals, refetch: refetchTerminals } = useRead(
    "ListTerminals",
    { server: id, fresh: true }
  );
  const { mutateAsync: delete_term } = useWrite("DeleteTerminal");
  const [_selected, setSelected] = useLocalStorage<{
    selected: string | undefined;
  }>(`server-${id}-selected-terminal-v2`, { selected: undefined });
  const selected =
    _selected.selected ?? `Terminal ${(terminals?.length ?? 0) + 1}`;
  const [input, setInput] = useLocalStorage(`server-${id}-terminal-input`, {
    input: "",
  });
  const logRef = useRef<HTMLPreElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const scrollBottom = () => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
    // scrollRef.current?.scroll({
    //   top: scrollRef.current.scrollHeight,
    //   behavior: "smooth",
    // });
  };
  const history = useMemo(
    () => terminal_history_manager(id, logRef, scrollRef),
    [id]
  );
  const [[last_cwd, curr, term_history], setRender] = useState(
    history.get_terminal_history(selected)
  );
  useEffect(() => {
    setRender(history.get_terminal_history(selected));
    scrollBottom();
  }, [selected]);

  const execute_terminal = (
    terminal: string,
    command: string,
    onFinish?: () => void
  ) => {
    history.execute_terminal(
      terminal,
      command,
      () => {
        setRender(history.get_terminal_history(terminal));
        refetchTerminals({});
        scrollBottom();
        onFinish?.();
        setTimeout(() => scrollBottom(), 100);
      },
      (error) =>
        toast({
          title: "Execute terminal failed",
          description: JSON.stringify(error, undefined, 2),
          variant: "destructive",
        })
    );
  };

  const delete_terminal = (terminal: string) =>
    delete_term({ server: id, terminal }).then(() => {
      refetchTerminals();
      if (selected === terminal) {
        setSelected({ selected: undefined });
      }
    });

  const { toast } = useToast();

  return (
    <Section titleOther={titleOther}>
      <div className="flex flex-col gap-4">
        <div className="flex gap-2 items-center">
          {terminals?.map((terminal) => (
            <Card
              key={terminal}
              className="flex gap-2 items-center cursor-pointer rounded-sm"
              onClick={() => setSelected({ selected: terminal })}
            >
              <CardHeader className="flex-row gap-2 px-2 py-1">
                <Button
                  variant={selected === terminal ? "outline" : "ghost"}
                  className="p-1"
                >
                  {terminal}
                </Button>
                <Button
                  onClick={(e) => {
                    e.stopPropagation();
                    delete_terminal(terminal);
                  }}
                  variant="destructive"
                  size="icon"
                  className="p-1 w-fit h-fit"
                >
                  <X className="w-4 h-4" />
                </Button>
              </CardHeader>
            </Card>
          ))}
          <Button
            onClick={() => {
              if (_selected.selected !== selected) setSelected({ selected });
              execute_terminal(selected, 'echo "Current dir: $PWD"');
            }}
          >
            New
          </Button>
        </div>

        <div
          ref={scrollRef}
          className="max-h-[50vh] px-2 py-1 border-solid overflow-scroll"
        >
          <div className="flex flex-col gap-2">
            {term_history?.map((history, index) => {
              const output = history.output.trim();
              return (
                <Card key={index}>
                  <CardHeader>
                    <CardTitle className="flex gap-4 items-center">
                      {history.command}
                      {history.code && <ExitCode code={history.code} />}
                    </CardTitle>
                    <CardDescription className="pt-1">
                      {/* {history.next_cwd && <code>{history.next_cwd}</code>} */}
                    </CardDescription>
                  </CardHeader>
                  {output && (
                    <CardContent>
                      <pre>{output}</pre>
                    </CardContent>
                  )}
                </Card>
              );
            })}
            <Card className={!curr ? "hidden" : ""}>
              <CardHeader>
                <CardTitle className="flex gap-4 items-center">
                  {curr?.command}
                  <ExitCode code={curr?.code} />
                </CardTitle>
                <CardDescription className="pt-1">
                  {/* {curr?.next_cwd && <code>{curr.next_cwd}</code>} */}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre ref={logRef} />
                {/* <pre className={!curr?.code ? "hidden" : ""}>
                  {curr?.output}
                </pre> */}
              </CardContent>
            </Card>
          </div>
        </div>

        <div className="flex items-center">
          <Badge variant="secondary" className="text-md">
            {(last_cwd ?? "") + "$"}
          </Badge>
          <Input
            value={input.input}
            onChange={(e) => setInput({ input: e.target.value })}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                execute_terminal(selected, input.input);
                setRender(history.get_terminal_history(selected));
                setInput({ input: "" });
                scrollBottom();
              }
            }}
            placeholder="Enter command"
            spellCheck={false}
          />
        </div>
      </div>
    </Section>
  );
};

const ExitCode = ({ code }: { code: string | undefined }) => {
  if (code === undefined) {
    return <StatusBadge text="Running" intent="Neutral" />;
  } else if (code === "0") {
    return <StatusBadge text="Success" intent="Good" />;
  } else {
    return <StatusBadge text={`Error (${code})`} intent="Critical" />;
  }
};
