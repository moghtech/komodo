import { useInvalidate, useUser, useWrite } from "@/lib/hooks";
import CreateModal from "@/ui/create-modal";
import { TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function NewVariable() {
  const invalidate = useInvalidate();
  const [name, setName] = useState("");
  const { mutateAsync: create, isPending } = useWrite("CreateVariable", {
    onSuccess: () => {
      invalidate(["ListVariables"], ["GetVariable"]);
      notifications.show({ message: "Variable Created" });
      setName("");
    },
  });
  const user = useUser().data;
  const disabled = !user?.admin;
  return (
    <CreateModal
      entityType="Variable"
      onConfirm={() => create({ name })}
      disabled={disabled}
      loading={isPending}
      openShiftKeyListener="N"
      configSection={() => (
        <>
          <TextInput
            autoFocus
            placeholder="variable-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            // onKeyDown={(e) => {
            //   if (!name) {
            //     return;
            //   }
            //   if (e.key === "Enter") {
            //     create({});
            //   }
            // }}
            error={!name.trim() && "Enter name"}
          />
        </>
      )}
    />
  );
}
