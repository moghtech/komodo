import { useState, useMemo } from "react";
import { Types } from "komodo_client";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { ToggleGroup, ToggleGroupItem } from "@ui/toggle-group";
import { Label } from "@ui/label";
import { useWrite } from "@lib/hooks";
import { useToast } from "@ui/use-toast";

type DurationUnit = "hours" | "days" | "infinite";

export const RecordingDialog = ({
  open,
  onOpenChange,
  target,
  services = [],
  onRecordingStarted,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: Types.ResourceTarget;
  services?: string[];
  onRecordingStarted: (recording: Types.LogRecording) => void;
}) => {
  const { toast } = useToast();
  const [name, setName] = useState("");
  const [durationValue, setDurationValue] = useState<number>(1);
  const [durationUnit, setDurationUnit] = useState<DurationUnit>("hours");

  // Generate default name placeholder
  const placeholderName = useMemo(() => {
    const date = new Date();
    const dateStr = date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    const timeStr = date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    const baseName = `${target.id.slice(0, 8)}_${dateStr}_${timeStr}`;

    if (services && services.length > 0) {
      const serviceList = services.length <= 2
        ? services.join(', ')
        : `${services.slice(0, 2).join(', ')}...+${services.length - 2}`;
      return `${baseName} (${serviceList})`;
    }

    return baseName;
  }, [target, services]);

  const { mutate: startRecording, isPending } = useWrite("StartLogRecording", {
    onSuccess: (recording) => {
      const durationText = durationUnit === "infinite"
        ? "until manually stopped"
        : `${durationValue} ${durationUnit}`;

      toast({
        title: "Recording started",
        description: `Logs will be recorded for ${durationText}`,
      });
      onRecordingStarted(recording);
      onOpenChange(false);
    },
    onError: (error: any) => {
      toast({
        title: "Failed to start recording",
        description: error?.message || "An error occurred",
        variant: "destructive",
      });
    },
  });

  const handleStart = () => {
    let durationDays: number | undefined = undefined;

    if (durationUnit === "hours") {
      durationDays = durationValue / 24;
    } else if (durationUnit === "days") {
      durationDays = durationValue;
    }
    // durationDays stays undefined for "infinite"

    startRecording({
      target,
      name: name || undefined,
      duration_days: durationDays,
      services,
    });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Start Log Recording</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          {/* Name field */}
          <div className="space-y-2">
            <Label htmlFor="recording-name">Session Name</Label>
            <Input
              id="recording-name"
              placeholder={placeholderName}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              Optional - defaults to auto-generated name if not specified
            </p>
          </div>

          {/* Duration selector */}
          <div className="space-y-2">
            <Label>Recording Duration</Label>
            <div className="flex items-center gap-2">
              <Input
                type="number"
                min="1"
                value={durationValue}
                onChange={(e) => setDurationValue(parseInt(e.target.value) || 1)}
                disabled={durationUnit === "infinite"}
                className="w-20"
              />
              <div className="border rounded-md">
                <ToggleGroup
                  type="single"
                  value={durationUnit}
                  onValueChange={(value) => value && setDurationUnit(value as DurationUnit)}
                >
                  <ToggleGroupItem value="hours" className="rounded-r-none">
                    Hours
                  </ToggleGroupItem>
                  <ToggleGroupItem value="days" className="rounded-none">
                    Days
                  </ToggleGroupItem>
                  <ToggleGroupItem value="infinite" className="rounded-l-none">
                    Infinite
                  </ToggleGroupItem>
                </ToggleGroup>
              </div>
            </div>
            {durationUnit === "infinite" && (
              <p className="text-sm text-muted-foreground">
                Recording will continue until manually stopped
              </p>
            )}
          </div>

          {services && services.length > 0 && (
            <div className="space-y-1">
              <Label>Services to record</Label>
              <div className="text-sm text-muted-foreground font-mono">
                {services.join(", ")}
              </div>
            </div>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleStart} disabled={isPending}>
            Start Recording
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};