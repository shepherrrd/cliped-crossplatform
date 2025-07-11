import { useState } from 'react';
import ClipboardItemCard from './ClipboardItem';
import Notification from './Notification';
import ConfirmDialog from './ConfirmDialog';
import { useClipboard } from '../Hooks/useClipboard';

export default function ClipboardList() {
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
    cancelConfirmation
  } = useClipboard();
  const [searchText, setSearchText] = useState('');

  const filteredItems = items.filter(item => 
    item.content.toLowerCase().includes(searchText.toLowerCase())
  );

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
          className={`toggle-button ${isEnabled ? 'enabled' : 'disabled'}`}
          onClick={toggleMonitoring}
          title={`Clipboard monitoring is ${isEnabled ? 'enabled' : 'disabled'}`}
        >
          <span className="toggle-icon">{isEnabled ? '🔥' : '⭕'}</span>
          <span className="toggle-text">
            {isEnabled ? 'Monitoring Active' : 'Monitoring Paused'}
          </span>
        </button>
      </div>

      {/* Header with search and clear button */}
      <div className="list-header">
        <input
          type="text"
          placeholder="🔍 Search clipboard..."
          value={searchText}
          onChange={(e) => setSearchText(e.target.value)}
          className="search-input"
        />
        {items.length > 0 && (
          <button 
            className="clear-all-button" 
            onClick={clearAll}
            title="Clear all clipboard items"
          >
            🗑️ Clear All
          </button>
        )}
      </div>

      {/* Items count */}
      {items.length > 0 && (
        <div className="items-count">
          {filteredItems.length} of {items.length} items
          {searchText && ` (filtered)`}
        </div>
      )}

      {/* Clipboard items */}
      <div className="items-container">
        {filteredItems.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">📋</div>
            <p className="empty-message">
              {searchText 
                ? 'No items match your search' 
                : isEnabled 
                  ? 'No clipboard history yet. Copy something to get started!'
                  : 'Clipboard monitoring is paused. Enable it to start collecting clipboard history.'
              }
            </p>
            {searchText && (
              <button 
                className="clear-search-button" 
                onClick={() => setSearchText('')}
              >
                Clear Search
              </button>
            )}
          </div>
        ) : (
          filteredItems.map((item) => (
            <ClipboardItemCard 
              key={item.id} 
              item={item} 
              onDelete={deleteItem} 
              onSelect={selectItem}
            />
          ))
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
