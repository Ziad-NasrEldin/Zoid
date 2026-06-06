import React from "react";
import ReactDOM from "react-dom/client";
import { Agentation } from "agentation";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
    {import.meta.env.DEV && <Agentation />}
  </React.StrictMode>,
);
