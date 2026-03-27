import { Group, GroupProps, Text } from "@mantine/core";

export interface DockerOptionsProps extends Omit<GroupProps, "children"> {
  options: [string, string][] | undefined;
}

export default function DockerOptions({
  options,
  ...props
}: DockerOptionsProps) {
  return (
    <Group gap="sm" {...props}>
      {options?.map(([key, value]) => (
        <Group key={key} gap="0" bdrs="sm" bg="accent" px="xs" py="0.2rem">
          <Text fz="sm" c="dimmed">
            {key}
          </Text>
          <Text fz="sm" c="dimmed">
            =
          </Text>
          <Text
            fz="sm"
            fw="bolder"
            maw={200}
            className="text-ellipsis"
            style={{ textWrap: "nowrap" }}
          >
            {value}
          </Text>
        </Group>
      ))}
    </Group>
  );
}
