import clsx from "clsx";
import Heading from "@theme/Heading";
import Link from "@docusaurus/Link";
import styles from "./styles.module.css";
import { JSX } from "react";
import {
  Layers,
  Server,
  Hammer,
  type LucideIcon,
  Route,
  FolderSync,
  Component,
  Terminal,
  Shield,
  Code,
} from "lucide-react";

type FeatureItem = {
  title: string;
  icon: LucideIcon;
  description: JSX.Element;
  to: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Unlimited Servers",
    icon: Server,
    to: "/docs/setup/connect-servers",
    description: (
      <>
        Connect all your servers, monitor CPU, memory, and disk usage, alert on
        thresholds, and open shell sessions from the browser.
      </>
    ),
  },
  {
    title: "Stacks and Containers",
    icon: Layers,
    to: "/docs/deploy/compose",
    description: (
      <>
        Deploy and manage containers and compose stacks across all your
        servers. View status and logs, connect to container shells, and
        orchestrate with automations.
      </>
    ),
  },
  {
    title: "Docker Swarm",
    icon: Component,
    to: "/docs/swarm",
    description: (
      <>
        Manage multiple swarms through a single entrypoint.
        View connected nodes, deploy services and stacks, and
        manage swarm configs and secrets.
      </>
    ),
  },
  {
    title: "Build Images",
    icon: Hammer,
    to: "/docs/build",
    description: (
      <>
        Automatically build images from git repos and push them to your registry for distribution.
        Integrated with AWS spot instances for unlimited capacity.
      </>
    ),
  },
  {
    title: "Procedures and Actions",
    icon: Route,
    to: "/docs/automate/procedures",
    description: (
      <>
        Chain executions into multi-stage procedures. Create scripts calling the Komodo API
        using the built-in editor for complex automations.
      </>
    ),
  },
  {
    title: "Declarative Resource Sync",
    icon: FolderSync,
    to: "/docs/automate/sync-resources",
    description: (
      <>
        Define all your resources as TOML files in a git repo and sync them to
        Komodo, keeping your infrastructure in version control.
      </>
    ),
  },
  {
    title: "Browser Terminals",
    icon: Terminal,
    to: "/docs/terminals",
    description: (
      <>
        Open persistent shell sessions on servers and containers directly from
        the browser. Multiple named sessions, shared access,
        and scriptable via Actions.
      </>
    ),
  },
  {
    title: "Role-Based Access Control",
    icon: Shield,
    to: "/docs/configuration/permissioning",
    description: (
      <>
        Granular permissions with user groups, per-resource access levels, and
        resource type restrictions. Control who can view or change every resource.
      </>
    ),
  },
  {
    title: "API, CLI, and Client Libraries",
    icon: Code,
    to: "/docs/ecosystem/api",
    description: (
      <>
        Full OpenAPI spec, a dedicated CLI, and typesafe client libraries for
        Rust and TypeScript. Integrate Komodo into any workflow or build your own
        tooling.
      </>
    ),
  },
];

function Feature({ title, icon: Icon, description, to }: FeatureItem) {
  return (
    <div className={clsx("col col--4", styles.featureCol)}>
      <Link to={to} className={styles.featureCardLink}>
        <div className={styles.featureCard}>
          <div className={styles.featureHeader}>
            <Heading as="h3">{title}</Heading>
            <Icon className={styles.featureIcon} />
          </div>
          <p>{description}</p>
        </div>
      </Link>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
