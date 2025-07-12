export interface ClipboardItem {
  id: string;
  content: string;
  timestamp: string;
  device: string;
  content_type: "text" | "image";
}

export interface ClipboardStore {
  items: ClipboardItem[];
}
