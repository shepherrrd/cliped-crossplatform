import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ClipboardItem } from '../types';

interface NotificationState {
  message: string;
  type: 'success' | 'error' | 'info';
  canUndo: boolean;
  undoData?: any;
}

interface ConfirmationState {
  isOpen: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
}

export function useClipboard() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [isEnabled, setIsEnabled] = useState(true);
  const [notification, setNotification] = useState<NotificationState | null>(null);
  const [lastDeletedItems, setLastDeletedItems] = useState<ClipboardItem[]>([]);
  const [confirmation, setConfirmation] = useState<ConfirmationState | null>(null);

  // Load initial clipboard history and monitoring status
  useEffect(() => {
    console.log("useClipboard: Loading initial data...");
    const loadHistory = async () => {
      try {
        console.log("useClipboard: Calling backend commands...");
        const [history, enabled] = await Promise.all([
          invoke<ClipboardItem[]>('get_clipboard_history'),
          invoke<boolean>('is_monitoring_enabled')
        ]);
        console.log("useClipboard: Received history:", history);
        console.log("useClipboard: Monitoring enabled:", enabled);
        setItems(history);
        setIsEnabled(enabled);
      } catch (error) {
        console.error('Failed to load clipboard history:', error);
      } finally {
        setLoading(false);
      }
    };

    loadHistory();
  }, []);

  // Auto-refresh clipboard history every 20ms
  useEffect(() => {
    console.log("useClipboard: Setting up auto-refresh polling...");
    
    const pollInterval = setInterval(async () => {
      try {
        const history = await invoke<ClipboardItem[]>('get_clipboard_history');
        
        setItems(prev => {
          // Only update if the history has actually changed
          if (prev.length !== history.length || 
              (history.length > 0 && prev.length > 0 && prev[0]?.id !== history[0]?.id)) {
            console.log(`useClipboard: History updated - ${prev.length} -> ${history.length} items`);
            return history;
          }
          return prev;
        });
      } catch (error) {
        console.error('Failed to poll clipboard history:', error);
      }
    }, 20); // Poll every 20ms for instant updates

    console.log("useClipboard: Auto-refresh polling started");

    return () => {
      console.log("useClipboard: Cleaning up auto-refresh polling...");
      clearInterval(pollInterval);
    };
  }, []);

  const showNotification = (message: string, type: 'success' | 'error' | 'info', canUndo = false, undoData?: any) => {
    setNotification({ message, type, canUndo, undoData });
    // Auto-hide notification after 5 seconds
    setTimeout(() => setNotification(null), 5000);
  };

  const clearAll = async () => {
    setConfirmation({
      isOpen: true,
      title: 'Clear All Items',
      message: 'Are you sure you want to clear all clipboard history?\n\nThis action cannot be undone.',
      onConfirm: async () => {
        try {
          const currentItems = [...items];
          await invoke('clear_clipboard_history');
          setItems([]);
          setLastDeletedItems(currentItems);
          showNotification('All clipboard history cleared', 'success', true, { type: 'clearAll', items: currentItems });
        } catch (error) {
          console.error('Failed to clear clipboard history:', error);
          showNotification('Failed to clear clipboard history', 'error');
        }
        setConfirmation(null);
      }
    });
  };

  const deleteItem = async (id: string) => {
    const itemToDelete = items.find(item => item.id === id);
    if (!itemToDelete) return;

    const previewText = itemToDelete.content.substring(0, 100) + (itemToDelete.content.length > 100 ? '...' : '');
    
    setConfirmation({
      isOpen: true,
      title: 'Delete Item',
      message: `Delete this clipboard item?\n\n"${previewText}"`,
      onConfirm: async () => {
        try {
          await invoke('delete_clipboard_item', { id });
          setItems(prev => prev.filter(item => item.id !== id));
          showNotification('Clipboard item deleted', 'success', true, { type: 'deleteItem', item: itemToDelete });
        } catch (error) {
          console.error('Failed to delete clipboard item:', error);
          showNotification('Failed to delete clipboard item', 'error');
        }
        setConfirmation(null);
      }
    });
  };

  const undoLastAction = async () => {
    if (!notification?.undoData) return;

    try {
      const { type, item, items: deletedItems } = notification.undoData;
      
      if (type === 'deleteItem' && item) {
        // Re-add the deleted item
        await invoke('add_clipboard_item', { item });
        setItems(prev => [item, ...prev]);
        showNotification('Item restored', 'success');
      } else if (type === 'clearAll' && deletedItems) {
        // Restore all deleted items
        for (const item of deletedItems.reverse()) {
          await invoke('add_clipboard_item', { item });
        }
        setItems(deletedItems);
        showNotification('All items restored', 'success');
      }
      
      setNotification(null);
    } catch (error) {
      console.error('Failed to undo action:', error);
      showNotification('Failed to undo action', 'error');
    }
  };

  const selectItem = async (content: string) => {
    try {
      await invoke('set_clipboard_content', { content });
      // Note: Window stays open for continuous use
    } catch (error) {
      console.error('Failed to set clipboard content:', error);
    }
  };

  const toggleMonitoring = async () => {
    try {
      const newState = await invoke<boolean>('toggle_monitoring');
      setIsEnabled(newState);
    } catch (error) {
      console.error('Failed to toggle monitoring:', error);
    }
  };

  const closeNotification = () => {
    setNotification(null);
  };

  const cancelConfirmation = () => {
    setConfirmation(null);
  };

  return {
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
  };
}
