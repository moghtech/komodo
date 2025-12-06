import { Types } from "komodo_client";

export type UsableResource = Exclude<Types.ResourceTarget["type"], "System">;

type IdComponent = React.FC<{ id: string }>;
type OptionalIdComponent = React.FC<{ id?: string }>;

export interface RequiredResourceComponents {
  list_item: (id: string) => Types.ResourceListItem<unknown> | undefined;
  resource_links: (
    resource: Types.Resource<unknown, unknown>
  ) => Array<string> | undefined;

  Description: React.FC;

  /** Header for individual resource pages */
  ResourcePageHeader: IdComponent;

  /** Summary card for use in dashboard */
  Dashboard: React.FC;

  /** New resource button / dialog */
  New: React.FC<{
    swarm_id?: string;
    server_id?: string;
    builder_id?: string;
    build_id?: string;
  }>;

  /** A table component to view resource list */
  Table: React.FC<{ resources: Types.ResourceListItem<unknown>[] }>;

  /** Dropdown menu to trigger group actions for selected resources */
  GroupActions: React.FC;

  /** Icon for the component */
  Icon: OptionalIdComponent;
  BigIcon: OptionalIdComponent;

  State: IdComponent;

  /** status metrics, like deployment state / status */
  Status: { [status: string]: IdComponent };

  /**
   * Some config items shown in header, like deployment server /image
   * or build repo / branch
   */
  Info: { [info: string]: IdComponent };

  /** Action buttons */
  Actions: { [action: string]: IdComponent };

  /** Resource specific sections */
  Page: { [section: string]: IdComponent };

  /** Config component for resource */
  Config: IdComponent;

  /** Danger zone for resource, containing eg rename, delete */
  DangerZone: IdComponent;
}
