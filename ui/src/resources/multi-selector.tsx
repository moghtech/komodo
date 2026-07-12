import { useRead } from "@/lib/hooks";
import { keepPreviousData } from "@tanstack/react-query";
import { MultiSelect } from "@mantine/core";
import { Types } from "komodo_client";
import { useDebounce } from "mogh_ui";
import { useMemo, useState } from "react";
import { UsableResource } from ".";
import { fmtResourceType } from "@/lib/formatting";

/**
 * Multi select over resource names, using server side term search
 * with `limit: 10` instead of pulling the full resource list.
 */
export default function ResourceMultiSelector({
  type,
  value,
  onChange,
  templates = Types.TemplatesQueryBehavior.Exclude,
  placeholder,
}: {
  type: UsableResource;
  /** The selected resource names. */
  value: string[];
  onChange: (names: string[]) => void;
  templates?: Types.TemplatesQueryBehavior;
  placeholder?: string;
}) {
  const [search, setSearch] = useState("");
  const debouncedSearch = useDebounce(search, 200);
  const terms = useMemo(
    () =>
      debouncedSearch
        .toLowerCase()
        .split(" ")
        .map((term) => term.trim())
        .filter((term) => term),
    [debouncedSearch],
  );

  const resources =
    useRead(
      `List${type}s`,
      { query: { templates, terms }, limit: 10 },
      // Keep the previous options visible while the
      // next search fetches, to prevent flashing.
      { placeholderData: keepPreviousData },
    ).data ?? [];

  // Keep the selected names in the options,
  // even when they don't match the current search.
  const data = useMemo(() => {
    const names = resources.map((resource) => resource.name);
    for (const name of value) {
      if (!names.includes(name)) {
        names.push(name);
      }
    }
    return names;
  }, [resources, value]);

  return (
    <MultiSelect
      placeholder={placeholder ?? `Filter by ${fmtResourceType(type)}s`}
      value={value}
      onChange={onChange}
      data={data}
      searchable
      clearable
      searchValue={search}
      onSearchChange={setSearch}
      // The results are already filtered server side,
      // client side filtering would reapply single-term matching.
      filter={({ options }) => options}
    />
  );
}
