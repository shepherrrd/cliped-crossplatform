import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./Styles/styles.css";

console.log("main.tsx loaded");

const rootElement = document.getElementById("root");
console.log("Root element:", rootElement);

if (rootElement) {
  const root = ReactDOM.createRoot(rootElement);
  console.log("Created React root, about to render App");

  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
  console.log("App rendered");
} else {
  console.error("Root element not found!");
}
