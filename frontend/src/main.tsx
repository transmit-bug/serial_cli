import React from "react";
import ReactDOM from "react-dom/client";
import "@/i18n";
import "@/index.css";
import { applyStoredTheme } from "@/hooks/useTheme";
import { App } from "./App";

// Apply theme before first paint to avoid flash
applyStoredTheme();

const root = document.getElementById("root");
if (!root) throw new Error("Root element not found");

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
