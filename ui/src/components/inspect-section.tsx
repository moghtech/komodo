import { ICONS } from "@/theme/icons";
import Section, { SectionProps } from "@/ui/section";
import { useState } from "react";
import { MonacoEditor } from "./monaco";
import ShowHideButton from "@/ui/show-hide-button";
import { Box } from "@mantine/core";
import { Types } from "komodo_client";
import { useRead } from "@/lib/hooks";

export interface InspectSectionProps extends Omit<SectionProps, "children"> {
  /* Inspect a read response */
  request?: Types.ReadRequest;
  json?: unknown;
  /* Use externally controlled show state */
  show?: boolean;
  setShow?: (show: boolean) => void;
  /* Use internal show state */
  showToggle?: boolean;
}

export default function InspectSection({
  request,
  json: __json,
  showToggle,
  show: __show = !showToggle,
  setShow,
  ...props
}: InspectSectionProps) {
  const {
    data: _json,
    isPending,
    isError,
  } = useRead(request?.type!, request?.params!, {
    enabled: !!request,
    refetchInterval: 10_000,
  });
  const json = __json ? __json : _json;
  const [_show, _setShow] = useState(true);
  const show = showToggle ? _show : __show;
  return (
    <Section
      isPending={!__json && isPending}
      error={
        !!__json
          ? undefined
          : isError
            ? "There was an error fetching the information"
            : !json
              ? "Did not find requested information"
              : undefined
      }
      title={props.titleOther ? undefined : "Inspect"}
      icon={props.titleOther ? undefined : <ICONS.Inspect size="1.3rem" />}
      titleRight={
        (showToggle || setShow) && (
          <Box pl="md">
            <ShowHideButton show={show} setShow={setShow ?? _setShow} />
          </Box>
        )
      }
      {...props}
    >
      {show && (
        <MonacoEditor
          value={json ? JSON.stringify(json, null, 2) : "NO DATA"}
          language="json"
          readOnly
        />
      )}
    </Section>
  );
}
