import { atomWithStorage, useUser } from "@lib/hooks";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";
import { Variables } from "./variables";
import { Tags } from "./tags";
import { UsersPage } from "./users";
import { Profile } from "./profile";
import { Page } from "@components/layouts";
import { ProvidersPage } from "./providers";
import { ExportButton } from "@components/export";
import { useAtom } from "jotai";

type SettingsView = "Variables" | "Tags" | "Providers" | "Users" | "Profile";

const viewAtom = atomWithStorage<SettingsView>("settings-view-v2", "Variables");

export const useSettingsView = () => useAtom<SettingsView>(viewAtom);

export default function Settings() {
  const user = useUser().data;
  const [view, setView] = useSettingsView();
  const currentView =
    (view === "Users" || view === "Providers") && !user?.admin
      ? "Variables"
      : view;
  return (
    <Page>
      <Tabs
        value={currentView}
        onValueChange={setView as any}
        className="flex flex-col gap-6"
      >
        <div className="flex items-center justify-between">
          <TabsList className="justify-start w-fit">
            <TabsTrigger value="Variables">Variables</TabsTrigger>
            <TabsTrigger value="Tags">Tags</TabsTrigger>
            {user?.admin && (
              <TabsTrigger value="Providers">Providers</TabsTrigger>
            )}
            {user?.admin && <TabsTrigger value="Users">Users</TabsTrigger>}
            <TabsTrigger value="Profile">Profile</TabsTrigger>
          </TabsList>

          {currentView === "Variables" && <ExportButton include_variables />}
        </div>

        <TabsContent value="Variables">
          <Variables />
        </TabsContent>
        <TabsContent value="Tags">
          <Tags />
        </TabsContent>
        {user?.admin && (
          <TabsContent value="Providers">
            <ProvidersPage />
          </TabsContent>
        )}
        {user?.admin && (
          <TabsContent value="Users">
            <UsersPage goToProfile={() => setView("Profile")} />
          </TabsContent>
        )}
        <TabsContent value="Profile">
          <Profile />
        </TabsContent>
      </Tabs>
    </Page>
  );
}
