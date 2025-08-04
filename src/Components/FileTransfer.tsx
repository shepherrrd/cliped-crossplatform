import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface FileTransferProps {
  onFileAdded?: () => void;
}

export default function FileTransfer({ onFileAdded = () => {} }: FileTransferProps) {
  const [dragOver, setDragOver] = useState(false);
  const [uploading, setUploading] = useState(false);
  
  // File size limit: 10MB for now (small media files, PDFs, etc.)
  const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB
  
  const isValidFileType = (fileName: string): boolean => {
    const allowedExtensions = [
      // Images
      '.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp',
      // Documents
      '.pdf', '.txt', '.doc', '.docx',
      // Small media
      '.mp3', '.wav', '.mp4', '.mov', '.avi',
      // Archives
      '.zip', '.rar', '.7z'
    ];
    
    const extension = fileName.toLowerCase().substring(fileName.lastIndexOf('.'));
    return allowedExtensions.includes(extension);
  };
  
  const validateFile = (file: File): string | null => {
    if (file.size > MAX_FILE_SIZE) {
      return `File "${file.name}" is too large. Maximum size is 10MB.`;
    }
    
    if (!isValidFileType(file.name)) {
      return `File type not supported: "${file.name}". Supported types: images, PDFs, documents, small media files.`;
    }
    
    return null;
  };

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
      let hasErrors = false;
      
      // Validate all files first
      for (const file of files) {
        const error = validateFile(file);
        if (error) {
          alert(error);
          hasErrors = true;
        }
      }
      
      if (hasErrors) {
        setUploading(false);
        return;
      }
      
      // Process valid files
      for (const file of files) {
        try {
          console.log(`Processing file: ${file.name} (${(file.size / 1024 / 1024).toFixed(2)}MB)`);
          
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
          alert(`Failed to process file: ${file.name}. Error: ${error}`);
        }
      }
      
      onFileAdded();
    } catch (error) {
      console.error("Failed to handle file drop:", error);
      alert("Failed to process dropped files");
    } finally {
      setUploading(false);
    }
  };

  const handleFileSelect = async () => {
    try {
      console.log("=== Starting file selection ===");
      setUploading(true);
      
      console.log("About to call open() function...");
      
      // Try using Tauri's built-in dialog via invoke
      const files = await invoke("show_open_dialog", {
        title: "Select a file",
        multiple: false,
      });
      
      console.log("File dialog returned:", files);
      
      if (files) {
        console.log("User selected file:", files);
        alert(`Selected file: ${files}`);
        
        try {
          console.log("Adding file to clipboard:", files);
          await invoke("add_file_to_clipboard", { filePath: files });
          console.log("Successfully added file to clipboard:", files);
          onFileAdded();
        } catch (error) {
          console.error("Failed to add file:", error);
          alert(`Failed to add file: ${files}. Error: ${error}`);
        }
      } else {
        console.log("No files selected or dialog was cancelled");
        alert("No files selected");
      }
    } catch (error) {
      console.error("ERROR in handleFileSelect:", error);
      alert(`Failed to open file dialog: ${error}`);
    } finally {
      console.log("=== Finishing file selection ===");
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
              <p>Max 10MB • Images, PDFs, Documents, Small Media</p>
              <p className="file-transfer-sync">Files sync automatically to connected devices</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}