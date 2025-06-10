import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { Switch } from "@ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Badge } from "@ui/badge";
import { DataTable, SortableHeader } from "@ui/data-table";
import { Types } from "komodo_client";
import { useState } from "react";
import {
  PlusCircle,
  Pen,
  Trash2,
  Clock,
  Calendar,
  CalendarDays,
} from "lucide-react";
import { TimezoneSelector } from "@components/util";

export const MaintenanceWindows = ({
  windows,
  onUpdate,
  disabled,
}: {
  windows: Types.MaintenanceWindow[];
  onUpdate: (windows: Types.MaintenanceWindow[]) => void;
  disabled: boolean;
}) => {
  const [isCreating, setIsCreating] = useState(false);
  const [editingWindow, setEditingWindow] = useState<
    [number, Types.MaintenanceWindow] | null
  >(null);

  const addWindow = (newWindow: Types.MaintenanceWindow) => {
    onUpdate([...windows, newWindow]);
    setIsCreating(false);
  };

  const updateWindow = (
    index: number,
    updatedWindow: Types.MaintenanceWindow
  ) => {
    onUpdate(windows.map((w, i) => (i === index ? updatedWindow : w)));
    setEditingWindow(null);
  };

  const deleteWindow = (index: number) => {
    onUpdate(windows.filter((_, i) => i !== index));
  };

  const toggleWindow = (index: number, enabled: boolean) => {
    onUpdate(windows.map((w, i) => (i === index ? { ...w, enabled } : w)));
  };

  return (
    <div className="space-y-4">
      {!disabled && (
        <Dialog open={isCreating} onOpenChange={setIsCreating}>
          <DialogTrigger asChild>
            <Button variant="secondary" className="flex items-center gap-2">
              <PlusCircle className="w-4 h-4" />
              Add Maintenance Window
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl">
            <MaintenanceWindowForm
              onSave={addWindow}
              onCancel={() => setIsCreating(false)}
            />
          </DialogContent>
        </Dialog>
      )}

      {windows.length > 0 && (
        <DataTable
          tableKey="maintenance-windows"
          data={windows}
          columns={[
            {
              accessorKey: "name",
              header: ({ column }) => (
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => (
                <div className="flex items-center gap-2">
                  <ScheduleIcon
                    scheduleType={
                      row.original.schedule_type ??
                      Types.MaintenanceScheduleType.Daily
                    }
                  />
                  <span className="font-medium">{row.original.name}</span>
                </div>
              ),
              size: 200,
            },
            {
              accessorKey: "schedule_type",
              header: ({ column }) => (
                <SortableHeader column={column} title="Schedule" />
              ),
              cell: ({ row }) => (
                <span className="text-sm">
                  <ScheduleDescription window={row.original} />
                </span>
              ),
              size: 150,
            },
            {
              accessorKey: "start_time",
              header: ({ column }) => (
                <SortableHeader column={column} title="Start Time" />
              ),
              cell: ({ row }) => (
                <span className="text-sm font-mono">
                  {formatTime(row.original)}
                </span>
              ),
              size: 180,
            },
            {
              accessorKey: "duration_minutes",
              header: ({ column }) => (
                <SortableHeader column={column} title="Duration" />
              ),
              cell: ({ row }) => (
                <span className="text-sm">
                  {row.original.duration_minutes} min
                </span>
              ),
              size: 100,
            },
            {
              accessorKey: "enabled",
              header: ({ column }) => (
                <SortableHeader column={column} title="Status" />
              ),
              cell: ({ row }) => (
                <div className="flex items-center gap-2">
                  <Badge
                    variant={row.original.enabled ? "default" : "secondary"}
                  >
                    {row.original.enabled ? "Enabled" : "Disabled"}
                  </Badge>
                  {!disabled && (
                    <Switch
                      checked={row.original.enabled}
                      onCheckedChange={(enabled) =>
                        toggleWindow(row.index, enabled)
                      }
                    />
                  )}
                </div>
              ),
              size: 120,
            },
            {
              id: "actions",
              header: "Actions",
              cell: ({ row }) =>
                !disabled && (
                  <div className="flex items-center gap-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        setEditingWindow([row.index, row.original])
                      }
                      className="h-8 w-8 p-0"
                    >
                      <Pen className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => deleteWindow(row.index)}
                      className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                ),
              size: 100,
            },
          ]}
        />
      )}

      {editingWindow && (
        <Dialog
          open={!!editingWindow}
          onOpenChange={() => setEditingWindow(null)}
        >
          <DialogContent className="max-w-2xl">
            <MaintenanceWindowForm
              initialData={editingWindow[1]}
              onSave={(window) => updateWindow(editingWindow[0], window)}
              onCancel={() => setEditingWindow(null)}
            />
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
};

const ScheduleIcon = ({
  scheduleType,
}: {
  scheduleType: Types.MaintenanceScheduleType;
}) => {
  switch (scheduleType) {
    case "Daily":
      return <Clock className="w-4 h-4" />;
    case "Weekly":
      return <Calendar className="w-4 h-4" />;
    case "OneTime":
      return <CalendarDays className="w-4 h-4" />;
    default:
      return <Clock className="w-4 h-4" />;
  }
};

const ScheduleDescription = ({
  window,
}: {
  window: Types.MaintenanceWindow;
}): string => {
  switch (window.schedule_type) {
    case "Daily":
      return "Daily";
    case "Weekly":
      return `Weekly (${window.day_of_week || "Monday"})`;
    case "OneTime":
      return `One-time (${window.date || "No date"})`;
    default:
      return "Unknown";
  }
};

const formatTime = (window: Types.MaintenanceWindow) => {
  const hours = window.hour!.toString().padStart(2, "0");
  const minutes = window.minute!.toString().padStart(2, "0");
  return `${hours}:${minutes} ${window.timezone ? `(${window.timezone})` : ""}`;
};

interface MaintenanceWindowFormProps {
  initialData?: Types.MaintenanceWindow;
  onSave: (window: Types.MaintenanceWindow) => void;
  onCancel: () => void;
}

const MaintenanceWindowForm = ({
  initialData,
  onSave,
  onCancel,
}: MaintenanceWindowFormProps) => {
  const [formData, setFormData] = useState<Types.MaintenanceWindow>(
    initialData || {
      name: "",
      description: "",
      schedule_type: Types.MaintenanceScheduleType.Daily,
      day_of_week: "",
      date: "",
      hour: 5,
      minute: 0,
      timezone: "",
      duration_minutes: 60,
      enabled: true,
    }
  );

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = "Name is required";
    }

    if (formData.hour! < 0 || formData.hour! > 23) {
      newErrors.hour = "Hour must be between 0 and 23";
    }

    if (formData.minute! < 0 || formData.minute! > 59) {
      newErrors.minute = "Minute must be between 0 and 59";
    }

    if (formData.duration_minutes <= 0) {
      newErrors.duration = "Duration must be greater than 0";
    }

    if (formData.schedule_type && formData.schedule_type === "OneTime") {
      const date = formData.date;
      if (!date || !/^\d{4}-\d{2}-\d{2}$/.test(date)) {
        newErrors.date = "Date must be in YYYY-MM-DD format";
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSave = () => {
    if (validate()) {
      onSave(formData);
    }
  };

  const updateScheduleType = (schedule_type: Types.MaintenanceScheduleType) => {
    setFormData((data) => ({
      ...data,
      schedule_type,
      day_of_week:
        schedule_type === Types.MaintenanceScheduleType.Weekly ? "Monday" : "",
      date:
        schedule_type === Types.MaintenanceScheduleType.OneTime
          ? new Date().toISOString().split("T")[0]
          : "",
    }));
  };

  return (
    <>
      <DialogHeader>
        <DialogTitle>
          {initialData
            ? "Edit Maintenance Window"
            : "Create Maintenance Window"}
        </DialogTitle>
      </DialogHeader>

      <div className="space-y-4">
        <div>
          <label className="text-sm font-medium">Name</label>
          <Input
            value={formData.name}
            onChange={(e) =>
              setFormData((data) => ({ ...data, name: e.target.value }))
            }
            placeholder="e.g., Daily Backup"
            className={errors.name ? "border-destructive" : ""}
          />
          {errors.name && (
            <p className="text-sm text-destructive mt-1">{errors.name}</p>
          )}
        </div>

        <div>
          <label className="text-sm font-medium">Schedule Type</label>
          <Select
            value={formData.schedule_type}
            onValueChange={(value: Types.MaintenanceScheduleType) =>
              updateScheduleType(value)
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {Object.values(Types.MaintenanceScheduleType).map(
                (schedule_type) => (
                  <SelectItem key={schedule_type} value={schedule_type}>
                    {schedule_type}
                  </SelectItem>
                )
              )}
            </SelectContent>
          </Select>
        </div>

        {formData.schedule_type === "Weekly" && (
          <div>
            <label className="text-sm font-medium">Day of Week</label>
            <Select
              value={formData.day_of_week || "Monday"}
              onValueChange={(value: Types.DayOfWeek) =>
                setFormData((data) => ({
                  ...data,
                  day_of_week: value,
                }))
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {Object.values(Types.DayOfWeek).map((day_of_week) => (
                  <SelectItem key={day_of_week} value={day_of_week}>
                    {day_of_week}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {formData.schedule_type === "OneTime" && (
          <div>
            <label className="text-sm font-medium">Date</label>
            <Input
              type="date"
              value={formData.date || new Date().toISOString().split("T")[0]}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  date: e.target.value,
                })
              }
              className={errors.date ? "border-destructive" : ""}
            />
            {errors.date && (
              <p className="text-sm text-destructive mt-1">{errors.date}</p>
            )}
          </div>
        )}

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-sm font-medium">Start Time</label>
            <Input
              type="time"
              value={`${formData.hour!.toString().padStart(2, "0")}:${formData.minute!.toString().padStart(2, "0")}`}
              onChange={(e) => {
                const [hour, minute] = e.target.value
                  .split(":")
                  .map((n) => parseInt(n) || 0);
                setFormData({
                  ...formData,
                  hour,
                  minute,
                });
              }}
              className={
                errors.hour || errors.minute ? "border-destructive" : ""
              }
            />
            {(errors.hour || errors.minute) && (
              <p className="text-sm text-destructive mt-1">
                {errors.hour || errors.minute}
              </p>
            )}
          </div>
          <div>
            <label className="text-sm font-medium">Timezone</label>
            <TimezoneSelector
              timezone={formData.timezone ?? ""}
              onChange={(timezone) =>
                setFormData((data) => ({ ...data, timezone }))
              }
              triggerClassName="w-full"
            />
          </div>
        </div>

        <div>
          <label className="text-sm font-medium">Duration (minutes)</label>
          <Input
            type="number"
            min={1}
            value={formData.duration_minutes}
            onChange={(e) =>
              setFormData((data) => ({
                ...data,
                duration_minutes: parseInt(e.target.value) || 60,
              }))
            }
            className={errors.duration ? "border-destructive" : ""}
          />
          {errors.duration && (
            <p className="text-sm text-destructive mt-1">{errors.duration}</p>
          )}
        </div>

        <div>
          <label className="text-sm font-medium">Description (optional)</label>
          <Input
            value={formData.description}
            onChange={(e) =>
              setFormData((data) => ({ ...data, description: e.target.value }))
            }
            placeholder="e.g., Automated backup process"
          />
        </div>
      </div>

      <DialogFooter>
        <Button variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={handleSave}>
          {initialData ? "Update" : "Create"}
        </Button>
      </DialogFooter>
    </>
  );
};
