import Section from "@/ui/section";
import { useStatsGranularity } from "../hooks";
import { useState } from "react";

export default function ServerHistoricalStats({ id }: { id: string }) {
  const [interval, setInterval] = useStatsGranularity();
  const [showHistorical, setShowHistorical] = useState(false);
  return <Section title="Historical">{/* CHARTS */}</Section>;
}
