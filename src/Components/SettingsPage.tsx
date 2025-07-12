import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Device {
  id: number;
  name: string;
  icon: string;
  ip: string;
  status?: string;
  sync_mode?: string;
  last_seen?: number;
}

interface SettingsPageProps {
  onBack: () => void;
}

const SettingsPage: React.FC<SettingsPageProps> = ({ onBack }) => {
  const [devices, setDevices] = useState<Device[]>([]);
  const [localDevice, setLocalDevice] = useState<Device | null>(null);
  const [availableDevices, setAvailableDevices] = useState<Device[]>([]);
  const [pendingConnections, setPendingConnections] = useState<Device[]>([]);
  const [isEditingName, setIsEditingName] = useState(false);
  const [newDeviceName, setNewDeviceName] = useState("");
  const [isDiscovering, setIsDiscovering] = useState(false);

  useEffect(() => {
    loadDevices();
    loadPendingConnections();
    discoverDevices();
  }, []);

  const loadDevices = async () => {
    try {
      // Fetch local device info
      const localDev = await invoke<Device>("get_local_device");
      setLocalDevice(localDev);
      if (localDev) {
        setNewDeviceName(localDev.name);
      }

      // Fetch connected devices
      const connectedDevs = await invoke<Device[]>("get_connected_devices");
      setDevices(connectedDevs);
    } catch (error) {
      console.error("Failed to load devices:", error);
    }
  };

  const loadPendingConnections = async () => {
    try {
      const pending = await invoke<Device[]>("get_pending_connections");
      setPendingConnections(pending);
    } catch (error) {
      console.error("Failed to load pending connections:", error);
    }
  };

  const discoverDevices = async () => {
    setIsDiscovering(true);
    try {
      const discovered = await invoke<Device[]>("discover_devices");
      // Filter out devices that are already connected or pending
      const connectedIds = devices.map((d) => d.id);
      const pendingIds = pendingConnections.map((d) => d.id);
      const filteredDiscovered = discovered.filter(
        (device) => !connectedIds.includes(device.id) && !pendingIds.includes(device.id)
      );
      setAvailableDevices(filteredDiscovered);
    } catch (error) {
      console.error("Failed to discover devices:", error);
    } finally {
      setIsDiscovering(false);
    }
  };

  const updateDeviceName = async () => {
    if (!newDeviceName.trim()) return;

    try {
      await invoke("update_device_name", { newName: newDeviceName.trim() });
      setLocalDevice((prev) =>
        prev ? { ...prev, name: newDeviceName.trim() } : null
      );
      setIsEditingName(false);
    } catch (error) {
      console.error("Failed to update device name:", error);
      alert("Failed to update device name");
    }
  };

  const sendConnectionRequest = async (device: Device) => {
    try {
      await invoke("send_connection_request_to_device", {
        targetDevice: device,
      });
      alert(`Connection request sent to ${device.name}!`);
      // Remove from available devices and refresh pending connections
      setAvailableDevices((prev) => prev.filter((d) => d.id !== device.id));
      await loadPendingConnections();
    } catch (error) {
      console.error("Failed to send connection request:", error);
      alert("Failed to send connection request");
    }
  };

  const acceptConnection = async (deviceId: number) => {
    try {
      await invoke("accept_connection", { deviceId });
      // Refresh all lists
      await loadDevices();
      await loadPendingConnections();
      await discoverDevices();
    } catch (error) {
      console.error("Failed to accept connection:", error);
      alert("Failed to accept connection");
    }
  };

  const denyConnection = async (deviceId: number) => {
    try {
      await invoke("deny_connection", { deviceId });
      // Refresh pending connections
      await loadPendingConnections();
    } catch (error) {
      console.error("Failed to deny connection:", error);
      alert("Failed to deny connection");
    }
  };

  const addDevice = (ipOrTag: string) => {
    invoke("add_device", { ipOrTag }).then(() => {
      alert("Device added successfully!");
    });
  };

  const removeDevice = (id: number) => {
    invoke("remove_device", { deviceId: id }).then(() => {
      setDevices(devices.filter((device) => device.id !== id));
    });
  };

  return (
    <div className="settings-page">
      {/* Header with back button */}
      <div className="settings-header">
        <button className="back-button" onClick={onBack}>
          ← Back to Clipboard
        </button>
        <h1>Settings</h1>
      </div>

      {localDevice && (
        <div className="local-device">
          <h2>Your Device</h2>
          <div className="device-info">
            <div className="device-avatar">💻</div>
            <div className="device-details">
              <div className="device-name-section">
                <strong>Name:</strong>{" "}
                {isEditingName ? (
                  <div className="edit-name-container">
                    <input
                      type="text"
                      value={newDeviceName}
                      onChange={(e) => setNewDeviceName(e.target.value)}
                      className="name-input"
                      onKeyDown={(e) => {
                        if (e.key === "Enter") updateDeviceName();
                        if (e.key === "Escape") {
                          setIsEditingName(false);
                          setNewDeviceName(localDevice.name);
                        }
                      }}
                      autoFocus
                    />
                    <button
                      className="save-name-btn"
                      onClick={updateDeviceName}
                    >
                      ✓
                    </button>
                    <button
                      className="cancel-name-btn"
                      onClick={() => {
                        setIsEditingName(false);
                        setNewDeviceName(localDevice.name);
                      }}
                    >
                      ✕
                    </button>
                  </div>
                ) : (
                  <span className="device-name">
                    {localDevice.name}
                    <button
                      className="edit-name-btn"
                      onClick={() => setIsEditingName(true)}
                      title="Edit device name"
                    >
                      ✏️
                    </button>
                  </span>
                )}
              </div>
              <p>
                <strong>ID:</strong> #{localDevice.id}
              </p>
              <p>
                <strong>IP:</strong> {localDevice.ip}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Pending Connection Requests */}
      {pendingConnections.length > 0 && (
        <div className="pending-connections">
          <h2>Pending Connection Requests</h2>
          <ul>
            {pendingConnections.map((device) => (
              <li key={device.id} className="device-item pending">
                <div className="device-avatar">🔔</div>
                <div className="device-details">
                  <p>
                    <strong>{device.name}</strong> wants to connect
                  </p>
                  <p>IP: {device.ip}</p>
                </div>
                <div className="connection-actions">
                  <button
                    className="accept-button"
                    onClick={() => acceptConnection(device.id)}
                  >
                    ✓ Accept
                  </button>
                  <button
                    className="deny-button"
                    onClick={() => denyConnection(device.id)}
                  >
                    ✕ Deny
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="connected-devices">
        <h2>Connected Devices</h2>
        {devices.length === 0 ? (
          <p className="no-devices">No connected devices</p>
        ) : (
          <ul>
            {devices.map((device) => (
              <li key={device.id} className="device-item">
                <div className="device-avatar">🖥️</div>
                <div className="device-details">
                  <p>
                    <strong>{device.name}</strong> (#{device.id})
                  </p>
                  <p>IP: {device.ip}</p>
                </div>
                <button
                  className="remove-button"
                  onClick={() => removeDevice(device.id)}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="available-devices">
        <div className="section-header">
          <h2>Available Devices</h2>
          <button
            className="refresh-button"
            onClick={discoverDevices}
            disabled={isDiscovering}
          >
            {isDiscovering ? "🔄 Scanning..." : "🔍 Scan Network"}
          </button>
        </div>
        {availableDevices.length === 0 ? (
          <p className="no-devices">
            {isDiscovering
              ? "Scanning for devices..."
              : "No devices found. Click 'Scan Network' to search."}
          </p>
        ) : (
          <ul>
            {availableDevices.map((device) => (
              <li key={device.id} className="device-item available">
                <div className="device-avatar">📱</div>
                <div className="device-details">
                  <p>
                    <strong>{device.name}</strong>
                  </p>
                  <p>IP: {device.ip}</p>
                </div>
                <button
                  className="connect-button"
                  onClick={() => sendConnectionRequest(device)}
                >
                  Connect
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="add-device">
        <h2>Add Device</h2>
        <input
          type="text"
          placeholder="Enter IP address or #Tag"
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              addDevice((e.target as HTMLInputElement).value);
              (e.target as HTMLInputElement).value = "";
            }
          }}
        />
      </div>
    </div>
  );
};

export default SettingsPage;
