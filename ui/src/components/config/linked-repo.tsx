import ResourceLink from "@/resources/link";
import ResourceSelector from "@/resources/selector";
import { ConfigItem } from "@/ui/config/item";
import { Group } from "@mantine/core";

export interface LinkedRepoProps {
  linkedRepo: string | undefined;
  repoLinked: boolean;
  set: (update: {
    /// KEEP as snake case, these are being passed to backend.
    linked_repo: string;
    // Set other props back to default.
    git_provider: string;
    git_account: string;
    git_https: boolean;
    repo: string;
    branch: string;
    commit: string;
  }) => void;
  disabled: boolean;
}

export default function LinkedRepo({
  linkedRepo,
  repoLinked,
  set,
  disabled,
}: LinkedRepoProps) {
  return (
    <ConfigItem
      label={
        linkedRepo ? (
          <Group fz="h3" fw="bold">
            Repo:
            <ResourceLink
              type="Repo"
              id={linkedRepo}
              fz="h3"
              iconSize="1.2rem"
            />
          </Group>
        ) : (
          "Select Repo"
        )
      }
      description={`Select an existing Repo to attach${!repoLinked ? ", or configure the repo below" : ""}.`}
    >
      <ResourceSelector
        type="Repo"
        selected={linkedRepo}
        onSelect={(linkedRepo) => {
          set({
            linked_repo: linkedRepo,
            // Set other props back to default
            git_provider: "github.com",
            git_account: "",
            git_https: true,
            repo: linkedRepo ? "" : "namespace/repo",
            branch: "main",
            commit: "",
          });
        }}
        disabled={disabled}
        clearable
      />
    </ConfigItem>
  );
}
