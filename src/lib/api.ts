import { invoke } from '@tauri-apps/api/core';
import type { Order, AppSettings, PrinterInfo } from './types';

export const api = {
  // Orders
  createOrder: (customerName: string, content: string, orderType: string): Promise<number> =>
    invoke<number>('create_order', { customerName, content, orderType }),

  getOrders: (orderType: string): Promise<Order[]> =>
    invoke<Order[]>('get_orders', { orderType }),

  getOrder: (id: number): Promise<Order> =>
    invoke<Order>('get_order', { id }),

  updateOrder: (id: number, customerName: string, content: string): Promise<void> =>
    invoke<void>('update_order', { id, customerName, content }),

  deleteOrder: (id: number): Promise<void> =>
    invoke<void>('delete_order', { id }),

  purgeOldOrders: (): Promise<number> =>
    invoke<number>('purge_old_orders'),

  // Print
  listPrinters: (): Promise<PrinterInfo[]> =>
    invoke<PrinterInfo[]>('list_printers'),

  printOrder: (orderId: number): Promise<void> =>
    invoke<void>('print_order', { orderId }),

  previewReceipt: (orderId: number): Promise<string> =>
    invoke<string>('preview_receipt', { orderId }),

  reprintOrder: (orderId: number): Promise<void> =>
    invoke<void>('reprint_order', { orderId }),

  testPrint: (): Promise<void> =>
    invoke<void>('test_print'),

  // Settings
  getSettings: (): Promise<AppSettings> =>
    invoke<AppSettings>('get_settings'),

  saveSettings: (settings: AppSettings): Promise<void> =>
    invoke<void>('save_settings', { settings }),
};
