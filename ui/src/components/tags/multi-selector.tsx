import { useRead } from "@/lib/hooks";
import { filterBySplit } from "mogh_ui";
import {
  Box,
  ComboboxItem,
  Group,
  MultiSelect,
  MultiSelectProps,
  Text,
} from "@mantine/core";
import { useMemo } from "react";

export interface TagMultiSelectorProps extends Omit<
  MultiSelectProps,
  "data" | "value" | "onChange"
> {
  /** The selected tag ids (or names with `useName`). */
  value: string[];
  onChange: (tags: string[]) => void;
  /** Use tag names as values instead of ids. */
  useName?: boolean;
}

export default function TagMultiSelector({
  value,
  onChange,
  useName,
  ...props
}: TagMultiSelectorProps) {
  const tags = useRead("ListTags", {}).data;

  const { data, colors } = useMemo(() => {
    const colors: { [value: string]: string | undefined } = {};
    const data =
      tags?.map((tag) => {
        const value = useName ? tag.name : tag._id?.$oid!;
        colors[value] = tag.color;
        return { value, label: tag.name };
      }) ?? [];
    return { data, colors };
  }, [tags, useName]);

  return (
    <MultiSelect
      placeholder={value.length === 0 ? "Select Tags" : undefined}
      value={value}
      onChange={onChange}
      data={data}
      searchable
      clearable
      nothingFoundMessage="No results."
      filter={({ options, search }) =>
        filterBySplit(
          options as ComboboxItem[],
          search,
          (option) => option.label,
        )
      }
      renderOption={({ option }) => (
        <Group justify="space-between" w="100%">
          <Text>{option.label}</Text>
          <Box
            w={25}
            h={25}
            bg={
              colors[option.value]
                ? `Tag${colors[option.value]}.9`
                : "TagSlate.9"
            }
            bdrs="md"
          />
        </Group>
      )}
      {...props}
    />
  );
}
