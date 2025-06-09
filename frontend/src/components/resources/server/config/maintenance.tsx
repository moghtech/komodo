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

export interface MaintenanceWindowsProps {
  windows: Types.MaintenanceWindow[];
  onUpdate: (windows: Types.MaintenanceWindow[]) => void;
  disabled: boolean;
}

export const MaintenanceWindows = ({
  windows,
  onUpdate,
  disabled,
}: MaintenanceWindowsProps) => {
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
    onUpdate(windows.filter((_, i) => i === index));
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
                    scheduleType={row.original.schedule_type!.type}
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
                  <ScheduleDescription
                    scheduleType={row.original.schedule_type!}
                  />
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
  scheduleType: Types.MaintenanceScheduleType["type"];
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
  scheduleType,
}: {
  scheduleType: Types.MaintenanceScheduleType;
}): string => {
  switch (scheduleType.type) {
    case "Daily":
      return "Daily";
    case "Weekly":
      return `Weekly (${scheduleType.data?.day_of_week || "Monday"})`;
    case "OneTime":
      return `One-time (${scheduleType.data?.date || "No date"})`;
    default:
      return "Unknown";
  }
};

const formatTime = (window: Types.MaintenanceWindow) => {
  const hours = window.hour!.toString().padStart(2, "0");
  const minutes = window.minute!.toString().padStart(2, "0");
  // const timezoneDisplay = getTimezoneDisplay(time.timezone_offset_minutes);
  return `${hours}:${minutes} ${window.timezone ? `(${window.timezone})` : ""}`;
};

// const getTimezoneDisplay = (offsetMinutes: number): string => {
//   const timezoneMap: Record<number, string> = {
//     [-720]: "UTC-12",
//     [-660]: "UTC-11",
//     [-600]: "UTC-10 (Hawaii)",
//     [-540]: "UTC-9 (Alaska)",
//     [-480]: "UTC-8 (Pacific)",
//     [-420]: "UTC-7 (Mountain)",
//     [-360]: "UTC-6 (Central)",
//     [-300]: "UTC-5 (Eastern)",
//     [-240]: "UTC-4 (Atlantic)",
//     [-180]: "UTC-3",
//     [-120]: "UTC-2",
//     [-60]: "UTC-1",
//     [0]: "UTC+0 (GMT)",
//     [60]: "UTC+1 (CET)",
//     [120]: "UTC+2 (EET)",
//     [180]: "UTC+3 (Moscow)",
//     [240]: "UTC+4",
//     [300]: "UTC+5",
//     [330]: "UTC+5:30 (India)",
//     [360]: "UTC+6",
//     [420]: "UTC+7",
//     [480]: "UTC+8 (China)",
//     [540]: "UTC+9 (Japan)",
//     [570]: "UTC+9:30",
//     [600]: "UTC+10 (Australia)",
//     [660]: "UTC+11",
//     [720]: "UTC+12 (New Zealand)",
//   };
//   return timezoneMap[offsetMinutes] || getTimezoneValue(offsetMinutes);
// };

interface MaintenanceWindowFormProps {
  initialData?: Types.MaintenanceWindow;
  onSave: (window: Types.MaintenanceWindow) => void;
  onCancel: () => void;
}

// Helper functions for timezone conversion
// const getTimezoneOffset = (timezoneValue: string): number => {
//   const match = timezoneValue.match(/UTC([+-]?)(\d+(?:\.\d+)?)/);
//   if (!match) return 0;

//   const sign = match[1] === "-" ? -1 : 1;
//   const hours = parseFloat(match[2]);
//   return sign * hours * 60; // Convert to minutes
// };

// const getTimezoneValue = (offsetMinutes: number): string => {
//   if (offsetMinutes === 0) return "UTC+0";

//   const hours = Math.abs(offsetMinutes) / 60;
//   const sign = offsetMinutes >= 0 ? "+" : "-";

//   if (hours === Math.floor(hours)) {
//     return `UTC${sign}${Math.floor(hours)}`;
//   } else {
//     const wholeHours = Math.floor(hours);
//     const minutes = (hours - wholeHours) * 60;
//     return `UTC${sign}${wholeHours}.${minutes === 30 ? "5" : "0"}`;
//   }
// };

const MaintenanceWindowForm = ({
  initialData,
  onSave,
  onCancel,
}: MaintenanceWindowFormProps) => {
  const [formData, setFormData] = useState<Types.MaintenanceWindow>(
    initialData || {
      name: "",
      description: "",
      schedule_type: { type: "Daily", data: {} },
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

    if (formData.schedule_type && formData.schedule_type.type === "OneTime") {
      const date = formData.schedule_type.data.date;
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

  const updateScheduleType = (type: "Daily" | "Weekly" | "OneTime") => {
    let scheduleType: Types.MaintenanceScheduleType;

    switch (type) {
      case "Daily":
        scheduleType = { type: "Daily", data: {} };
        break;
      case "Weekly":
        scheduleType = {
          type: "Weekly",
          data: { day_of_week: "Monday" as Types.DayOfWeek },
        };
        break;
      case "OneTime":
        scheduleType = {
          type: "OneTime",
          data: { date: new Date().toISOString().split("T")[0] },
        };
        break;
      default:
        scheduleType = { type: "Daily", data: {} };
    }

    setFormData({ ...formData, schedule_type: scheduleType });
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
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
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
            value={formData.schedule_type!.type}
            onValueChange={(value: "Daily" | "Weekly" | "OneTime") =>
              updateScheduleType(value)
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Daily">Daily</SelectItem>
              <SelectItem value="Weekly">Weekly</SelectItem>
              <SelectItem value="OneTime">One-time</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {formData.schedule_type!.type === "Weekly" && (
          <div>
            <label className="text-sm font-medium">Day of Week</label>
            <Select
              value={
                formData.schedule_type!.type === "Weekly"
                  ? formData.schedule_type!.data.day_of_week || "Monday"
                  : "Monday"
              }
              onValueChange={(value: Types.DayOfWeek) =>
                setFormData({
                  ...formData,
                  schedule_type: {
                    type: "Weekly",
                    data: { day_of_week: value },
                  },
                })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Monday">Monday</SelectItem>
                <SelectItem value="Tuesday">Tuesday</SelectItem>
                <SelectItem value="Wednesday">Wednesday</SelectItem>
                <SelectItem value="Thursday">Thursday</SelectItem>
                <SelectItem value="Friday">Friday</SelectItem>
                <SelectItem value="Saturday">Saturday</SelectItem>
                <SelectItem value="Sunday">Sunday</SelectItem>
              </SelectContent>
            </Select>
          </div>
        )}

        {formData.schedule_type!.type === "OneTime" && (
          <div>
            <label className="text-sm font-medium">Date</label>
            <Input
              type="date"
              value={
                formData.schedule_type!.type === "OneTime"
                  ? formData.schedule_type!.data.date ||
                    new Date().toISOString().split("T")[0]
                  : ""
              }
              onChange={(e) =>
                setFormData({
                  ...formData,
                  schedule_type: {
                    type: "OneTime",
                    data: { date: e.target.value },
                  },
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
            {/* <label className="text-sm font-medium">Timezone</label>
            <Select
              value={getTimezoneValue(
                formData.start_time.timezone_offset_minutes
              )}
              onValueChange={(value) => {
                const offsetMinutes = getTimezoneOffset(value);
                setFormData({
                  ...formData,
                  start_time: {
                    ...formData.start_time,
                    timezone_offset_minutes: offsetMinutes,
                  },
                });
              }}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="UTC-12">UTC-12 (Baker Island)</SelectItem>
                <SelectItem value="UTC-11">UTC-11 (American Samoa)</SelectItem>
                <SelectItem value="UTC-10">UTC-10 (Hawaii)</SelectItem>
                <SelectItem value="UTC-9">UTC-9 (Alaska)</SelectItem>
                <SelectItem value="UTC-8">UTC-8 (Pacific Time)</SelectItem>
                <SelectItem value="UTC-7">UTC-7 (Mountain Time)</SelectItem>
                <SelectItem value="UTC-6">UTC-6 (Central Time)</SelectItem>
                <SelectItem value="UTC-5">UTC-5 (Eastern Time)</SelectItem>
                <SelectItem value="UTC-4">UTC-4 (Atlantic Time)</SelectItem>
                <SelectItem value="UTC-3">UTC-3 (Argentina)</SelectItem>
                <SelectItem value="UTC-2">UTC-2 (Mid-Atlantic)</SelectItem>
                <SelectItem value="UTC-1">UTC-1 (Azores)</SelectItem>
                <SelectItem value="UTC+0">UTC+0 (London/GMT)</SelectItem>
                <SelectItem value="UTC+1">UTC+1 (Central Europe)</SelectItem>
                <SelectItem value="UTC+2">UTC+2 (Eastern Europe)</SelectItem>
                <SelectItem value="UTC+3">UTC+3 (Moscow)</SelectItem>
                <SelectItem value="UTC+4">UTC+4 (Dubai)</SelectItem>
                <SelectItem value="UTC+5">UTC+5 (Pakistan)</SelectItem>
                <SelectItem value="UTC+5.5">UTC+5:30 (India)</SelectItem>
                <SelectItem value="UTC+6">UTC+6 (Bangladesh)</SelectItem>
                <SelectItem value="UTC+7">UTC+7 (Thailand)</SelectItem>
                <SelectItem value="UTC+8">UTC+8 (China/Singapore)</SelectItem>
                <SelectItem value="UTC+9">UTC+9 (Japan)</SelectItem>
                <SelectItem value="UTC+9.5">
                  UTC+9:30 (Australia Central)
                </SelectItem>
                <SelectItem value="UTC+10">UTC+10 (Australia East)</SelectItem>
                <SelectItem value="UTC+11">UTC+11 (Solomon Islands)</SelectItem>
                <SelectItem value="UTC+12">UTC+12 (New Zealand)</SelectItem>
              </SelectContent>
            </Select> */}
          </div>
        </div>

        <div>
          <label className="text-sm font-medium">Duration (minutes)</label>
          <Input
            type="number"
            min={1}
            value={formData.duration_minutes}
            onChange={(e) =>
              setFormData({
                ...formData,
                duration_minutes: parseInt(e.target.value) || 60,
              })
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
              setFormData({ ...formData, description: e.target.value })
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
