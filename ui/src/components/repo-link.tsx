import { Group, Text } from "@mantine/core";
import { FolderGit } from "lucide-react";

export interface RepoLinkProps {
  repo: string;
  link: string;
}

export default function RepoLink({ repo, link }: RepoLinkProps) {
  return (
    <Group
      renderRoot={(props) => <a href={link} target="_blank" {...props} />}
      gap="xs"
      wrap="nowrap"
    >
      <FolderGit size="1rem" />
      <Text>{repo}</Text>
    </Group>
  );
}
