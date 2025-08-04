import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ClipboardItem } from "../types";

type Props = {
  item: ClipboardItem;
  onDelete: (id: string) => void;
  onSelect: (content: string) => void;
};

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

export default function ClipboardItemCard({ item, onDelete, onSelect }: Props) {
  const [isExpanded, setIsExpanded] = useState(false);

  const maxPreviewLength = 80;
  const needsExpansion = item.content.length > maxPreviewLength;
  const truncatedContent = needsExpansion
    ? item.content.substring(0, maxPreviewLength) + "..."
    : item.content;

  const handleItemClick = () => {
    if (item.content_type === "file") {
      // For files, expand/collapse instead of copying
      setIsExpanded(!isExpanded);
    } else {
      // For text/image items, copy to clipboard
      onSelect(item.content);
    }
  };

  const handleFileDownload = async () => {
    if (!item.file_path) return;
    
    try {
      const fileContent = await invoke<number[]>("get_file_content", { filePath: item.file_path });
      const fileName = item.file_name || "downloaded_file";
      
      // Convert number array to Uint8Array
      const uint8Array = new Uint8Array(fileContent);
      
      const savedPath = await invoke<string>("save_received_file", { 
        content: Array.from(uint8Array), 
        fileName 
      });
      
      console.log("File saved to:", savedPath);
    } catch (error) {
      console.error("Failed to download file:", error);
    }
  };

  const formatTimestamp = (timestamp: string) => {
    // Handle both timestamp formats - HH:MM:SS and full datetime
    if (timestamp.includes(":") && !timestamp.includes("T")) {
      // Format like "14:30:25"
      return timestamp;
    }

    const date = new Date(timestamp);
    if (isNaN(date.getTime())) {
      return timestamp; // Return as-is if can't parse
    }

    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return date.toLocaleDateString();
  };

  return (
    <div
      className={`clipboard-item ${isExpanded ? "expanded" : ""} ${item.content_type === "file" ? "file-item" : ""}`}
      onClick={() => handleItemClick()}
    >
      <div className="item-header">
        <div className="item-icon">
          {item.content_type === "image" ? "🖼️" : 
           item.content_type === "file" ? "📁" : "📝"}
        </div>

        <div className="item-preview">
          {!isExpanded && (
            <div
              className={`item-content-preview ${
                needsExpansion ? "truncated" : ""
              }`}
            >
              {truncatedContent}
            </div>
          )}

          <div className="item-meta">
            <div className="device-info">
              <span className="device-icon">💻</span>
              <span>{item.device}</span>
            </div>
            <span>•</span>
            <span>{formatTimestamp(item.timestamp)}</span>
            {item.content_type === "file" && item.file_size && (
              <>
                <span>•</span>
                <span>{formatFileSize(item.file_size)}</span>
              </>
            )}
            {needsExpansion && (
              <>
                <span>•</span>
                <button
                  className="more-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsExpanded(!isExpanded);
                  }}
                >
                  {isExpanded ? "Show less" : "Show more"}
                </button>
              </>
            )}
          </div>
        </div>

        <div className="item-actions">
          {!isExpanded && (
            <button
              className="action-button delete"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(item.id);
              }}
              title="Delete item"
            >
              🗑️
            </button>
          )}
        </div>
      </div>

      {isExpanded && (
        <div className="item-expanded-content">
          <div className="expanded-text">{item.content}</div>
          <div className="expanded-actions">
            {item.content_type === "file" ? (
              <button
                className="expanded-action-btn download-button"
                onClick={(e) => {
                  e.stopPropagation();
                  handleFileDownload();
                }}
              >
                Download
              </button>
            ) : (
              <button
                className="expanded-action-btn copy-button"
                onClick={(e) => {
                  e.stopPropagation();
                  onSelect(item.content);
                }}
              >
                Copy
              </button>
            )}
            <button
              className="expanded-action-btn delete-button"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(item.id);
              }}
            >
              Delete
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
