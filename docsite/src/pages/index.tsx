import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";
import styles from "./index.module.css";
import KomodoLogo from "../components/KomodoLogo";
import { JSX } from "react";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <div style={{ display: "flex", gap: "1rem", justifyContent: "center" }}>
          <div style={{ position: "relative" }}>
            <KomodoLogo width="600px" />
            <h1
              className="hero__title"
              style={{
                margin: 0,
                position: "absolute",
                top: "40%",
                left: "50%",
                transform: "translate(-50%, -50%)",
                color: "white",
              }}
            >
              Komodo
            </h1>
          </div>
        </div>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro"
          >
            Docs
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="https://demo.komo.do"
          >
            Live Demo
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="https://github.com/moghtech/komodo"
          >
            GitHub
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="https://github.com/moghtech/komodo#screenshots"
          >
            Screenshots
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title="Home" description={siteConfig.tagline}>
      <HomepageHeader />
      <main>
        <div className={styles.upgradeBanner}>
          <div className="container">
            Running <b>Komodo v1</b>? See the{" "}
            <Link to="/docs/setup/version-upgrades#upgrading-to-komodo-v2">
              v2 migration guide
            </Link>
            .
          </div>
        </div>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
