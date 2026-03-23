import UrlBuilderConfig from "./url";
import ServerBuilderConfig from "./server";
import AwsBuilderConfig from "./aws";
import { useFullBuilder } from "..";

export default function BuilderConfig({ id }: { id: string }) {
  const config = useFullBuilder(id)?.config;
  switch (config?.type) {
    case "Aws":
      return <AwsBuilderConfig id={id} />;
    case "Server":
      return <ServerBuilderConfig id={id} />;
    case "Url":
      return <UrlBuilderConfig id={id} />;
  }
}
