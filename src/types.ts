export interface ClipboardItem {
  id: string;
  content: string;
  timestamp: string;
  device: string;
  content_type: "text" | "image" | "file";
  file_path?: string;
  file_size?: number;
  file_name?: string;
}

export interface ClipboardStore {
  items: ClipboardItem[];
}
