import { useRead } from "@/lib/hooks";
import { Group, Pagination } from "@mantine/core";

/**
 * The server side default page size on `List<Resource>` calls,
 * from the Core config `default_pagination_limit`.
 */
export function usePageSize() {
  const limit = useRead("GetCoreInfo", {}).data?.default_pagination_limit;
  // Fall back to the Core config default while loading.
  if (limit === undefined) return 30;
  // `limit: 0` disables pagination, so the page is never full.
  return limit === 0 ? Infinity : limit;
}

/**
 * Pagination controls for the paginated `List<Resource>` calls.
 * Only renders when needed, ie the results fill the page
 * or the user is past the first page.
 */
export default function ListPagination({
  page,
  setPage,
  count,
}: {
  page: number;
  setPage: (page: number) => void;
  count: number;
}) {
  const pageSize = usePageSize();
  if (count < pageSize && page === 0) return null;
  return (
    <Pagination.Root
      total={count >= pageSize ? page + 2 : page + 1}
      value={page + 1}
      onChange={(page) => setPage(page - 1)}
    >
      <Group gap="0.2rem" justify="center">
        <Pagination.First />
        <Pagination.Previous />
        <Pagination.Items />
        <Pagination.Next />
      </Group>
    </Pagination.Root>
  );
}
