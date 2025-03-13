import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { Button } from "@ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { FileDown, Loader2 } from "lucide-react";
import { useState } from "react";
import { CopyButton } from "./util";
import { MonacoEditor } from "./monaco";

export const ExportButton = ({
  targets,
  user_groups,
  tags,
  include_variables,
}: {
  targets?: Types.ResourceTarget[];
  user_groups?: string[];
  tags?: string[];
  include_variables?: boolean;
}) => {
  const [open, setOpen] = useState(false);
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="flex gap-2 items-center">
          <FileDown className="w-4 h-4" />
          Toml
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[900px] max-w-[95vw]">
        <DialogHeader>
          <DialogTitle>Export to Toml</DialogTitle>
        </DialogHeader>
        {targets || user_groups || include_variables ? (
          <ExportTargetsLoader
            targets={targets}
            user_groups={user_groups}
            include_variables={include_variables}
          />
        ) : (
          <ExportAllLoader tags={tags} />
        )}
      </DialogContent>
    </Dialog>
  );
};

const ExportTargetsLoader = ({
  targets,
  user_groups,
  include_variables,
}: {
  targets?: Types.ResourceTarget[];
  user_groups?: string[];
  include_variables?: boolean;
}) => {
  const { data, isPending } = useRead("ExportResourcesToToml", {
    targets: targets ? targets : [],
    user_groups: user_groups ? user_groups : [],
    include_variables,
  });
  return <ExportPre loading={isPending} content={data?.toml} />;
};

const ExportAllLoader = ({
  tags,
}: {
  tags?: string[];
}) => {
  const { data, isPending } = useRead("ExportAllResourcesToToml", {
    tags,
    include_resources: true,
    include_variables: true,
    include_user_groups: true,
  });
  return <ExportPre loading={isPending} content={data?.toml} />;
};

const ExportPre = ({
  loading,
  content,
}: {
  loading: boolean;
  content: string | undefined;
}) => {
  return (
    <div className="relative flex justify-center w-full overflow-y-scroll max-h-[80vh]">
      {loading && <Loader2 className="w-8 h-8 animate-spin" />}
      <MonacoEditor value={content} language="toml" readOnly />
      <CopyButton content={content} className="absolute top-4 right-4" />
    </div>
  );
};
