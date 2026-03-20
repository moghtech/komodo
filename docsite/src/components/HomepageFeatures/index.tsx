import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";
import { JSX } from "react";
import {
  Box,
  Layers,
  Server,
  Hammer,
  Play,
  GitBranch,
  type LucideIcon,
  Route,
  FolderSync,
} from "lucide-react";

type FeatureItem = {
  title: string;
  icon: LucideIcon;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Deploy Containers",
    icon: Box,
    description: (
      <>
        Deploy and manage Docker containers across all your servers. View status and
        logs, connect to container shells, and orchestrate them with automations.
      </>
    ),
  },
  {
    title: "Docker Compose Stacks",
    icon: Layers,
    description: (
      <>
        Deploy compose stacks with files defined in the UI or sourced from a git
        repo, with automatic redeploy on push.
      </>
    ),
  },
  {
    title: "Unlimited Servers",
    icon: Server,
    description: (
      <>
        Connect all your servers, monitor CPU, memory, and disk usage, alert on
        thresholds, and open shell sessions from the browser.
      </>
    ),
  },
  {
    title: "Automated Image Builds",
    icon: Hammer,
    description: (
      <>
        Build auto-versioned Docker images from git repos. Trigger builds on
        webhook, integrated with AWS spot instances for unlimited capacity.
      </>
    ),
  },
  {
    title: "Procedures and Actions",
    icon: Route,
    description: (
      <>
        Chain executions into multi-stage procedures, or use the UI editor for
        scripts that call the Komodo API for complex automations.
      </>
    ),
  },
  {
    title: "Declarative Resource Sync",
    icon: FolderSync,
    description: (
      <>
        Define all your resources as TOML files in a git repo and sync them to
        Komodo, keeping your infrastructure in version control.
      </>
    ),
  },
];

function Feature({ title, icon: Icon, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4", styles.featureCol)}>
      <div className={styles.featureCard}>
        <div className={styles.featureHeader}>
          <Heading as="h3">{title}</Heading>
          <Icon className={styles.featureIcon} />
        </div>
        <p>{description}</p>
      </div>
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
