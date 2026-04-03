import Section from "@/ui/section";
import UpdateList, { UpdateListProps } from "./list";
import { ICONS } from "@/lib/icons";
import { ActionIcon, Stack } from "@mantine/core";
import { Link } from "react-router-dom";

export interface UpdatesSectionProps extends UpdateListProps {
  link?: string;
}

export default function UpdatesSection({
  link,
  ...listProps
}: UpdatesSectionProps) {
  return (
    <Section
      title="Updates"
     
      icon={<ICONS.Update size="1.3rem" />}
      actions={
        link && (
          <ActionIcon component={Link} to={link}>
            <ICONS.ExternalLink size="1rem" />
          </ActionIcon>
        )
      }
      maw={{ xl2: 500, xl3: 600 }}
      forceHeaderGroup
      withBorder
    >
      <Stack mah={180} style={{ overflow: "auto" }}>
        <UpdateList max={10} showAllLink={link} {...listProps} />
      </Stack>
    </Section>
  );
}
