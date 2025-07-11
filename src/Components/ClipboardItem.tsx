import { useState } from "react";
import { ClipboardItem } from "../types";

type Props = {
  item: ClipboardItem;
  onDelete: (id: string) => void;
  onSelect: (content: string) => void;
};

export default function ClipboardItemCard({ item, onDelete, onSelect }: Props) {
  const [isExpanded, setIsExpanded] = useState(false);
  
  const maxPreviewLength = 80;
  const needsExpansion = item.content.length > maxPreviewLength;
  const truncatedContent = needsExpansion 
    ? item.content.substring(0, maxPreviewLength) + "..."
    : item.content;

  const formatTimestamp = (timestamp: string) => {
    // Handle both timestamp formats - HH:MM:SS and full datetime
    if (timestamp.includes(':') && !timestamp.includes('T')) {
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
      className={`clipboard-item ${isExpanded ? 'expanded' : ''}`}
      onClick={() => onSelect(item.content)}
    >
      <div className="item-header">
        <div className="item-icon">
          {item.content_type === 'image' ? '🖼️' : '📝'}
        </div>
        
        <div className="item-preview">
          <div className={`item-content-preview ${!isExpanded && needsExpansion ? 'truncated' : ''}`}>
            {isExpanded ? item.content : truncatedContent}
          </div>
          
          <div className="item-meta">
            <div className="device-info">
              <span className="device-icon">💻</span>
              <span>{item.device}</span>
            </div>
            <span>•</span>
            <span>{formatTimestamp(item.timestamp)}</span>
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
                  {isExpanded ? 'Show less' : 'Show more'}
                </button>
              </>
            )}
          </div>
        </div>
        
        <div className="item-actions">
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
        </div>
      </div>
      
      {isExpanded && (
        <div className="item-expanded-content">
          <div className="expanded-text">
            {item.content}
          </div>
          <div className="expanded-actions">
            <button 
              className="expanded-action-btn copy-button"
              onClick={(e) => {
                e.stopPropagation();
                onSelect(item.content);
              }}
            >
              Copy
            </button>
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
