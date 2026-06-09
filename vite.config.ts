import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

const agentationDetailFixedEntry = fileURLToPath(
  new URL("./src/vendor/agentation-fixed.mjs", import.meta.url),
);

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  resolve: {
    alias: {
      agentation: agentationDetailFixedEntry,
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (
            id.includes("/src/agents/sessionState") ||
            id.includes("/src/agents/sessionPortraits") ||
            id.includes("/src/agents/hermesProfileClient") ||
            id.includes("/src/agents/types")
          ) return undefined;
          if (id.includes("/src/agents/")) return "workspace-agents";
          if (id.includes("/src/brain/")) return "workspace-brain";
          if (id.includes("/src/code/")) return "workspace-code";
          if (id.includes("/src/content/")) return "workspace-content";
          if (id.includes("/src/automations/")) return "workspace-automations";
          if (id.includes("/src/providers/")) return "workspace-providers";
          if (!id.includes("node_modules")) return undefined;
          if (id.includes("/react/") || id.includes("/react-dom/")) return "react-vendor";
          if (id.includes("/lucide-react/")) return "icons";
          if (id.includes("/@tauri-apps/")) return "tauri-vendor";
          if (id.includes("/agentation/")) return "agentation";
          return "vendor";
        },
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
