import { fmtUtcOffset } from "@/lib/formatting";
import { useRead } from "@/lib/hooks";
import { Select, SelectProps } from "@mantine/core";
import { Types } from "komodo_client";

export interface TimezoneSelectorProps extends Omit<SelectProps, "onChange"> {
  timezone: string;
  onChange: (timezone: "" | Types.IanaTimezone) => void;
}

export default function TimezoneSelector({
  timezone,
  onChange,
  disabled,
  ...selectProps
}: TimezoneSelectorProps) {
  const coreTz = useRead("GetCoreInfo", {}).data?.timezone || "Core TZ";
  return (
    <Select
      w={{ base: "85%", lg: 400 }}
      {...selectProps}
      value={timezone}
      onChange={(tz) => {
        if (!tz) return;
        onChange(tz as Types.IanaTimezone);
      }}
      data={[
        { value: "", label: `Default (${coreTz})` },
        ...Object.values(Types.IanaTimezone).map((tz) => ({
          value: tz,
          label: `${tz} (${fmtUtcOffset(tz)})`,
        })),
      ]}
      searchable
    />
  );
}
