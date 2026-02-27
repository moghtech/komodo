import { useMemo } from "react";
import { MoghAuth, Types } from "komodo_client";
import { Badge, Button, Text } from "@mantine/core";
import { KOMODO_BASE_URL } from "@/main";
import { notifications } from "@mantine/notifications";
import { useLoginOptions, useManageAuth } from "@/lib/hooks";
import { DataTable } from "@/ui/data-table";
import { ICONS } from "@/theme/icons";
import ConfirmModal from "@/ui/confirm-modal";
import Section from "@/ui/section";

const useLinkWithExternalLogin = () => {
  const { mutateAsync } = useManageAuth("BeginExternalLoginLink");
  return (provider: MoghAuth.Types.ExternalLoginProvider) =>
    mutateAsync({}).then(() =>
      location.replace(
        `${KOMODO_BASE_URL}/auth/${provider.toLowerCase()}/link`,
      ),
    );
};

export const LinkedLogins = ({
  user,
  refetchUser,
}: {
  user: Types.User;
  refetchUser: () => void;
}) => {
  const options = useLoginOptions().data;
  const loginProviders: Array<{
    provider: MoghAuth.Types.LoginProvider;
    enabled: boolean;
    data: string | undefined;
  }> = useMemo(() => {
    return [
      {
        provider: MoghAuth.Types.LoginProvider.Local,
        enabled: !!options?.local,
        data: (
          user.linked_logins?.Local as Extract<
            Types.UserConfig,
            { type: "Local" }
          >
        )?.data?.password
          ? "########"
          : undefined,
      },
      {
        provider: MoghAuth.Types.LoginProvider.Oidc,
        enabled: !!options?.oidc,
        data: (
          user.linked_logins?.Oidc as Extract<
            Types.UserConfig,
            { type: "Oidc" }
          >
        )?.data?.user_id,
      },
      {
        provider: MoghAuth.Types.LoginProvider.Github,
        enabled: !!options?.github,
        data: (
          user.linked_logins?.Github as Extract<
            Types.UserConfig,
            { type: "Github" }
          >
        )?.data?.github_id,
      },
      {
        provider: MoghAuth.Types.LoginProvider.Google,
        enabled: !!options?.google,
        data: (
          user.linked_logins?.Google as Extract<
            Types.UserConfig,
            { type: "Google" }
          >
        )?.data?.google_id,
      },
    ].filter(
      ({ enabled, provider }) => enabled && user.config.type !== provider,
    );
  }, [user, options]);
  const linkWithExternalLogin = useLinkWithExternalLogin();
  const { mutateAsync: unlink } = useManageAuth("UnlinkLogin", {
    onSuccess: () => {
      notifications.show({ message: "Unlinked login." });
      refetchUser();
    },
  });

  if (!loginProviders.length) {
    return null;
  }

  return (
    <Section
      title="Providers"
      titleFz="h3"
      icon={<ICONS.Provider size="1.2rem" />}
      withBorder
    >
      <DataTable
        noBorder
        tableKey="login-providers-v1"
        data={loginProviders}
        columns={[
          {
            header: "Provider",
            accessorKey: "provider",
            cell: ({ row }) => <Text fw="bold">{row.original.provider}</Text>,
          },
          {
            header: "Linked",
            cell: ({
              row: {
                original: { data },
              },
            }) => (
              <Badge color={data ? "green.8" : "red"}>
                {data ? "Linked" : "Unlinked"}
              </Badge>
            ),
          },
          {
            header: "Data",
            cell: ({
              row: {
                original: { data },
              },
            }) =>
              data && (
                <Text
                  maw="20vw"
                  size="sm"
                  style={{
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    textWrap: "nowrap",
                  }}
                >
                  {data}
                </Text>
              ),
          },
          {
            header: "Link",
            cell: ({
              row: {
                original: { provider, data },
              },
            }) =>
              data ? (
                <ConfirmModal
                  icon={<ICONS.Unlink size="1rem" />}
                  onConfirm={() => unlink({ provider })}
                  confirmText="Unlink"
                  title="Unlink Login"
                >
                  Unlink
                </ConfirmModal>
              ) : provider === "Local" ? (
                <>Set password above to enable.</>
              ) : (
                <Button
                  color="green.8"
                  onClick={() => linkWithExternalLogin(provider as any)}
                  leftSection={<ICONS.Create size="1rem" />}
                >
                  Link {provider}
                </Button>
              ),
          },
        ]}
      />
    </Section>
  );
};
