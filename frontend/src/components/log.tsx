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
  Circle,
} from "lucide-react";
import { ReactNode, useEffect, useRef, useState } from "react";
import { Section } from "./layouts";
import { Switch } from "@ui/switch";
import { Input } from "@ui/input";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { useToast } from "@ui/use-toast";
import { useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { RecordingDialog } from "./log/RecordingDialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@ui/dialog";

export type LogStream = "stdout" | "stderr";

export const LogSection = ({
  regular_logs,
  search_logs,
  titleOther,
  extraParams,
  target,
  hideTitle,
  services,
}: {
  regular_logs: (
    timestamps: boolean,
    stream: LogStream,
    tail: number,
    poll: boolean
  ) => {
    Log: ReactNode;
    refetch: () => void;
    stderr: boolean;
  };
  search_logs: (
    timestamps: boolean,
    terms: string[],
    invert: boolean,
    poll: boolean
  ) => { Log: ReactNode; refetch: () => void; stderr: boolean };
  titleOther?: ReactNode;
  extraParams?: ReactNode;
  target?: Types.ResourceTarget;
  hideTitle?: boolean;
  services?: string[];
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
  const [recordingDialogOpen, setRecordingDialogOpen] = useState(false);
  const [stopDialogOpen, setStopDialogOpen] = useState(false);

  // Check if there's an active recording for this target
  const { data: recordings, refetch: refetchRecordings } = useRead(
    "ListLogRecordings",
    {
      active_only: true,
    },
    {
      enabled: !!target,
      refetchInterval: 5000, // Check every 5 seconds
    }
  );

  // Check if this specific target is being recorded
  const activeRecording = recordings?.find(
    r => r.target.type === target?.type && r.target.id === target?.id
  );
  const isRecording = !!activeRecording;

  // Stop recording mutation
  const { mutate: stopRecording } = useWrite("StopLogRecording", {
    onSuccess: () => {
      toast({ title: "Recording stopped" });
      refetchRecordings();
    },
  });

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
    ? search_logs(timestamps, terms, invert, poll)
    : regular_logs(timestamps, stream, Number(tail), poll);

  return (
    <>
      {titleOther && <div className="mb-4">{titleOther}</div>}
      <Section
        title={hideTitle || titleOther ? undefined : "Log"}
        icon={hideTitle || titleOther ? undefined : <ScrollText className="w-4 h-4" />}
        actions={
          <div className="flex flex-col gap-2 w-full">
          {/* Row 1: Primary Controls & Display */}
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              {/* Primary Actions */}
              {target && (
                <Button
                  variant={isRecording ? "destructive" : "outline"}
                  size="icon"
                  onClick={() => {
                    if (isRecording && activeRecording) {
                      // Open stop confirmation dialog
                      setStopDialogOpen(true);
                    } else {
                      // Open dialog to start recording
                      setRecordingDialogOpen(true);
                    }
                  }}
                  title={isRecording ? "Stop recording" : "Start recording"}
                >
                  <Circle
                    className={`w-4 h-4 ${isRecording ? 'animate-pulse' : 'text-red-500'}`}
                    fill='currentColor'
                  />
                </Button>
              )}

              {/* Display Controls */}
              <div className="border rounded-md">
                <ToggleGroup
                  type="single"
                  value={stream}
                  onValueChange={setStream as any}
                >
                  <ToggleGroupItem value="stdout" className="rounded-r-none">
                    stdout
                  </ToggleGroupItem>
                  <ToggleGroupItem value="stderr" className="rounded-l-none">
                    stderr
                    {stderr && (
                      <AlertOctagon className="w-4 h-4 ml-2 stroke-red-500" />
                    )}
                  </ToggleGroupItem>
                </ToggleGroup>
              </div>
            </div>

            {/* Context Selector (right-aligned) */}
            {extraParams && (
              <div className="ml-auto">
                {extraParams}
              </div>
            )}
          </div>

          {/* Row 2: Search & Settings */}
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              {/* Invert toggle moved before search */}
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Invert</span>
                <Switch checked={invert} onCheckedChange={setInvert} />
              </label>

              {/* Search Terms - shown before search input */}
              {terms.map((term, index) => (
                <Button
                  key={term}
                  variant="destructive"
                  onClick={() => setTerms(terms.filter((_, i) => i !== index))}
                  className="flex gap-2 items-center py-0 px-2 h-8"
                >
                  {term}
                  <X className="w-3 h-3" />
                </Button>
              ))}

              {/* Search Section */}
              <div className="relative">
                <Input
                  placeholder="Search Logs"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  onBlur={addTerm}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") addTerm();
                  }}
                  className="w-[200px] xl:w-[280px]"
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

              {/* Tail Length Selector - disabled when search is active */}
              <TailLengthSelector
                selected={tail}
                onSelect={set}
                disabled={terms.length > 0}
              />

              {terms.length > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={clearSearch}
                >
                  Clear all
                </Button>
              )}
            </div>

            {/* Settings (right-aligned) */}
            <div className="flex items-center gap-3 ml-auto">
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Timestamps</span>
                <Switch checked={timestamps} onCheckedChange={setTimestamps} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Auto-refresh</span>
                <Switch checked={poll} onCheckedChange={setPoll} />
              </label>
              <Button
                variant="outline"
                size="icon"
                onClick={() => refetch()}
                title={
                  poll
                    ? terms.length > 0
                      ? "Refresh now (auto-refreshing every 10s)"
                      : "Refresh now (auto-refreshing every 3s)"
                    : "Refresh logs"
                }
              >
                <RefreshCw className="w-4 h-4" />
              </Button>
            </div>
          </div>
          </div>
        }
      >
        {Log}
      </Section>

      {/* Recording Dialog */}
      {target && (
        <RecordingDialog
          open={recordingDialogOpen}
          onOpenChange={setRecordingDialogOpen}
          target={target}
          services={services || []}
          onRecordingStarted={() => {
            setRecordingDialogOpen(false);
            refetchRecordings();
          }}
        />
      )}

      {/* Stop Recording Confirmation Dialog */}
      <Dialog open={stopDialogOpen} onOpenChange={setStopDialogOpen}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>Stop Recording?</DialogTitle>
            <DialogDescription>
              This will stop the current log recording session. The recorded logs will be saved and available in the log recordings page.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setStopDialogOpen(false)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                if (activeRecording) {
                  stopRecording({ id: activeRecording.id });
                  setStopDialogOpen(false);
                }
              }}
            >
              Stop Recording
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
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
