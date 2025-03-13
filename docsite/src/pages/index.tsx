import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";

import styles from "./index.module.css";
import KomodoLogo from "../components/KomodoLogo";

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
              }}
            >
              Komodo
            </h1>
          </div>
        </div>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div style={{ display: "flex", justifyContent: "center" }}>
          <div className={styles.buttons}>
            <Link
              className="button button--secondary button--lg"
              to="/docs/intro"
            >
              Docs
            </Link>
            <Link
              className="button button--secondary button--lg"
              to="https://github.com/moghtech/komodo"
            >
              Github
            </Link>
            <Link
              className="button button--secondary button--lg"
              to="https://github.com/moghtech/komodo#screenshots"
              style={{
                width: "100%",
                boxSizing: "border-box",
                gridColumn: "span 2",
              }}
            >
              Screenshots
            </Link>
            <Link
              className="button button--secondary button--lg"
              to="https://demo.komo.do"
              style={{
                width: "100%",
                boxSizing: "border-box",
                gridColumn: "span 2",
              }}
            >
              Demo
            </Link>
          </div>
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
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
