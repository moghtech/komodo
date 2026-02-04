import { ICONS } from "@/theme/icons";
import { ResourceLink } from "@/resources/common";
import { Center, Group, Loader, Text } from "@mantine/core";
import RepoLink from "./repo-link";
import { NotepadText } from "lucide-react";

export interface FileSourceInfo {
  linked_repo: string;
  files_on_host: boolean;
  repo: string;
  /** The http/s link to the repository page */
  repo_link: string;
}

export interface FileSourceProps {
  info: FileSourceInfo | undefined;
}

export default function FileSource({ info }: FileSourceProps) {
  if (!info) {
    return (
      <Center>
        <Loader size="sm" />
      </Center>
    );
  } else if (info.files_on_host) {
    return (
      <Group gap="xs">
        <ICONS.Server size="1rem" />
        <Text>Files on Server</Text>
      </Group>
    );
  } else if (info.linked_repo) {
    return <ResourceLink type="Repo" id={info.linked_repo} />;
  } else if (info.repo) {
    return <RepoLink repo={info.repo} link={info.repo_link} />;
  } else {
    return (
      <Group>
        <NotepadText size="1rem" />
        <Text>UI Defined</Text>
      </Group>
    );
  }
}
