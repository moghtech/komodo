import React, { useState, useEffect, useCallback } from "react";
import OriginalSearchBar from "@easyops-cn/docusaurus-search-local/dist/client/client/theme/SearchBar";
import styles from "./styles.module.css";

const quickLinks = [
  { label: "Getting Started", to: "/docs/intro" },
  { label: "Setup Komodo Core", to: "/docs/setup" },
  { label: "Connect Servers", to: "/docs/setup/connect-servers" },
  { label: "Docker Compose", to: "/docs/deploy/compose" },
  { label: "Deploy Containers", to: "/docs/deploy/containers" },
  { label: "Docker Swarm", to: "/docs/swarm" },
  { label: "Build Images", to: "/docs/build" },
  { label: "Procedures and Actions", to: "/docs/automate/procedures" },
  { label: "Sync Resources", to: "/docs/automate/sync-resources" },
];

export default function SearchBarWrapper(props: any) {
  const [showQuickLinks, setShowQuickLinks] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const quickLinksWidth = 320;

  const updatePosition = useCallback(() => {
    const input = document.querySelector<HTMLInputElement>(
      ".navbar__search-input",
    );
    if (!input) return;
    const rect = input.getBoundingClientRect();
    setPosition({
      top: rect.bottom + 8,
      left: rect.left + rect.width / 2 - quickLinksWidth / 2,
    });
  }, []);

  // Inject Shift+S shortcut badge into search input container
  useEffect(() => {
    const inject = () => {
      const input = document.querySelector<HTMLInputElement>(
        ".navbar__search-input",
      );
      if (
        !input ||
        input.parentElement?.querySelector(`.${styles.shortcutBadge}`)
      )
        return;

      const badge = document.createElement("span");
      badge.className = styles.shortcutBadge;

      const shiftS = document.createElement("kbd");
      shiftS.textContent = "shift + s";

      badge.appendChild(shiftS);
      input.parentElement?.appendChild(badge);

      const hideBadge = () => {
        badge.style.display = "none";
      };
      const showBadge = () => {
        if (input.value === "") badge.style.display = "";
      };

      input.addEventListener("focus", hideBadge);
      input.addEventListener("blur", showBadge);
    };

    inject();
    // Re-inject after Docusaurus client-side navigation
    const observer = new MutationObserver(inject);
    observer.observe(document.body, { childList: true, subtree: true });
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const onFocusIn = (e: FocusEvent) => {
      const target = e.target as HTMLElement;
      if (target?.classList?.contains("navbar__search-input")) {
        const input = target as HTMLInputElement;
        if (input.value === "") {
          updatePosition();
          setShowQuickLinks(true);
        }
      }
    };

    const onInput = (e: Event) => {
      const target = e.target as HTMLInputElement;
      if (target?.classList?.contains("navbar__search-input")) {
        if (target.value !== "") {
          setShowQuickLinks(false);
        } else {
          updatePosition();
          setShowQuickLinks(true);
        }
      }
    };

    const onFocusOut = (e: FocusEvent) => {
      const target = e.target as HTMLElement;
      if (target?.classList?.contains("navbar__search-input")) {
        setTimeout(() => setShowQuickLinks(false), 150);
      }
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (
        e.shiftKey &&
        e.key === "S" &&
        !["INPUT", "TEXTAREA", "SELECT"].includes(
          (e.target as HTMLElement)?.tagName,
        ) &&
        !(e.target as HTMLElement)?.isContentEditable
      ) {
        e.preventDefault();
        const input = document.querySelector<HTMLInputElement>(
          ".navbar__search-input",
        );
        input?.focus();
      }
    };

    document.addEventListener("focusin", onFocusIn);
    document.addEventListener("input", onInput);
    document.addEventListener("focusout", onFocusOut);
    document.addEventListener("keydown", onKeyDown);

    return () => {
      document.removeEventListener("focusin", onFocusIn);
      document.removeEventListener("input", onInput);
      document.removeEventListener("focusout", onFocusOut);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [updatePosition]);

  return (
    <>
      <OriginalSearchBar {...props} />
      {showQuickLinks && (
        <div
          className={styles.quickLinks}
          style={{ top: position.top, left: position.left }}
        >
          {/* <div className={styles.quickLinksHeader}>Common</div> */}
          {quickLinks.map(({ label, to }) => (
            <a
              key={to}
              href={to}
              className={styles.quickLink}
              onClick={() => setShowQuickLinks(false)}
            >
              {label}
            </a>
          ))}
        </div>
      )}
    </>
  );
}
