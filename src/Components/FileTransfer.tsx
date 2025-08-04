import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface FileTransferProps {
  onFileAdded?: () => void;
}

export default function FileTransfer({ onFileAdded = () => {} }: FileTransferProps) {
  const [dragOver, setDragOver] = useState(false);
  const [uploading, setUploading] = useState(false);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    setUploading(true);

    try {
      const files = Array.from(e.dataTransfer.files);
      
      for (const file of files) {
        try {
          // Read file content as array buffer
          const arrayBuffer = await file.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // Convert to regular array for JSON serialization
          const content = Array.from(uint8Array);
          
          // Save the file and add to clipboard
          const savedPath = await invoke<string>("save_received_file", { 
            content, 
            fileName: file.name 
          });
          
          // Add the saved file to clipboard
          await invoke("add_file_to_clipboard", { filePath: savedPath });
          console.log("Added dropped file to clipboard:", file.name);
        } catch (error) {
          console.error("Failed to process dropped file:", file.name, error);
          alert(`Failed to process file: ${file.name}`);
        }
      }
      
      onFileAdded();
    } catch (error) {
      console.error("Failed to handle file drop:", error);
    } finally {
      setUploading(false);
    }
  };

  const handleFileSelect = async () => {
    try {
      setUploading(true);
      
      const files = await open({
        multiple: true,
        title: "Select files to add to clipboard",
      });
      
      if (files) {
        const filePaths = Array.isArray(files) ? files : [files];
        
        for (const filePath of filePaths) {
          try {
            await invoke("add_file_to_clipboard", { filePath });
            console.log("Added file to clipboard:", filePath);
          } catch (error) {
            console.error("Failed to add file:", error);
            alert(`Failed to add file: ${filePath}`);
          }
        }
        
        onFileAdded();
      }
    } catch (error) {
      console.error("Failed to select files:", error);
      alert("Failed to open file dialog");
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="file-transfer-container">
      <div
        className={`file-drop-zone ${dragOver ? "drag-over" : ""} ${uploading ? "uploading" : ""}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={handleFileSelect}
      >
        {uploading ? (
          <div className="file-transfer-status">
            <div className="loading-spinner"></div>
            <span>Adding files to clipboard...</span>
          </div>
        ) : (
          <div className="file-transfer-prompt">
            <div className="file-transfer-icon">📁</div>
            <div className="file-transfer-text">
              <p><strong>Drop files here or click to select</strong></p>
              <p>Files will be available for sync to connected devices</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}