import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import { useRead, useSetTitle } from "@lib/hooks";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { RecordedLogViewer } from "@components/log/RecordedLogViewer";
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
      services: selectedServices.length > 0 ? selectedServices : undefined,
    },
    {
      enabled: !!id,
      refetchInterval: recording?.status === Types.LogRecordingStatus.Recording ? 5000 : false
    }
  );

  // Set page title
  useSetTitle(recording ? `Log Recording - ${getSessionName(recording)}` : "Log Recording");

  // Generate session name from recording data
  function getSessionName(rec: Types.LogRecording | Types.LogRecordingListItem): string {
    // Use custom name if provided
    if ('name' in rec && rec.name) {
      return rec.name;
    }

    // Otherwise generate default name
    const date = new Date(rec.start_ts);
    const dateStr = date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    const timeStr = date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    const baseName = `${rec.target.id.slice(0, 8)}_${dateStr}_${timeStr}`;

    // If it's a stack recording with specific services, show them
    if (rec.target.type === 'Stack' && rec.services && rec.services.length > 0) {
      const serviceList = rec.services.length <= 2
        ? rec.services.join(', ')
        : `${rec.services.slice(0, 2).join(', ')}...+${rec.services.length - 2}`;
      return `${baseName} (${serviceList})`;
    }

    return baseName;
  }

  // Add search term
  const addSearchTerm = () => {
    if (search && !searchTerms.includes(search)) {
      setSearchTerms([...searchTerms, search]);
      setSearch("");
    }
  };

  // Remove search term
  const removeSearchTerm = (term: string) => {
    setSearchTerms(searchTerms.filter(t => t !== term));
  };

  // Clear all search
  const clearSearch = () => {
    setSearch("");
    setSearchTerms([]);
  };

  // Get the log content based on stream selection
  const logContent = useMemo(() => {
    if (!logs) return "";

    return stream === "all"
      ? (logs.stdout || "") + (logs.stderr || "")
      : stream === "stdout"
        ? (logs.stdout || "")
        : (logs.stderr || "");
  }, [logs, stream]);

  // Handle export
  const handleExport = useCallback((content: string, filtered: boolean) => {
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    const suffix = filtered ? '-filtered' : '';
    a.download = `${recording ? getSessionName(recording) : 'logs'}${suffix}.log`;
    a.click();
    URL.revokeObjectURL(url);
  }, [recording]);

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
        {/* Enhanced Toolbar - Two Row Layout */}
        <div className="flex flex-col gap-2 w-full">
          {/* Row 1: Display Controls */}
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              {/* Stream selector with better styling */}
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

            {/* Service selector for stack recordings */}
            {recording?.target.type === 'Stack' && recording.services && recording.services.length > 0 && (
              <div className="ml-auto">
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
              </div>
            )}
          </div>

          {/* Row 2: Search & Settings */}
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              {/* Invert toggle before search */}
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Invert</span>
                <Switch checked={invert} onCheckedChange={setInvert} />
              </label>

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

            {/* Settings (right-aligned) */}
            <div className="flex items-center gap-3 ml-auto">
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Case sensitive</span>
                <Switch checked={caseSensitive} onCheckedChange={setCaseSensitive} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Regex</span>
                <Switch checked={regex} onCheckedChange={setRegex} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Line Numbers</span>
                <Switch checked={showLineNumbers} onCheckedChange={setShowLineNumbers} />
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <span className="text-muted-foreground text-sm">Timestamps</span>
                <Switch checked={timestamps} onCheckedChange={setTimestamps} />
              </label>
              <Button variant="outline" size="icon" onClick={() => refetch()}>
                <RefreshCw className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>

        {/* Log content */}
        <div className="flex-1 relative" style={{ height: '75vh' }}>
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
            <RecordedLogViewer
              logs={logContent}
              searchTerms={searchTerms}
              caseSensitive={caseSensitive}
              regex={regex}
              invert={invert}
              showLineNumbers={showLineNumbers}
              onExport={handleExport}
              className="h-full"
            />
          )}
        </div>
      </div>
    </div>
  );
}

