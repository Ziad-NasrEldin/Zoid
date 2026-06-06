import { AgentsHermesScreen } from "./agents/AgentsHermesScreen";

type NavigationStatus = "ready" | "idle" | "blocked";

type NavigationItem = {
  label: string;
  meta: string;
  status: NavigationStatus;
};

const navigationItems: NavigationItem[] = [
  { label: "Today", meta: "Current work", status: "idle" },
  { label: "Projects", meta: "Build lanes", status: "idle" },
  { label: "Agents", meta: "Hermes chat", status: "ready" },
  { label: "Code", meta: "Repos", status: "idle" },
  { label: "Content", meta: "OmniSocials", status: "blocked" },
  { label: "Automations", meta: "Routines", status: "idle" },
  { label: "Settings", meta: "Local app", status: "idle" },
];

const statusLabel = {
  ready: "ready",
  idle: "empty",
  blocked: "blocked",
} satisfies Record<NavigationStatus, string>;

function StatusDot({ status }: { status: NavigationItem["status"] }) {
  return <span aria-hidden="true" className={`status-dot ${status}`} />;
}

export default function App() {
  return (
    <main className="zoid25-shell" aria-label="Zoid 25 desktop scaffold">
      <aside className="blue-rail" aria-label="Global controls">
        <div className="rail-menu" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <span className="rail-lettermark" aria-label="Zoid 25">Z25</span>
        <nav className="rail-language" aria-label="Interface language">
          <span>EN</span>
          <span>FR</span>
          <span>日本</span>
        </nav>
      </aside>

      <aside className="editorial-sidebar" aria-label="Primary navigation">
        <div className="window-controls" aria-hidden="true">
          <span className="control close" />
          <span className="control minimize" />
          <span className="control zoom" />
        </div>

        <header className="brand-block">
          <p className="kana-line">ゾイド</p>
          <h1>
            ZOID
            <br />
            25
          </h1>
          <p className="brand-subtitle">macOS AI operating scaffold</p>
        </header>

        <nav className="nav-list" aria-label="Zoid 25 sections">
          {navigationItems.map((item) => (
            <button
              aria-current={item.label === "Agents" ? "page" : undefined}
              className={item.label === "Agents" ? "nav-row active" : "nav-row"}
              key={item.label}
              type="button"
            >
              <span className="nav-title">{item.label}</span>
              <span className="nav-meta">{item.meta}</span>
              <span className="nav-state">
                <StatusDot status={item.status} />
                {statusLabel[item.status]}
              </span>
            </button>
          ))}
        </nav>
      </aside>

      <AgentsHermesScreen />
    </main>
  );
}
