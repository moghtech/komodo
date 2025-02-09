import React from "react";
import RemoteCodeFile from "./RemoteCodeFile";
import Tabs from "@theme/Tabs";
import TabItem from "@theme/TabItem";

export default function ComposeAndEnv({
  file_name,
}: {
  file_name: string;
}) {
  return (
    <Tabs>
      <TabItem value={file_name}>
        <RemoteCodeFile
          title={`https://github.com/moghtech/komodo/blob/main/compose/${file_name}`}
          url={`https://raw.githubusercontent.com/moghtech/komodo/main/compose/${file_name}`}
          language="yaml"
        />
      </TabItem>
      <TabItem value="compose.env">
        <RemoteCodeFile
          title="https://github.com/moghtech/komodo/blob/main/compose/compose.env"
          url="https://raw.githubusercontent.com/moghtech/komodo/main/compose/compose.env"
          language="bash"
        />
      </TabItem>
    </Tabs>
  );
}
