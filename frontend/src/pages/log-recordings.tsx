import { Page } from "@components/layouts";
import { ResourceLink } from "@components/resources/common";
import {
  useRead,
  useSetTitle,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Input } from "@ui/input";
import { Search, Circle, Square, Trash, Download, Film } from "lucide-react";
import { useState } from "react";
import { Button } from "@ui/button";
import { Badge } from "@ui/badge";
import { useToast } from "@ui/use-toast";
import { Link } from "react-router-dom";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@ui/dialog";

// Helper function to generate session name
function getSessionName(recording: Types.LogRecordingListItem): string {
  // Use custom name if provided
  if (recording.name) {
    return recording.name;
  }

  // Otherwise generate default name
  const date = new Date(recording.start_ts);
  const dateStr = date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  const timeStr = date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
  const baseName = `${recording.target.id.slice(0, 8)}_${dateStr}_${timeStr}`;

  // If it's a stack recording with specific services, show them
  if (recording.target.type === 'Stack' && recording.services && recording.services.length > 0) {
    const serviceList = recording.services.length <= 2
      ? recording.services.join(', ')
      : `${recording.services.slice(0, 2).join(', ')}...+${recording.services.length - 2}`;
    return `${baseName} (${serviceList})`;
  }

  return baseName;
}

export default function LogRecordingsPage() {
  useSetTitle("Log Recordings");
  const { toast } = useToast();
  const [search, setSearch] = useState("");
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [recordingToDelete, setRecordingToDelete] = useState<Types.LogRecordingListItem | null>(null);

  // Fetch all recordings without filter to show all
  const { data: recordings, refetch } = useRead("ListLogRecordings", {
    active_only: false,
  });


  // Mutations
  const { mutate: stopRecording } = useWrite("StopLogRecording", {
    onSuccess: () => {
      toast({ title: "Recording stopped" });
      refetch();
    },
  });

  const { mutate: deleteRecording } = useWrite("DeleteLogRecording", {
    onSuccess: () => {
      toast({ title: "Recording deleted" });
      refetch();
      setDeleteDialogOpen(false);
      setRecordingToDelete(null);
    },
  });


  // Filter recordings based on search
  const filtered = (recordings || []).filter(recording => {
    const searchLower = search.toLowerCase();
    // Search by resource ID or type
    return (
      recording.target.id.toLowerCase().includes(searchLower) ||
      recording.target.type.toLowerCase().includes(searchLower) ||
      recording.username?.toLowerCase().includes(searchLower)
    );
  });


  const handleDownload = (recording: Types.LogRecordingListItem) => {
    // First fetch the logs
    fetch(`/read/GetRecordedLogs`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ id: recording.id }),
    })
      .then(res => res.json())
      .then(logs => {
        if (logs) {
          const blob = new Blob([logs], { type: "text/plain" });
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;
          a.download = `log-recording-${recording.id}-${new Date(recording.start_ts).toISOString()}.log`;
          a.click();
          URL.revokeObjectURL(url);
        }
      });
  };


  return (
    <Page
      icon={<Film className="w-8" />}
      title="Log Recordings"
      subtitle={
        <div className="text-muted-foreground">
          Manage and view recorded logs from your resources.
        </div>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap gap-4 items-center justify-between">
          <div>{/* Empty div for left side */}</div>
          <div className="flex items-center gap-4 flex-wrap">
            <div className="relative">
              <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
              <Input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="search..."
                className="pl-8 w-[200px] lg:w-[300px]"
              />
            </div>
          </div>
        </div>

        {/* All Recordings Table */}
        <DataTable
          tableKey="log-recordings"
          data={filtered}
          defaultSort={[
            {
              id: "start_ts",
              desc: true,
            },
          ]}
          columns={[
            {
              accessorKey: "name",
              header: "Session",
              cell: ({ row }) => (
                <Link
                  to={`/log-recordings/${row.original.id}`}
                  className="flex items-center gap-2 text-sm hover:underline"
                >
                  <Film
                    className={`w-4 h-4 ${
                      row.original.status === Types.LogRecordingStatus.Recording
                        ? 'text-green-500'
                        : 'text-blue-500'
                    }`}
                  />
                  <span>{getSessionName(row.original)}</span>
                </Link>
              ),
            },
            {
              accessorKey: "target",
              header: "Resource",
              cell: ({ row }) => (
                <ResourceLink
                  type={row.original.target.type as any}
                  id={row.original.target.id}
                />
              ),
            },
            {
              accessorKey: "username",
              header: "User",
              cell: ({ row }) => row.original.username || row.original.user_id,
            },
            {
              accessorKey: "start_ts",
              header: ({ column }) => (
                <SortableHeader column={column} title="Started" />
              ),
              cell: ({ row }) =>
                new Date(row.original.start_ts).toLocaleString(),
            },
            {
              accessorKey: "duration",
              header: "Duration",
              cell: ({ row }) => {
                if (row.original.status === Types.LogRecordingStatus.Recording) {
                  // For active recordings, calculate from start to now
                  const start = new Date(row.original.start_ts).getTime();
                  const now = Date.now();
                  const hours = Math.floor((now - start) / 3600000);
                  const minutes = Math.floor(((now - start) % 3600000) / 60000);
                  return `${hours}h ${minutes}m`;
                } else if (row.original.expire_ts) {
                  // For completed recordings, use expire_ts
                  const start = new Date(row.original.start_ts).getTime();
                  const expire = new Date(row.original.expire_ts).getTime();
                  const hours = Math.floor((expire - start) / 3600000);
                  const minutes = Math.floor(((expire - start) % 3600000) / 60000);
                  return `${hours}h ${minutes}m`;
                }
                return "-";
              },
            },
            {
              accessorKey: "status",
              header: "Status",
              cell: ({ row }) => (
                row.original.status === Types.LogRecordingStatus.Recording ? (
                  <Badge variant="default" className="bg-red-500">
                    <Circle className="w-3 h-3 mr-1 animate-pulse" />
                    Recording
                  </Badge>
                ) : (
                  <Badge variant="secondary">Completed</Badge>
                )
              ),
            },
            {
              id: "actions",
              header: "Actions",
              cell: ({ row }) => (
                <div className="flex gap-2">
                  {row.original.status === Types.LogRecordingStatus.Recording && (
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => stopRecording({ id: row.original.id })}
                    >
                      <Square className="w-3 h-3 mr-1" />
                      Stop
                    </Button>
                  )}
                  {row.original.status !== Types.LogRecordingStatus.Recording && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDownload(row.original)}
                    >
                      <Download className="w-3 h-3 mr-1" />
                      Download
                    </Button>
                  )}
                  {row.original.status !== Types.LogRecordingStatus.Recording && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        setRecordingToDelete(row.original);
                        setDeleteDialogOpen(true);
                      }}
                    >
                      <Trash className="w-3 h-3" />
                    </Button>
                  )}
                </div>
              ),
            },
          ]}
        />
      </div>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>Delete Recording?</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this recording? This action cannot be undone.
              {recordingToDelete && (
                <div className="mt-3 p-2 bg-secondary rounded">
                  <div className="text-sm font-medium">{getSessionName(recordingToDelete)}</div>
                  <div className="text-xs text-muted-foreground mt-1">
                    Recorded on {new Date(recordingToDelete.start_ts).toLocaleString()}
                  </div>
                </div>
              )}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDeleteDialogOpen(false);
                setRecordingToDelete(null);
              }}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                if (recordingToDelete) {
                  deleteRecording({ id: recordingToDelete.id });
                }
              }}
            >
              Delete Recording
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Page>
  );
}