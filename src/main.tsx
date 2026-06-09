import React from "react";
import ReactDOM from "react-dom/client";
import { Agentation } from "agentation";
import App from "./App";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
    <Agentation />
  </React.StrictMode>,
);
