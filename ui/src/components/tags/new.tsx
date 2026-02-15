import { useInvalidate, useWrite } from "@/lib/hooks";
import CreateModal from "@/ui/create-modal";
import { TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function NewTag() {
  const invalidate = useInvalidate();
  const [name, setName] = useState("");
  const { mutateAsync: create, isPending } = useWrite("CreateTag", {
    onSuccess: () => {
      invalidate(["ListTags"], ["GetTag"]);
      notifications.show({ message: "Tag Created" });
      setName("");
    },
  });
  return (
    <CreateModal
      entityType="Tag"
      onConfirm={() => create({ name })}
      disabled={false}
      loading={isPending}
      openShiftKeyListener="N"
      configSection={() => (
        <>
          <TextInput
            autoFocus
            placeholder="tag-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            error={!name.trim() && "Enter name"}
          />
        </>
      )}
    />
  );
}
