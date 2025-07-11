import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useHotKey() {
  useEffect(() => {
    console.log("useHotKey initialized");
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check if Ctrl+V is pressed
      if ((e.ctrlKey || e.metaKey) && e.key === "v") {
        e.preventDefault();
        console.log("Ctrl+V pressed, invoking paste action");
        invoke("toggle_window");
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
