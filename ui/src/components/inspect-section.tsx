import { ICONS } from "@/theme/icons";
import Section, { SectionProps } from "@/ui/section";
import { useState } from "react";
import { MonacoEditor } from "./monaco";
import ShowHideButton from "@/ui/show-hide-button";
import { Box } from "@mantine/core";

export interface InspectSectionProps extends SectionProps {
  json: unknown;
}

export default function InspectSection({
  json,
  ...props
}: InspectSectionProps) {
  const [show, setShow] = useState(false);
  return (
    <Section
      title="Inspect"
      icon={<ICONS.Search size="1.3rem" />}
      titleRight={
        <Box pl="md">
          <ShowHideButton show={show} setShow={setShow} />
        </Box>
      }
      {...props}
    >
      {show && (
        <MonacoEditor
          value={JSON.stringify(json, null, 2)}
          language="json"
          readOnly
        />
      )}
    </Section>
  );
}
