import React from "react";

export default function KomodoLogo({ width = "4rem" }) {
  return (
    <img
      style={{ width, height: "auto", mixBlendMode: "multiply" }}
      src="img/monitor-lizard.png"
      alt="monitor-lizard"
    />
  );
}
