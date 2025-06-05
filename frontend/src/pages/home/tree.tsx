import { ExportButton } from "@components/export";
import { Page, Section } from "@components/layouts";
import { ResourceComponents } from "@components/resources";
import { DeploymentTable } from "@components/resources/deployment/table";
import { ServerComponents } from "@components/resources/server";
import { TagsFilter, TagsWithBadge } from "@components/tags";
import { useFilterResources, useRead, useTagsFilter } from "@lib/hooks";
import { Button } from "@ui/button";
import { Card, CardHeader, CardTitle } from "@ui/card";
import { Input } from "@ui/input";
import { atom, useAtom } from "jotai";
import { Fragment, useState } from "react";
import { Link } from "react-router-dom";

const searchAtom = atom("");

export default function Tree() {
  const [search, setSearch] = useAtom(searchAtom);
  const tags = useTagsFilter();
  const servers = useRead("ListServers", { query: { tags } }).data;
  return (
    <Page
      titleOther={
        <div className="flex items-center justify-between">
          <div className="flex gap-4 items-center">
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="search..."
              className="w-[200px] lg:w-[300px]"
            />
            <ExportButton tags={tags} />
          </div>
          <TagsFilter />
        </div>
      }
    >
      <Section>
        <div className="grid gap-6">
          {servers?.map((server) => <Server key={server.id} id={server.id} />)}
        </div>
      </Section>
    </Page>
  );
}

const Server = ({ id }: { id: string }) => {
  const [search] = useAtom(searchAtom);
  // const [open, setOpen] = useLocalStorage(`server-tree-open-${id}`, false);
  const [open, setOpen] = useState(false);
  const server = useRead("ListServers", {}).data?.find(
    (server) => server.id === id
  );
  const deployments = useRead("ListDeployments", {}).data?.filter(
    (deployment) => deployment.info.server_id === id
  );
  const filtered = useFilterResources(deployments, search);
  return (
    <div className="grid gap-2">
      <Card
        className="h-full hover:bg-accent/50 group-focus:bg-accent/50 transition-colors cursor-pointer"
        onClick={() => setOpen(!open)}
      >
        <CardHeader className="p-4 flex-row justify-between items-center">
          <CardTitle>{server?.name}</CardTitle>
          <div className="flex gap-3 justify-between items-center">
            <TagsWithBadge tag_ids={server?.tags} />
            {server?.id && (
              <div className="flex gap-4 items-center">
                {Object.entries(ServerComponents.Info).map(([key, Info], i) => (
                  <Fragment key={key}>
                    {i !== 0 && "|"} <Info id={server.id} />
                  </Fragment>
                ))}
              </div>
            )}
            <Link to={`/servers/${server?.id}`}>
              <Button variant="outline">
                <ResourceComponents.Server.Icon id={server?.id} />
              </Button>
            </Link>
          </div>
        </CardHeader>
      </Card>
      {open && <DeploymentTable deployments={filtered ?? []} />}
    </div>
  );
};
