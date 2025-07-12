import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useGlobalShortcut() {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Handle global shortcut: Ctrl+V on macOS, Ctrl+Shift+V on Windows/Linux
      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const targetKey = "KeyV";

      if (isMac) {
        // macOS: Control+V
        if (
          event.ctrlKey &&
          event.code === targetKey &&
          !event.shiftKey &&
          !event.metaKey &&
          !event.altKey
        ) {
          event.preventDefault();
          invoke("toggle_window").catch(console.error);
        }
      } else {
        // Windows/Linux: Ctrl+Shift+V
        if (
          event.ctrlKey &&
          event.shiftKey &&
          event.code === targetKey &&
          !event.metaKey &&
          !event.altKey
        ) {
          event.preventDefault();
          invoke("toggle_window").catch(console.error);
        }
      }
    };

    // Add global event listener
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);
}
