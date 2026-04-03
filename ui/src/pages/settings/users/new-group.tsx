import { useInvalidate, useWrite } from "@/lib/hooks";
import { CreateModal } from "mogh_ui";
import { TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function NewUserGroup() {
  const inv = useInvalidate();
  const { mutateAsync: create, isPending } = useWrite("CreateUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      notifications.show({ message: "Created User Group", color: "green" });
    },
  });
  const [name, setName] = useState("");
  return (
    <CreateModal
      entityType="User Group"
      onConfirm={() => create({ name }).then(() => true)}
      loading={isPending}
      configSection={() => (
        <>
          <TextInput
            autoFocus
            placeholder="group-name"
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
