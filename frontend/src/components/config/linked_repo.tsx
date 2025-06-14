import { ResourceLink, ResourceSelector } from "@components/resources/common";
import { ConfigItem } from "./util";

export const LinkedRepoConfig = ({
  linked_repo,
  repo_linked,
  set,
  disabled,
}: {
  linked_repo: string | undefined;
  repo_linked: boolean;
  set: (update: {
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
}) => {
  return (
    <ConfigItem
      label={
        linked_repo ? (
          <div className="flex gap-3 text-lg font-bold">
            Repo:
            <ResourceLink type="Repo" id={linked_repo} />
          </div>
        ) : (
          "Select Repo"
        )
      }
      description={`Select an existing Repo to attach${!repo_linked ? ", or configure the repo below" : ""}.`}
    >
      <ResourceSelector
        type="Repo"
        selected={linked_repo}
        onSelect={(linked_repo) =>
          set({
            linked_repo,
            // Set other props back to default.
            git_provider: "github.com",
            git_account: "",
            git_https: true,
            repo: linked_repo ? "" : "namespace/repo",
            branch: "main",
            commit: "",
          })
        }
        disabled={disabled}
        align="start"
      />
    </ConfigItem>
  );
};
