import { Types } from "komodo_client";
import { UsableResource } from "@/resources";

export const fmtVersion = (version: Types.Version | undefined) => {
  if (!version) return "...";
  const { major, minor, patch } = version;
  if (major === 0 && minor === 0 && patch === 0) return "Latest";
  return `v${major}.${minor}.${patch}`;
};

export function fmtResourceType(type: UsableResource) {
  if (type === "ResourceSync") {
    return "Resource Sync";
  }
  return type;
}

export function fmtUtcOffset(tz: Types.IanaTimezone): string {
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

export function fmtPortMount(port: Types.Port) {
  return `${port.IP ? port.IP + ":" : ""}${port.PublicPort ?? "NONE"}:${port.PrivatePort ?? "NONE"}${port.Type ? "/" + port.Type : ""}`;
}

export function fmtMaintenanceWindowTime(window: Types.MaintenanceWindow) {
  const hours = window.hour!.toString().padStart(2, "0");
  const minutes = window.minute!.toString().padStart(2, "0");
  return `${hours}:${minutes} ${window.timezone ? `(${window.timezone})` : ""}`;
}
