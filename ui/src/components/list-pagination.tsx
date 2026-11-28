import { useRead } from "@/lib/hooks";
import {
  ListPaginationProps,
  ListPagination as MoghListPagination,
} from "mogh_ui";

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
export default function ListPagination(
  props: Omit<ListPaginationProps, "pageSize">,
) {
  const pageSize = usePageSize();
  return <MoghListPagination {...props} pageSize={pageSize} />;
}
