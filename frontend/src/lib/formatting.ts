import { Types } from "komodo_client";

export const fmt_date = (d: Date) => {
  const hours = d.getHours();
  const minutes = d.getMinutes();
  return `${fmt_month(d.getMonth())} ${d.getDate()} ${
    hours > 9 ? hours : "0" + hours
  }:${minutes > 9 ? minutes : "0" + minutes}`;
};

export const fmt_utc_date = (d: Date) => {
  const hours = d.getUTCHours();
  const minutes = d.getUTCMinutes();
  return `${fmt_month(d.getUTCMonth())} ${d.getUTCDate()} ${
    hours > 9 ? hours : "0" + hours
  }:${minutes > 9 ? minutes : "0" + minutes}`;
};

const fmt_month = (month: number) => {
  switch (month) {
    case 0:
      return "Jan";
    case 1:
      return "Feb";
    case 2:
      return "Mar";
    case 3:
      return "Apr";
    case 4:
      return "May";
    case 5:
      return "Jun";
    case 6:
      return "Jul";
    case 7:
      return "Aug";
    case 8:
      return "Sep";
    case 9:
      return "Oct";
    case 10:
      return "Nov";
    case 11:
      return "Dec";
  }
};

export const fmt_date_with_minutes = (d: Date) => {
  // return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`;
  return d.toLocaleString();
};

export const fmt_version = (version: Types.Version | undefined) => {
  if (!version) return "...";
  const { major, minor, patch } = version;
  if (major === 0 && minor === 0 && patch === 0) return "Latest";
  return `v${major}.${minor}.${patch}`;
};

export const fmt_duration = (start_ts: number, end_ts: number) => {
  const start = new Date(start_ts);
  const end = new Date(end_ts);
  const durr = end.getTime() - start.getTime();
  const seconds = durr / 1000;
  const minutes = Math.floor(seconds / 60);
  const remaining_seconds = seconds % 60;
  return `${
    minutes > 0 ? `${minutes} minute${minutes > 1 ? "s" : ""} ` : ""
  }${remaining_seconds.toFixed(minutes > 0 ? 0 : 1)} seconds`;
};

export const fmt_operation = (operation: Types.Operation) => {
  return operation.match(/[A-Z][a-z]+|[0-9]+/g)?.join(" ")!;
};

export const fmt_upper_camelcase = (input: string) => {
  return input.match(/[A-Z][a-z]+|[0-9]+/g)?.join(" ")!;
};

/// list_all_items => List All Items
export function snake_case_to_upper_space_case(snake: string) {
  if (snake.length === 0) return "";
  return snake
    .split("_")
    .map((item) => item[0].toUpperCase() + item.slice(1))
    .join(" ");
}

const BYTES_PER_MB = 1e6;
const BYTES_PER_GB = BYTES_PER_MB * 1000;

export function format_size_bytes(size_bytes: number) {
  if (size_bytes > BYTES_PER_GB) {
    return `${(size_bytes / BYTES_PER_GB).toFixed(1)} GB`;
  } else {
    return `${(size_bytes / BYTES_PER_MB).toFixed(1)} MB`;
  }
}

export function fmt_utc_offset(tz: Types.IanaTimezone): string {
  switch (tz) {
    case Types.IanaTimezone.EtcGmtMinus12:
      return "UTC-12:00";
    case Types.IanaTimezone.PacificPagoPago:
      return "UTC-11:00";
    case Types.IanaTimezone.PacificHonolulu:
      return "UTC-10:00";
    case Types.IanaTimezone.PacificMarquesas:
      return "UTC-09:30";
    case Types.IanaTimezone.AmericaAnchorage:
      return "UTC-09:00";
    case Types.IanaTimezone.AmericaLosAngeles:
      return "UTC-08:00";
    case Types.IanaTimezone.AmericaDenver:
      return "UTC-07:00";
    case Types.IanaTimezone.AmericaChicago:
      return "UTC-06:00";
    case Types.IanaTimezone.AmericaNewYork:
      return "UTC-05:00";
    case Types.IanaTimezone.AmericaHalifax:
      return "UTC-04:00";
    case Types.IanaTimezone.AmericaStJohns:
      return "UTC-03:30";
    case Types.IanaTimezone.AmericaSaoPaulo:
      return "UTC-03:00";
    case Types.IanaTimezone.AmericaNoronha:
      return "UTC-02:00";
    case Types.IanaTimezone.AtlanticAzores:
      return "UTC-01:00";
    case Types.IanaTimezone.EtcUtc:
      return "UTC+00:00";
    case Types.IanaTimezone.EuropeBerlin:
      return "UTC+01:00";
    case Types.IanaTimezone.EuropeBucharest:
      return "UTC+02:00";
    case Types.IanaTimezone.EuropeMoscow:
      return "UTC+03:00";
    case Types.IanaTimezone.AsiaTehran:
      return "UTC+03:30";
    case Types.IanaTimezone.AsiaDubai:
      return "UTC+04:00";
    case Types.IanaTimezone.AsiaKabul:
      return "UTC+04:30";
    case Types.IanaTimezone.AsiaKarachi:
      return "UTC+05:00";
    case Types.IanaTimezone.AsiaKolkata:
      return "UTC+05:30";
    case Types.IanaTimezone.AsiaKathmandu:
      return "UTC+05:45";
    case Types.IanaTimezone.AsiaDhaka:
      return "UTC+06:00";
    case Types.IanaTimezone.AsiaYangon:
      return "UTC+06:30";
    case Types.IanaTimezone.AsiaBangkok:
      return "UTC+07:00";
    case Types.IanaTimezone.AsiaShanghai:
      return "UTC+08:00";
    case Types.IanaTimezone.AustraliaEucla:
      return "UTC+08:45";
    case Types.IanaTimezone.AsiaTokyo:
      return "UTC+09:00";
    case Types.IanaTimezone.AustraliaAdelaide:
      return "UTC+09:30";
    case Types.IanaTimezone.AustraliaSydney:
      return "UTC+10:00";
    case Types.IanaTimezone.AustraliaLordHowe:
      return "UTC+10:30";
    case Types.IanaTimezone.PacificPortMoresby:
      return "UTC+11:00";
    case Types.IanaTimezone.PacificAuckland:
      return "UTC+12:00";
    case Types.IanaTimezone.PacificChatham:
      return "UTC+12:45";
    case Types.IanaTimezone.PacificTongatapu:
      return "UTC+13:00";
    case Types.IanaTimezone.PacificKiritimati:
      return "UTC+14:00";
  }
}
