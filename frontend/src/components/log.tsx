import { logToHtml } from "@lib/utils";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import {
  AlertOctagon,
  ChevronDown,
  RefreshCw,
  ScrollText,
  X,
} from "lucide-react";
import { ReactNode, useEffect, useRef, useState } from "react";
import { Section } from "./layouts";
import { Switch } from "@ui/switch";
import { Input } from "@ui/input";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { useToast } from "@ui/use-toast";
import { useLocalStorage } from "@lib/hooks";

export type LogStream = "stdout" | "stderr";

export const LogSection = ({
  regular_logs,
  search_logs,
  titleOther,
}: {
  regular_logs: (
    timestamps: boolean,
    stream: LogStream,
    tail: number
  ) => {
    Log: ReactNode;
    refetch: () => void;
    stderr: boolean;
  };
  search_logs: (
    timestamps: boolean,
    terms: string[],
    invert: boolean
  ) => { Log: ReactNode; refetch: () => void; stderr: boolean };
  titleOther?: ReactNode;
}) => {
  const { toast } = useToast();
  const [timestamps, setTimestamps] = useLocalStorage(
    "log-timestamps-v1",
    false
  );
  const [stream, setStream] = useState<LogStream>("stdout");
  const [tail, set] = useState("100");
  const [terms, setTerms] = useState<string[]>([]);
  const [invert, setInvert] = useState(false);
  const [search, setSearch] = useState("");
  const [poll, setPoll] = useLocalStorage("log-poll-v1", false);

  const addTerm = () => {
    if (!search.length) return;
    if (terms.includes(search)) {
      toast({ title: "Search term is already present" });
      setSearch("");
      return;
    }
    setTerms([...terms, search]);
    setSearch("");
  };

  const clearSearch = () => {
    setSearch("");
    setTerms([]);
  };

  const { Log, refetch, stderr } = terms.length
    ? search_logs(timestamps, terms, invert)
    : regular_logs(timestamps, stream, Number(tail));

  return (
    <Section
      title={titleOther ? undefined : "Log"}
      icon={titleOther ? undefined : <ScrollText className="w-4 h-4" />}
      titleOther={titleOther}
      itemsCenterTitleRow
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          <div className="flex items-center gap-2">
            <div className="text-muted-foreground flex gap-1 text-sm">
              Invert
            </div>
            <Switch checked={invert} onCheckedChange={setInvert} />
          </div>
          {terms.map((term, index) => (
            <Button
              key={term}
              variant="destructive"
              onClick={() => setTerms(terms.filter((_, i) => i !== index))}
              className="flex gap-2 items-center py-0 px-2"
            >
              {term}
              <X className="w-4 h-h" />
            </Button>
          ))}
          <div className="relative">
            <Input
              placeholder="Search Logs"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              onBlur={addTerm}
              onKeyDown={(e) => {
                if (e.key === "Enter") addTerm();
              }}
              className="w-[180px] xl:w-[240px]"
            />
            <Button
              variant="ghost"
              size="icon"
              onClick={clearSearch}
              className="absolute right-0 top-1/2 -translate-y-1/2"
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
          <ToggleGroup
            type="single"
            value={stream}
            onValueChange={setStream as any}
          >
            <ToggleGroupItem value="stdout">stdout</ToggleGroupItem>
            <ToggleGroupItem value="stderr">
              stderr
              {stderr && (
                <AlertOctagon className="w-4 h-4 ml-2 stroke-red-500" />
              )}
            </ToggleGroupItem>
          </ToggleGroup>
          <Button variant="secondary" size="icon" onClick={() => refetch()}>
            <RefreshCw className="w-4 h-4" />
          </Button>
          <div
            className="flex items-center gap-2 cursor-pointer"
            onClick={() => setTimestamps((t) => !t)}
          >
            <div className="text-muted-foreground text-sm">Timestamps</div>
            <Switch checked={timestamps} />
          </div>
          <div
            className="flex items-center gap-2 cursor-pointer"
            onClick={() => setPoll((p) => !p)}
          >
            <div className="text-muted-foreground text-sm">Poll</div>
            <Switch checked={poll} />
          </div>
          <TailLengthSelector
            selected={tail}
            onSelect={set}
            disabled={search.length > 0}
          />
        </div>
      }
    >
      {Log}
    </Section>
  );
};

export const Log = ({
  log,
  stream,
}: {
  log: Types.Log | undefined;
  stream: "stdout" | "stderr";
}) => {
  const _log = log?.[stream as keyof typeof log] as string | undefined;
  const ref = useRef<HTMLDivElement>(null);
  const scroll = () =>
    ref.current?.scroll({
      top: ref.current.scrollHeight,
      behavior: "smooth",
    });
  useEffect(scroll, [_log]);
  return (
    <>
      <div ref={ref} className="h-[75vh] overflow-y-auto">
        <pre
          dangerouslySetInnerHTML={{
            __html: _log ? logToHtml(_log) : `no ${stream} logs`,
          }}
          className="-scroll-mt-24 pb-[20vh]"
        />
      </div>
      <Button
        variant="secondary"
        className="absolute top-4 right-4"
        onClick={scroll}
      >
        <ChevronDown className="h-4 w-4" />
      </Button>
    </>
  );
};

export const TailLengthSelector = ({
  selected,
  onSelect,
  disabled,
}: {
  selected: string;
  onSelect: (value: string) => void;
  disabled?: boolean;
}) => (
  <Select value={selected} onValueChange={onSelect} disabled={disabled}>
    <SelectTrigger className="w-[120px]">
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      <SelectGroup>
        {["100", "500", "1000", "5000"].map((length) => (
          <SelectItem key={length} value={length}>
            {length} lines
          </SelectItem>
        ))}
      </SelectGroup>
    </SelectContent>
  </Select>
);
