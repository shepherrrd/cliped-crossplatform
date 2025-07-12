import React from "react";
import ClipboardList from "./Components/ClipboardList";
import SettingsPage from "./Components/SettingsPage";
import "./Styles/styles.css";

export default function App() {
  const [showSettings, setShowSettings] = React.useState(false);

  return (
    <div className="app-container">
      {/* Header */}
      <div className="app-header">
        <h1 className="app-title">📋 Cliped</h1>
        <div className="hotkey-info">
          Beautiful clipboard manager with frosted glass UI
        </div>
      </div>

      {/* Main Content */}
      {showSettings ? (
        <SettingsPage onBack={() => setShowSettings(false)} />
      ) : (
        <ClipboardList onOpenSettings={() => setShowSettings(true)} />
      )}
    </div>
  );
}
