import { Skeleton, Stack } from "@mantine/core";

export default function TableSkeleton({
  rows = 6,
  rowHeight = "3.5vh",
}: {
  rows?: number;
  rowHeight?: string | number;
}) {
  return (
    <Stack gap="xs">
      {Array.from({ length: rows }).map((_, i) => (
        <Skeleton key={i} height={rowHeight} />
      ))}
    </Stack>
  );
}
