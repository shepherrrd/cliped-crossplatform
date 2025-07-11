import ClipboardList from './Components/ClipboardList';
import './Styles/styles.css';

export default function App() {
  return (
    <div className="app-container">
      {/* Header */}
      <div className="app-header">
        <h1 className="app-title">📋 Cliped</h1>
        <div className="hotkey-info">
          Beautiful clipboard manager with frosted glass UI
        </div>
      </div>

      {/* Clipboard List */}
      <ClipboardList />
    </div>
  );
}
