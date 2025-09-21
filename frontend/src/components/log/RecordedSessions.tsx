import { useState } from "react";
import { Types } from "komodo_client";
import { useRead, useWrite } from "@lib/hooks";
import { Button } from "@ui/button";
import { Badge } from "@ui/badge";
import { Download, Trash2, Square, Eye } from "lucide-react";
import { Log } from "./LiveLog";
import { format } from "date-fns";
import { useToast } from "@ui/use-toast";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@ui/dialog";

export const RecordedSessions = ({
  target,
}: {
  target: Types.ResourceTarget;
}) => {
  const { toast } = useToast();
  const [selectedRecording, setSelectedRecording] = useState<string | null>(null);
  const [viewDialogOpen, setViewDialogOpen] = useState(false);

  const { data: recordings, refetch } = useRead("ListLogRecordings", {
    target,
    active_only: false,
  });

  const { data: recordedLogs } = useRead(
    "GetRecordedLogs",
    {
      recording_id: selectedRecording || "",
      tail: 1000,
      timestamps: false,
    },
    {
      enabled: !!selectedRecording,
    }
  );

  const { mutate: deleteRecording } = useWrite("DeleteLogRecording", {
    onSuccess: () => {
      toast({ title: "Recording deleted" });
      setSelectedRecording(null);
      setViewDialogOpen(false);
      refetch();
    },
  });

  const { mutate: stopRecording } = useWrite("StopLogRecording", {
    onSuccess: () => {
      toast({ title: "Recording stopped" });
      refetch();
    },
  });

  const formatDate = (ts: number) => {
    return format(new Date(ts), "MMM d, HH:mm");
  };

  const calculateDuration = (start: number, end?: number | null) => {
    const endTime = end || Date.now();
    const hours = Math.floor((endTime - start) / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days} day${days > 1 ? 's' : ''}`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''}`;
    return "< 1 hour";
  };

  const downloadLogs = (recordingId: string) => {
    if (!recordedLogs || selectedRecording !== recordingId) return;

    const combined = `${recordedLogs.stdout}\n\n--- STDERR ---\n${recordedLogs.stderr}`;
    const blob = new Blob([combined], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `logs-${recordingId}-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const viewRecording = (id: string) => {
    setSelectedRecording(id);
    setViewDialogOpen(true);
  };

  if (!recordings || recordings.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        No recorded sessions. Click "Record" to start capturing logs.
      </div>
    );
  }

  return (
    <>
      <div className="space-y-4">
        <div className="border rounded-lg">
          <table className="w-full">
            <thead>
              <tr className="border-b">
                <th className="text-left p-3 font-medium">Date</th>
                <th className="text-left p-3 font-medium">Duration</th>
                <th className="text-left p-3 font-medium">Status</th>
                <th className="text-right p-3 font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {recordings.map((recording) => (
                <tr key={recording.id} className="border-b hover:bg-accent/50">
                  <td className="p-3">{formatDate(recording.start_ts)}</td>
                  <td className="p-3">
                    {calculateDuration(recording.start_ts, recording.expire_ts)}
                  </td>
                  <td className="p-3">
                    {recording.status === Types.LogRecordingStatus.Recording ? (
                      <Badge variant="destructive" className="text-xs">Recording</Badge>
                    ) : recording.status === Types.LogRecordingStatus.Stopped ? (
                      <Badge variant="secondary" className="text-xs">Complete</Badge>
                    ) : (
                      <Badge variant="outline" className="text-xs">Expired</Badge>
                    )}
                  </td>
                  <td className="p-3">
                    <div className="flex items-center justify-end gap-1">
                      {recording.status === Types.LogRecordingStatus.Recording ? (
                        <Button
                          size="sm"
                          variant="outline"
                          className="h-7"
                          onClick={() => stopRecording({ id: recording.id })}
                        >
                          <Square className="w-3 h-3 mr-1" />
                          Stop
                        </Button>
                      ) : null}
                      <Button
                        size="sm"
                        variant="outline"
                        className="h-7"
                        onClick={() => viewRecording(recording.id)}
                      >
                        <Eye className="w-3 h-3 mr-1" />
                        View
                      </Button>
                      {selectedRecording === recording.id && recordedLogs && (
                        <Button
                          size="sm"
                          variant="outline"
                          className="h-7"
                          onClick={() => downloadLogs(recording.id)}
                        >
                          <Download className="w-3 h-3 mr-1" />
                          Download
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 w-7 p-0"
                        onClick={() => deleteRecording({ id: recording.id })}
                      >
                        <Trash2 className="w-3 h-3" />
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Log viewer dialog */}
      <Dialog open={viewDialogOpen} onOpenChange={setViewDialogOpen}>
        <DialogContent className="max-w-4xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle className="flex items-center justify-between">
              <span>Recorded Logs</span>
              {selectedRecording && recordedLogs && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => downloadLogs(selectedRecording)}
                >
                  <Download className="w-4 h-4 mr-2" />
                  Download
                </Button>
              )}
            </DialogTitle>
          </DialogHeader>
          <div className="h-[60vh] overflow-auto">
            <Log
              log={recordedLogs}
              stream="stdout"
            />
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
};