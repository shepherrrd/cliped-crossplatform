import { useState } from "react";
import ClipboardItemCard from "./ClipboardItem";
import Notification from "./Notification";
import ConfirmDialog from "./ConfirmDialog";
import FileTransfer from "./FileTransfer";
import { useClipboard } from "../Hooks/useClipboard";

interface ClipboardListProps {
  onOpenSettings: () => void;
}

type ViewMode = "all" | "files";

export default function ClipboardList({ onOpenSettings }: ClipboardListProps) {
  const {
    items,
    loading,
    clearAll,
    deleteItem,
    selectItem,
    isEnabled,
    toggleMonitoring,
    notification,
    undoLastAction,
    closeNotification,
    confirmation,
    cancelConfirmation,
    nextPage,
    previousPage,
    canGoNext,
    canGoPrevious,
    totalCount,
    currentPage,
    totalPages,
    itemsPerPage,
  } = useClipboard();
  const [searchText, setSearchText] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("all");
  const [showFileTransfer, setShowFileTransfer] = useState(false);

  const filteredItems = items.filter((item) => {
    const matchesSearch = item.content.toLowerCase().includes(searchText.toLowerCase()) ||
                         (item.file_name && item.file_name.toLowerCase().includes(searchText.toLowerCase()));
    
    if (viewMode === "files") {
      return matchesSearch && item.content_type === "file";
    }
    
    return matchesSearch;
  });

  const handleFileAdded = () => {
    setShowFileTransfer(false);
    // The useClipboard hook will automatically refresh and show the new file
  };

  if (loading) {
    return (
      <div className="loading-container">
        <div className="loading-spinner"></div>
        <span>Loading clipboard history...</span>
      </div>
    );
  }

  return (
    <div className="clipboard-list">
      {/* Monitoring Toggle */}
      <div className="monitoring-toggle">
        <button
          className={`toggle-button ${isEnabled ? "enabled" : "disabled"}`}
          onClick={toggleMonitoring}
          title={`Clipboard monitoring is ${
            isEnabled ? "enabled" : "disabled"
          }`}
        >
          <span className="toggle-icon">{isEnabled ? "🔥" : "⭕"}</span>
          <span className="toggle-text">
            {isEnabled ? "Monitoring Active" : "Monitoring Paused"}
          </span>
        </button>
      </div>

      {/* Header with search and action buttons */}
      <div className="list-header">
        <input
          type="text"
          placeholder="🔍 Search clipboard..."
          value={searchText}
          onChange={(e) => setSearchText(e.target.value)}
          className="search-input"
        />
        <div className="header-buttons">
          <div className="view-mode-buttons">
            <button
              className={`view-mode-button ${viewMode === "all" ? "active" : ""}`}
              onClick={() => setViewMode("all")}
              title="Show all items"
            >
              All
            </button>
            <button
              className={`view-mode-button ${viewMode === "files" ? "active" : ""}`}
              onClick={() => setViewMode("files")}
              title="Show files only"
            >
              Files
            </button>
          </div>
          <button
            className="file-transfer-button"
            onClick={() => setShowFileTransfer(!showFileTransfer)}
            title="Transfer files"
          >
            📁
          </button>
          {items.length > 0 && (
            <button
              className="clear-all-button"
              onClick={clearAll}
              title="Clear all clipboard items"
            >
              🗑️ Clear All
            </button>
          )}
          <button
            className="settings-button"
            onClick={onOpenSettings}
            title="Open Settings"
          >
            ⚙️
          </button>
        </div>
      </div>

      {/* Items count and pagination info */}
      {items.length > 0 && (
        <div className="items-count">
          {searchText ? (
            `${filteredItems.length} of ${totalCount > 0 ? totalCount : items.length} items (filtered)`
          ) : (
            `Showing ${currentPage * itemsPerPage + 1}-${Math.min((currentPage + 1) * itemsPerPage, totalCount)} of ${totalCount} items`
          )}
        </div>
      )}

      {/* Top Pagination Controls */}
      {!searchText && totalPages > 1 && (
        <div className="pagination-container top">
          <button
            className="pagination-button previous"
            onClick={previousPage}
            disabled={!canGoPrevious || loading}
            title="Previous page"
          >
            ← Previous
          </button>
          <div className="pagination-info">
            Page {currentPage + 1} of {totalPages}
          </div>
          <button
            className="pagination-button next"
            onClick={nextPage}
            disabled={!canGoNext || loading}
            title="Next page"
          >
            Next →
          </button>
        </div>
      )}

      {/* File Transfer */}
      {showFileTransfer && (
        <FileTransfer onFileAdded={handleFileAdded} />
      )}

      {/* Clipboard items */}
      <div className="items-container">
        {filteredItems.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">📋</div>
            <p className="empty-message">
              {searchText
                ? "No items match your search"
                : isEnabled
                ? "No clipboard history yet. Copy something to get started!"
                : "Clipboard monitoring is paused. Enable it to start collecting clipboard history."}
            </p>
            {searchText && (
              <button
                className="clear-search-button"
                onClick={() => setSearchText("")}
              >
                Clear Search
              </button>
            )}
          </div>
        ) : (
          <>
            {filteredItems.map((item) => (
              <ClipboardItemCard
                key={item.id}
                item={item}
                onDelete={deleteItem}
                onSelect={selectItem}
              />
            ))}
            {/* Bottom Pagination Controls */}
            {!searchText && totalPages > 1 && (
              <div className="pagination-container bottom">
                <button
                  className="pagination-button previous"
                  onClick={previousPage}
                  disabled={!canGoPrevious || loading}
                  title="Previous page"
                >
                  ← Previous
                </button>
                <div className="pagination-info">
                  Page {currentPage + 1} of {totalPages}
                </div>
                <button
                  className="pagination-button next"
                  onClick={nextPage}
                  disabled={!canGoNext || loading}
                  title="Next page"
                >
                  Next →
                </button>
              </div>
            )}
          </>
        )}
      </div>

      {/* Notification */}
      {notification && (
        <Notification
          message={notification.message}
          type={notification.type}
          canUndo={notification.canUndo}
          onUndo={undoLastAction}
          onClose={closeNotification}
        />
      )}

      {/* Confirmation Dialog */}
      {confirmation && (
        <ConfirmDialog
          isOpen={confirmation.isOpen}
          title={confirmation.title}
          message={confirmation.message}
          onConfirm={confirmation.onConfirm}
          onCancel={cancelConfirmation}
        />
      )}
    </div>
  );
}
