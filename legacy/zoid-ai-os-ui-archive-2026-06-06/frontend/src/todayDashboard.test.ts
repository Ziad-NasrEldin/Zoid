import { buildTodayDashboardView } from "./todayDashboard";

const view = buildTodayDashboardView({ currentDateLabel: "Friday, Jun 5" });

if (view.title !== "Today" || view.dateLabel !== "Friday, Jun 5") {
  throw new Error("Today dashboard must preserve Stitch top-bar title and date label");
}

if (view.designSystem.name !== "Apple-design-analysis") {
  throw new Error("Today dashboard must explicitly use the updated Apple-design-analysis design system");
}

if (view.designSystem.primary !== "#0066cc" || view.designSystem.canvas !== "#ffffff" || view.designSystem.parchment !== "#f5f5f7") {
  throw new Error("Today dashboard must expose updated DESIGN.md color tokens");
}

if (!view.hero.title.includes("native evidence") || view.hero.tags.join(" ") !== "Native evidence Review gates No simulation") {
  throw new Error("Today dashboard hero must make native evidence, review gates, and no-simulation truth visible");
}

if (!view.sampleNotice.includes("Craft sample") || !view.sampleNotice.includes("Real native Zoid state") || !view.sampleNotice.includes("do not claim live records")) {
  throw new Error("Today dashboard must visibly classify sample operational content as non-live and non-simulated");
}

const tones = [
  ...view.topPanels.flatMap((panel) => [panel.statusTone, ...panel.items.map((item) => item.tone)]),
  ...view.operationPanels.flatMap((panel) => [panel.statusTone, ...panel.items.map((item) => item.tone)]),
  ...view.secondaryPanels.flatMap((panel) => [panel.statusTone, ...panel.items.map((item) => item.tone)]),
].filter(Boolean).map(String);

if (tones.some((tone) => tone === "red" || tone === "green")) {
  throw new Error("Today dashboard must not introduce red/green accent tones under Apple-design-analysis");
}

const topPanelTitles = view.topPanels.map((panel) => panel.title).join("|");
if (topPanelTitles !== "Needs Attention|Active Work") {
  throw new Error("Today dashboard must keep the Stitch priority row panels");
}

const operationTitles = view.operationPanels.map((panel) => panel.title).join("|");
if (operationTitles !== "Tasks|Calendar|Content Queue|Agents Status") {
  throw new Error("Today dashboard must keep the four Stitch operation panels in order");
}

const secondaryTitles = view.secondaryPanels.map((panel) => panel.title).join("|");
if (secondaryTitles !== "Dirty Repos|Automations|Recent Activity|Inbox Brief") {
  throw new Error("Today dashboard must keep the Stitch secondary panels in order");
}

if (view.operationPanels.find((panel) => panel.title === "Content Queue")?.media?.kind !== "product-photo") {
  throw new Error("Content Queue media must be classified as product-photo so it is the only allowed shadow-bearing image");
}

if (view.designSystem.disallowedUiShadow !== true) {
  throw new Error("Today dashboard must mark UI card shadows as disallowed under DESIGN.md");
}
