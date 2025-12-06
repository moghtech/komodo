import { Loader2 } from "lucide-react";
import { MonacoEditor } from "./monaco";

export const InspectResponseViewer = ({
  response,
  error,
  isPending,
  isError,
}: {
  response: Record<any, any> | undefined;
  error: unknown;
  isPending: boolean;
  isError: boolean;
}) => {
  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4 min-h-[60vh]">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }
  if (isError) {
    return (
      <div className="min-h-[60vh] flex flex-col">
        <h1 className="flex w-full py-4">Failed to inspect.</h1>
        {(error ?? undefined) && (
          <MonacoEditor
            value={JSON.stringify(error, null, 2)}
            language="json"
            readOnly
          />
        )}
      </div>
    );
  }
  return (
    <div className="min-h-[60vh]">
      <MonacoEditor
        value={JSON.stringify(response, null, 2)}
        language="json"
        readOnly
      />
    </div>
  );
};
