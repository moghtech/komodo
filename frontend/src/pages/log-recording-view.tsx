import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { useRead, useSetTitle } from "@lib/hooks";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { LogViewer } from "@components/log";
import { Skeleton } from "@ui/skeleton";
import {
  Film,
  ChevronLeft,
  X,
  AlertOctagon,
  RefreshCw,
  Clock,
  User,
  Circle,
} from "lucide-react";
import { CaretSortIcon } from "@radix-ui/react-icons";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { useState, useMemo, useCallback } from "react";
import { Link, useParams } from "react-router-dom";

export default function LogRecordingViewPage() {
  const { id } = useParams<{ id: string }>();

  const [search, setSearch] = useState("");
  const [searchTerms, setSearchTerms] = useState<string[]>([]);
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [regex, setRegex] = useState(false);
  const [invert, setInvert] = useState(false);
  const [stream, setStream] = useState<"stdout" | "stderr" | "all">("all");
  const [timestamps, setTimestamps] = useState(false);
  const [selectedServices, setSelectedServices] = useState<string[]>([]);
  const [showLineNumbers, setShowLineNumbers] = useState(false);
  const [wrapLines, setWrapLines] = useState(false);
  const [showLogLevels, setShowLogLevels] = useState(true);

  // Fetch the recording metadata
  const { data: recording, isLoading: recordingLoading } = useRead(
    "GetLogRecording",
    { id: id || "" },
    { enabled: !!id }
  );

  // Fetch the actual logs with auto-refresh for active recordings
  const { data: logs, refetch, isLoading: logsLoading } = useRead(
    "GetRecordedLogs",
    {
      recording_id: id || "",
      tail: 50000, // Large number to get all logs
      timestamps,
    },
    {
      enabled: !!id,
      refetchInterval: recording?.status === Types.LogRecordingStatus.Recording ? 5000 : false
    }
  );

  useSetTitle(recording ? `Log Recording - ${getSessionName(recording)}` : "Log Recording");
  function getSessionName(rec: Types.LogRecording | Types.LogRecordingListItem): string {
    if ('name' in rec && rec.name) {
      return rec.name;
    }

    const date = new Date(rec.start_ts);
    const dateStr = date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    const timeStr = date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    const baseName = `${rec.target.id.slice(0, 8)}_${dateStr}_${timeStr}`;

    if (rec.target.type === 'Stack' && rec.services && rec.services.length > 0) {
      const serviceList = rec.services.length <= 2
        ? rec.services.join(', ')
        : `${rec.services.slice(0, 2).join(', ')}...+${rec.services.length - 2}`;
      return `${baseName} (${serviceList})`;
    }

    return baseName;
  }

  const addSearchTerm = () => {
    if (search && !searchTerms.includes(search)) {
      setSearchTerms([...searchTerms, search]);
      setSearch("");
    }
  };

  const removeSearchTerm = (term: string) => {
    setSearchTerms(searchTerms.filter(t => t !== term));
  };

  const clearSearch = () => {
    setSearch("");
    setSearchTerms([]);
  };
  const logContent = useMemo(() => {
    if (!logs) {
      return "";
    }

    const stdout = logs.stdout || "";
    const stderr = logs.stderr || "";

    let content = "";
    if (stream === "all") {
      content = stdout + (stdout && stderr ? "\n" : "") + stderr;
    } else if (stream === "stdout") {
      content = stdout;
    } else {
      content = stderr;
    }

    return content || "";
  }, [logs, stream]);

  const handleExport = useCallback((content: string, filtered: boolean, format: 'text' | 'json' | 'csv' = 'text') => {
    let exportContent = content;
    let mimeType = 'text/plain';
    let extension = '.log';

    if (format === 'json') {
      const lines = content.split('\n').map((line, index) => ({
        lineNumber: index + 1,
        timestamp: timestamps ? new Date().toISOString() : undefined,
        content: line,
        level: detectLogLevel(line)
      }));
      exportContent = JSON.stringify(lines, null, 2);
      mimeType = 'application/json';
      extension = '.json';
    } else if (format === 'csv') {
      const lines = content.split('\n');
      const csvHeader = 'Line Number,Timestamp,Level,Content\n';
      const csvContent = lines.map((line, index) => {
        const level = detectLogLevel(line) || '';
        const escapedLine = `"${line.replace(/"/g, '""')}"`;
        return `${index + 1},${timestamps ? new Date().toISOString() : ''},${level},${escapedLine}`;
      }).join('\n');
      exportContent = csvHeader + csvContent;
      mimeType = 'text/csv';
      extension = '.csv';
    }

    const blob = new Blob([exportContent], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    const suffix = filtered ? '-filtered' : '';
    a.download = `${recording ? getSessionName(recording) : 'logs'}${suffix}${extension}`;
    a.click();
    URL.revokeObjectURL(url);
  }, [recording, timestamps]);

  const detectLogLevel = (line: string): string | null => {
    if (/\b(ERROR|ERR|FATAL|FAIL)\b/i.test(line)) return 'ERROR';
    if (/\b(WARN|WARNING)\b/i.test(line)) return 'WARN';
    if (/\b(INFO|INFORMATION)\b/i.test(line)) return 'INFO';
    if (/\b(DEBUG|TRACE|VERBOSE)\b/i.test(line)) return 'DEBUG';
    return null;
  };

  if (!id) {
    return (
      <Page
        icon={<Film className="w-8" />}
        title="Log Recording Not Found"
      >
        <div className="text-muted-foreground">
          The requested log recording was not found.
        </div>
      </Page>
    );
  }

  return (
    <div>
      <div className="w-full flex items-center justify-between mb-12">
        <Link to="/log-recordings">
          <Button className="gap-2" variant="secondary">
            <ChevronLeft className="w-4" />
            Back
          </Button>
        </Link>
      </div>
      <div className="flex flex-col gap-6">
        <div>
          <div className="flex items-center gap-3 mb-2">
            <Film className="w-8 h-8" />
            <h1 className="text-3xl font-bold">{recording ? getSessionName(recording) : "Loading..."}</h1>
            {recording?.status === Types.LogRecordingStatus.Recording && (
              <Circle className="w-6 h-6 text-red-500 animate-pulse" fill="currentColor" />
            )}
          </div>
          {recordingLoading ? (
            <div className="flex items-center gap-4 mt-4">
              <Skeleton className="h-4 w-32" />
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-4 w-40" />
            </div>
          ) : recording && (
            <div className="flex items-center gap-4 text-sm text-muted-foreground mt-4">
              <ResourceLink
                type={recording.target.type as any}
                id={recording.target.id}
              />
              <div className="flex items-center gap-1">
                <User className="w-4 h-4" />
                {recording.user_id}
              </div>
              <div className="flex items-center gap-1">
                <Clock className="w-4 h-4" />
                {new Date(recording.start_ts).toLocaleString()}
              </div>
            </div>
          )}
        </div>
        {/* Enhanced Toolbar - Three Row Layout */}
        <div className="flex flex-col gap-4 w-full">
          {/* Row 1: Data Source Controls (stderr, stdout, services, refresh) */}
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-2">
              {/* Stream selector */}
              <div className="border rounded-md">
                <ToggleGroup
                  type="single"
                  value={stream}
                  onValueChange={(v) => v && setStream(v as any)}
                >
                  <ToggleGroupItem value="all" className="rounded-r-none">
                    All
                  </ToggleGroupItem>
                  <ToggleGroupItem value="stdout" className="rounded-none">
                    stdout
                  </ToggleGroupItem>
                  <ToggleGroupItem value="stderr" className="rounded-l-none">
                    stderr
                    {logs?.stderr && (
                      <AlertOctagon className="w-4 h-4 ml-2 stroke-red-500" />
                    )}
                  </ToggleGroupItem>
                </ToggleGroup>
              </div>
            </div>

            {/* Service selector and refresh on the right */}
            <div className="flex items-center gap-2">
              {recording?.target.type === 'Stack' && recording.services && recording.services.length > 0 && (
                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <div className="px-3 py-2 border rounded-md flex items-center gap-2 hover:bg-accent/70 text-sm">
                      <div className="text-muted-foreground">Services:</div>
                      {selectedServices.length === 0
                        ? "All"
                        : selectedServices.length <= 2
                          ? selectedServices.join(", ")
                          : `${selectedServices.slice(0, 2).join(", ")}...+${selectedServices.length - 2}`
                      }
                      <CaretSortIcon className="h-4 w-4 opacity-50" />
                    </div>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    {recording.services.map(service => (
                      <DropdownMenuCheckboxItem
                        key={service}
                        checked={selectedServices.includes(service)}
                        onClick={(e) => {
                          e.preventDefault();
                          setSelectedServices(prev =>
                            prev.includes(service)
                              ? prev.filter(s => s !== service)
                              : [...prev, service]
                          );
                        }}
                      >
                        {service}
                      </DropdownMenuCheckboxItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
              <Button variant="outline" size="icon" onClick={() => refetch()}>
                <RefreshCw className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Row 2: LogViewer Display Settings (timestamps, log levels, line numbers, wrap) */}
          <div className="flex items-center justify-between gap-4 bg-muted/30 rounded-md p-3">
            <div className="flex items-center gap-6">
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-background/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Timestamps</span>
                <Switch checked={timestamps} onCheckedChange={setTimestamps} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-background/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Log Levels</span>
                <Switch checked={showLogLevels} onCheckedChange={setShowLogLevels} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-background/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Line Numbers</span>
                <Switch checked={showLineNumbers} onCheckedChange={setShowLineNumbers} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-background/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Wrap Lines</span>
                <Switch checked={wrapLines} onCheckedChange={setWrapLines} />
              </label>
            </div>
          </div>

          {/* Row 3: Search Controls (invert, search input, search terms, case sensitive, regex) */}
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              {/* Invert toggle */}
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-muted/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Invert</span>
                <Switch checked={invert} onCheckedChange={setInvert} />
              </label>

              {/* Search input */}
              <div className="relative">
                <Input
                  placeholder="Search Logs"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") addSearchTerm();
                  }}
                  className="w-[200px] xl:w-[280px]"
                />
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setSearch("")}
                  className="absolute right-0 top-1/2 -translate-y-1/2"
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>

              {/* Search terms */}
              {searchTerms.map(term => (
                <Button
                  key={term}
                  variant="destructive"
                  onClick={() => removeSearchTerm(term)}
                  className="flex gap-2 items-center py-0 px-2 h-8"
                >
                  {term}
                  <X className="w-3 h-3" />
                </Button>
              ))}

              {searchTerms.length > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={clearSearch}
                >
                  Clear all
                </Button>
              )}
            </div>

            {/* Search Settings (right-aligned) */}
            <div className="flex items-center gap-4 ml-auto">
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-muted/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Case sensitive</span>
                <Switch checked={caseSensitive} onCheckedChange={setCaseSensitive} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer px-2 py-1 hover:bg-muted/50 rounded transition-colors">
                <span className="text-muted-foreground text-sm">Regex</span>
                <Switch checked={regex} onCheckedChange={setRegex} />
              </label>
            </div>
          </div>
        </div>

        {/* Log content */}
        <div className="flex-1 relative" style={{ height: 'calc(100vh - 320px)' }}>
          {logsLoading ? (
            <div className="space-y-2 p-4">
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-3/4" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-2/3" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-4/5" />
            </div>
          ) : (
            <LogViewer
              logs={logContent}
              searchTerms={searchTerms}
              caseSensitive={caseSensitive}
              regex={regex}
              invert={invert}
              showLineNumbers={showLineNumbers}
              onExport={handleExport}
              className="h-full"
              wrapLines={wrapLines}
              showLogLevels={showLogLevels}
            />
          )}
        </div>
      </div>
    </div>
  );
}

