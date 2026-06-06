import { readFileSync } from "node:fs";

const app = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");

if (!app.includes("Zoid 25")) {
  throw new Error("Zoid 25 brand label is missing");
}

if (!app.includes('aria-label="Primary navigation"')) {
  throw new Error("Primary navigation sidebar scaffold is missing");
}

if (!app.includes("DESIGN SYSTEM READY")) {
  throw new Error("Design-system readiness label is missing");
}

if (!app.includes("blue-rail")) {
  throw new Error("Kujoyama-style blue rail is missing");
}
