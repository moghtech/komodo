import { useInvalidate, useWrite } from "@/lib/hooks";
import CreateModal from "@/ui/create-modal";
import { TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function NewServiceUser() {
  const inv = useInvalidate();
  const { mutateAsync: create, isPending } = useWrite("CreateServiceUser", {
    onSuccess: () => {
      inv(["ListUsers"]);
      notifications.show({ message: "Created Service User", color: "green" });
    },
  });
  const [username, setUsername] = useState("");

  return (
    <CreateModal
      entityType="Service User"
      onConfirm={() => create({ username, description: "" })}
      loading={isPending}
      configureLabel="a unique username"
      configSection={() => (
        <>
          <TextInput
            autoFocus
            placeholder="username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            error={!username.trim() && "Enter username"}
          />
        </>
      )}
    />
  );
}
