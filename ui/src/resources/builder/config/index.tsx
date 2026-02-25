import { useRead } from "@/lib/hooks";
import UrlBuilderConfig from "./url";
import ServerBuilderConfig from "./server";
import AwsBuilderConfig from "./aws";

export default function BuilderConfig({ id }: { id: string }) {
  const config = useRead("GetBuilder", { builder: id }).data?.config;
  switch (config?.type) {
    case "Aws":
      return <AwsBuilderConfig id={id} />;
    case "Server":
      return <ServerBuilderConfig id={id} />;
    case "Url":
      return <UrlBuilderConfig id={id} />;
  }
}
