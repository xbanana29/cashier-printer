export interface Order {
  id: number;
  customer_name: string;
  content: string;
  order_type: string;
  created_at: string;
}

export interface AppSettings {
  default_printer: string;
  paper_size: '58mm' | '75mm' | '80mm';
  store_name: string;
  footer_text: string;
  /** Baud rate for serial/COM port connections. Ignored for CUPS/network. */
  serial_baud_rate: number;
  /** Send auto-cut command after each receipt. Disable for TM-U220 without cutter. */
  auto_cut: boolean;
  /** Workstation display name shown in history and at the bottom of receipts. */
  pc_name: string;
  /** ESC/POS character size for order content lines: "normal" | "tall" | "wide" | "large" */
  content_font_size: string;
  /** Extra blank lines fed after the receipt (0–5) to push paper past the print head. */
  extra_feeds: number;
}

export interface PrinterInfo {
  name: string;
  is_default: boolean;
  connection_type: string;
}

export interface PeerInfo {
  device_id: string;
  pc_name: string;
  addr: string;
  last_seen: number;
  orders_synced: number;
}

export interface AppError {
  type: 'Database' | 'Print' | 'PrinterNotFound' | 'Settings' | 'NotFound';
  message: string;
}
