interface NotificationProps {
  message: string;
  type: "success" | "error" | "info";
  canUndo: boolean;
  onUndo?: () => void;
  onClose: () => void;
}

export default function Notification({
  message,
  type,
  canUndo,
  onUndo,
  onClose,
}: NotificationProps) {
  const getIcon = () => {
    switch (type) {
      case "success":
        return "✅";
      case "error":
        return "❌";
      case "info":
        return "ℹ️";
      default:
        return "ℹ️";
    }
  };

  const getStyles = () => {
    const baseStyles = {
      background: "rgba(40, 40, 45, 0.95)",
      backdropFilter: "blur(20px)",
      border: "1px solid rgba(255, 255, 255, 0.2)",
    };

    switch (type) {
      case "success":
        return { ...baseStyles, borderColor: "rgba(76, 175, 80, 0.5)" };
      case "error":
        return { ...baseStyles, borderColor: "rgba(244, 67, 54, 0.5)" };
      case "info":
        return { ...baseStyles, borderColor: "rgba(33, 150, 243, 0.5)" };
      default:
        return baseStyles;
    }
  };

  return (
    <div className="notification" style={getStyles()}>
      <div className="notification-content">
        <span className="notification-icon">{getIcon()}</span>
        <span className="notification-message">{message}</span>
      </div>

      <div className="notification-actions">
        {canUndo && onUndo && (
          <button className="notification-btn undo-btn" onClick={onUndo}>
            ↺ Undo
          </button>
        )}
        <button className="notification-btn close-btn" onClick={onClose}>
          ✕
        </button>
      </div>
    </div>
  );
}
