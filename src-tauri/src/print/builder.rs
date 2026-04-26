use crate::db::orders::Order;
use crate::db::settings::AppSettings;
use escpos::driver::Driver;
use escpos::errors::Result as EscResult;
use escpos::printer::Printer;
use escpos::utils::*;
use std::sync::{Arc, Mutex};

/// In-memory ESC/POS byte buffer driver.
///
/// Implements the `escpos::driver::Driver` trait by collecting all written
/// bytes into a `Vec<u8>` instead of sending them to a physical device.
#[derive(Clone)]
pub struct VecDriver {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl VecDriver {
    pub fn new() -> Self {
        Self {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        Arc::try_unwrap(self.buf)
            .unwrap_or_else(|arc| {
                let guard = arc.lock().expect("VecDriver into_bytes lock poisoned");
                Mutex::new(guard.clone())
            })
            .into_inner()
            .unwrap_or_default()
    }
}

impl Driver for VecDriver {
    fn name(&self) -> String {
        "vec".to_string()
    }

    fn write(&self, data: &[u8]) -> EscResult<()> {
        let mut buf = self.buf.lock().expect("VecDriver lock poisoned");
        buf.extend_from_slice(data);
        Ok(())
    }

    fn read(&self, _buf: &mut [u8]) -> EscResult<usize> {
        Ok(0)
    }

    fn flush(&self) -> EscResult<()> {
        Ok(())
    }
}

/// Number of printable characters per line for each paper width.
///
/// TM-U220 (9-pin, 76mm): Font A = 40 cols. Using 42 causes the last 2 chars of
/// a 42-char line to overflow, splitting " [ ]" as "[" on line N, "]" on line N+1.
fn char_width(paper_size: &str) -> usize {
    match paper_size {
        "58mm" => 32,
        "75mm" => 40, // TM-U220 / 76mm dot-matrix: 40 cols (Font A)
        _ => 48,       // 80mm default
    }
}

/// Effective characters per line for item text, accounting for font width scaling.
/// Double-width fonts take 2 hardware columns per character, so the wrap budget is halved.
fn effective_width(paper_size: &str, font_size: &str) -> usize {
    let base = char_width(paper_size);
    if is_double_wide(font_size) { base / 2 } else { base }
}

/// Returns true when the font size maps to a double-width ESC/POS mode.
fn is_double_wide(font_size: &str) -> bool {
    match font_size {
        "wide" | "large" => true,
        _ => font_size.parse::<u32>().unwrap_or(11) >= 18,
    }
}

#[allow(dead_code)]
/// Returns true when the font size maps to a double-height ESC/POS mode (tall or large).
fn is_double_tall(font_size: &str) -> bool {
    match font_size {
        "tall" | "large" => true,
        _ => matches!(font_size.parse::<u32>().unwrap_or(11), 14..=17 | 22..),
    }
}

/// ESC/POS GS ! byte that encodes the requested content font size.
/// Supports both legacy keywords ("normal","tall","wide","large") and numeric pt values (8–24).
///
/// Numeric mapping:  ≤13 → normal (1×1) · 14–17 → tall (1×2) · 18–21 → wide (2×1) · 22+ → large (2×2)
fn content_font_byte(font_size: &str) -> u8 {
    match font_size {
        "normal" => return 0x00,
        "tall"   => return 0x10,
        "wide"   => return 0x01,
        "large"  => return 0x11,
        _ => {}
    }
    match font_size.parse::<u32>().unwrap_or(11) {
        0..=13  => 0x00,
        14..=17 => 0x10,
        18..=21 => 0x01,
        _       => 0x11,
    }
}

/// Checkbox area appended to the last line of each item: 1 space + box.
/// Provides room for a manual pen check-off on the printed receipt.
const CHECKBOX: &str = " [ ]";

/// Word-wrap a single item to `text_width` columns, then format as a receipt
/// line with dot-leaders and a checkbox:
///
/// ```text
/// 2 sak aci ......................... [ ]
/// teks yang sangat panjang sekali ini
/// dilanjutkan ke bawah ............. [ ]
/// ```
///
/// - Lines before the last: plain wrapped text (no dots, no checkbox)
/// - Last line: text + dot-leaders filling to `text_width` + CHECKBOX
fn format_item_line(item: &str, total_width: usize) -> Vec<String> {
    let checkbox_len = CHECKBOX.len();
    // Reserve space for checkbox; text body fits in the remaining columns
    let text_width = total_width.saturating_sub(checkbox_len);

    let mut out = Vec::new();

    if item.is_empty() {
        // Empty line → just a dot-leader row for blank items
        let dots: String = ".".repeat(text_width);
        out.push(format!("{dots}{CHECKBOX}"));
        return out;
    }

    let wrapped: Vec<String> = textwrap::wrap(item, text_width)
        .into_iter()
        .map(|s| s.into_owned())
        .collect();

    let last_idx = wrapped.len().saturating_sub(1);
    for (i, segment) in wrapped.iter().enumerate() {
        if i < last_idx {
            // Continuation lines: print as-is (no checkbox yet)
            out.push(segment.clone());
        } else {
            // Last (or only) segment: pad with dots then checkbox
            let dots_needed = text_width.saturating_sub(segment.len());
            let dots: String = ".".repeat(dots_needed);
            out.push(format!("{segment}{dots}{CHECKBOX}"));
        }
    }

    out
}

/// Word-wrap a single item without dot-leaders or checkbox. Used for tanda terima
/// content lines where free-text entries don't need a check-off box.
fn format_plain_line(item: &str, width: usize) -> Vec<String> {
    if item.is_empty() {
        return vec![String::new()];
    }
    textwrap::wrap(item, width)
        .into_iter()
        .map(|s| s.into_owned())
        .collect()
}

/// Build the receipt as a list of plain-text lines (no ESC/POS control bytes).
/// Used both by `build_receipt` (which then feeds the lines to the ESC/POS
/// printer) and by `build_receipt_preview` (plain text for on-screen display).
pub fn build_receipt_lines(order: &Order, settings: &AppSettings) -> Vec<String> {
    let width = char_width(&settings.paper_size);
    let eff_width = effective_width(&settings.paper_size, &settings.content_font_size);

    let mut lines: Vec<String> = Vec::new();

    if order.order_type == "receipt" {
        // ── Tanda Terima layout ───────────────────────────────────────────────
        // Line 0 = "TANDA TERIMA" → rendered as .preview-cname (big, bold, centred)
        lines.push("TANDA TERIMA".to_string());
        lines.push(format!("  Diterima dari : {}", order.customer_name));
        lines.push(format!("  Tanggal       : {}", order.created_at));
        lines.push(String::new()); // blank before items

        for item in order.content.lines() {
            for line in format_plain_line(item, eff_width) {
                lines.push(format!("\x01{}", line));
            }
        }

        lines.push(String::new()); // blank after items

        // Signature area: 3 blank lines + underline + name placeholder
        lines.push(String::new());
        lines.push(String::new());
        lines.push(String::new());
        let sig = "_".repeat(20);
        let sig_pad = (width.saturating_sub(20)) / 2;
        lines.push(format!("{:>w$}", sig, w = sig_pad + 20));
        let label = "(Nama Terang)";
        let label_pad = (width.saturating_sub(label.len())) / 2;
        lines.push(format!("{:>w$}", label, w = label_pad + label.len()));
    } else {
        // ── Pesanan (order) layout ────────────────────────────────────────────
        // Line 0 — customer name, raw (no padding).
        // Centering is handled by the frontend via CSS `text-align: center` on the
        // `.preview-cname` element, so manual padding here would fight with the
        // 2× font-size used for rendering and produce incorrect visual offset.
        lines.push(order.customer_name.clone());

        lines.push(format!("  Tanggal  : {}", order.created_at));
        lines.push(String::new()); // blank line before items

        for item in order.content.lines() {
            for line in format_item_line(item, eff_width) {
                lines.push(format!("\x01{}", line));
            }
        }

        lines.push(String::new()); // blank line after items

        // Footer
        if !settings.footer_text.is_empty() {
            let pad = (width.saturating_sub(settings.footer_text.len())) / 2;
            lines.push(format!(
                "{:>width$}",
                settings.footer_text,
                width = pad + settings.footer_text.len()
            ));
        }

        // Store name at bottom, centred within the 2-space indented area
        if !settings.store_name.is_empty() {
            let available = width.saturating_sub(2);
            let pad = (available.saturating_sub(settings.store_name.len())) / 2;
            lines.push(format!(
                "  {:>width$}",
                settings.store_name,
                width = pad + settings.store_name.len()
            ));
        }

        // PC name
        if !settings.pc_name.is_empty() {
            lines.push(format!("PC: {}", settings.pc_name));
        }
    }

    lines
}

/// Build ESC/POS receipt bytes for the given order and settings.
pub fn build_receipt(order: &Order, settings: &AppSettings) -> Vec<u8> {
    let width = char_width(&settings.paper_size);
    let eff_width = effective_width(&settings.paper_size, &settings.content_font_size);
    let font_byte = content_font_byte(&settings.content_font_size);

    let driver = VecDriver::new();
    let driver_clone = driver.clone();

    {
        let mut printer = Printer::new(driver, Protocol::default(), None);

        let result: EscResult<()> = (|| {
            printer.init()?;

            if order.order_type == "receipt" {
                // ── TANDA TERIMA header ───────────────────────────────────────
                printer.justify(JustifyMode::CENTER)?;
                printer.custom(b"\x1D\x21\x10")?; // double height
                printer.bold(true)?;
                printer.writeln("TANDA TERIMA")?;
                printer.bold(false)?;
                printer.custom(b"\x1D\x21\x00")?;
                printer.justify(JustifyMode::LEFT)?;

                printer.writeln(&format!("  Diterima dari : {}", order.customer_name))?;
                printer.writeln(&format!("  Tanggal       : {}", order.created_at))?;
                printer.writeln("")?;

                // ── Content items ─────────────────────────────────────────────
                printer.custom(&[0x1D, 0x21, font_byte])?;
                for item in order.content.lines() {
                    for line in format_plain_line(item, eff_width) {
                        printer.writeln(&line)?;
                    }
                }
                printer.custom(b"\x1D\x21\x00")?;
                printer.writeln("")?;

                // ── Signature area ────────────────────────────────────────────
                printer.writeln("")?;
                printer.writeln("")?;
                printer.writeln("")?;
                let sig = "_".repeat(20);
                let sig_pad = (width.saturating_sub(20)) / 2;
                printer.writeln(&format!("{:>w$}", sig, w = sig_pad + 20))?;
                let label = "(Nama Terang)";
                let label_pad = (width.saturating_sub(label.len())) / 2;
                printer.writeln(&format!("{:>w$}", label, w = label_pad + label.len()))?;
            } else {
                // ── Customer name: centred, double-height, bold ───────────────
                printer.justify(JustifyMode::CENTER)?;
                // GS ! 0x10 → double height (height×2, width×1)
                printer.custom(b"\x1D\x21\x10")?;
                printer.bold(true)?;
                printer.writeln(&order.customer_name)?;
                printer.bold(false)?;
                // GS ! 0x00 → reset to normal size
                printer.custom(b"\x1D\x21\x00")?;
                printer.justify(JustifyMode::LEFT)?;

                printer.writeln(&format!("  Tanggal  : {}", order.created_at))?;
                printer.writeln("")?;

                // ── Content items with configured font size ───────────────────
                printer.custom(&[0x1D, 0x21, font_byte])?;
                for item in order.content.lines() {
                    for line in format_item_line(item, eff_width) {
                        printer.writeln(&line)?;
                    }
                }
                printer.custom(b"\x1D\x21\x00")?;
                printer.writeln("")?;

                // ── Footer ────────────────────────────────────────────────────
                if !settings.footer_text.is_empty() {
                    let pad = (width.saturating_sub(settings.footer_text.len())) / 2;
                    printer.writeln(&format!(
                        "{:>w$}",
                        settings.footer_text,
                        w = pad + settings.footer_text.len()
                    ))?;
                }

                // ── Store name: centred via ESC/POS alignment ─────────────────
                if !settings.store_name.is_empty() {
                    printer.justify(JustifyMode::CENTER)?;
                    printer.writeln(&settings.store_name)?;
                    printer.justify(JustifyMode::LEFT)?;
                }

                // ── PC / kasir name ───────────────────────────────────────────
                if !settings.pc_name.is_empty() {
                    printer.writeln(&format!("PC: {}", settings.pc_name))?;
                }
            }

            printer.feeds(3 + settings.extra_feeds)?;
            if settings.auto_cut {
                printer.print_cut()?;
            } else {
                printer.print()?;
            }
            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("ESC/POS build error: {e}");
        }
    }

    driver_clone.into_bytes()
}

/// Return the receipt as a plain-text string for on-screen preview.
pub fn build_receipt_preview(order: &Order, settings: &AppSettings) -> String {
    build_receipt_lines(order, settings).join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::orders::Order;
    use crate::db::settings::AppSettings;

    fn default_settings() -> AppSettings {
        AppSettings {
            default_printer: String::new(),
            paper_size: "80mm".to_string(),
            store_name: String::new(),
            footer_text: String::new(),
            serial_baud_rate: 9600,
            auto_cut: false,
            pc_name: String::new(),
            content_font_size: "normal".to_string(),
            extra_feeds: 0,
        }
    }

    fn make_order(customer: &str, content: &str) -> Order {
        Order {
            id: 1,
            customer_name: customer.to_string(),
            content: content.to_string(),
            order_type: "order".to_string(),
            created_at: "2026-04-24 10:00:00".to_string(),
        }
    }

    fn make_receipt(customer: &str, content: &str) -> Order {
        Order {
            id: 2,
            customer_name: customer.to_string(),
            content: content.to_string(),
            order_type: "receipt".to_string(),
            created_at: "2026-04-24 10:00:00".to_string(),
        }
    }

    // ── effective_width ───────────────────────────────────────────────────────

    #[test]
    fn effective_width_normal_and_tall_unchanged() {
        assert_eq!(effective_width("80mm", "normal"), 48);
        assert_eq!(effective_width("80mm", "tall"), 48);
        assert_eq!(effective_width("58mm", "normal"), 32);
        assert_eq!(effective_width("58mm", "tall"), 32);
        assert_eq!(effective_width("75mm", "normal"), 40);
        assert_eq!(effective_width("75mm", "tall"), 40);
    }

    #[test]
    fn effective_width_wide_and_large_halved() {
        assert_eq!(effective_width("80mm", "wide"), 24);
        assert_eq!(effective_width("80mm", "large"), 24);
        assert_eq!(effective_width("58mm", "wide"), 16);
        assert_eq!(effective_width("75mm", "wide"), 20);
        assert_eq!(effective_width("75mm", "large"), 20);
    }

    #[test]
    fn item_line_75mm_never_exceeds_40_chars() {
        // Regression guard: 75mm=42 caused " [ ]" to split across lines on TM-U220.
        // Every formatted line must be ≤ 40 chars so nothing overflows the printer margin.
        let total_width = char_width("75mm"); // must be 40
        let items = [
            "2slop signatur",
            "1slop prima kertas",
            "3slop prima biasa",
            "2slop aroma bold 16",
            "3slopmoza",
            "3slop ggf",
            "3slop ggm 12",
            "2slop aroma bold 12",
            "2slop surya 12 coklat",
            // Long item that wraps
            "Pesanan dengan nama barang yang sangat panjang sekali sampai harus ke baris berikutnya",
        ];
        for item in &items {
            for line in format_item_line(item, total_width) {
                assert!(
                    line.len() <= total_width,
                    "item '{item}' produced line len {} > {total_width}: '{line}'",
                    line.len()
                );
            }
        }
    }

    // ── content_font_byte ─────────────────────────────────────────────────────

    #[test]
    fn content_font_byte_values() {
        // Legacy keywords
        assert_eq!(content_font_byte("normal"), 0x00);
        assert_eq!(content_font_byte("tall"),   0x10);
        assert_eq!(content_font_byte("wide"),   0x01);
        assert_eq!(content_font_byte("large"),  0x11);
        assert_eq!(content_font_byte(""),       0x00); // unknown → normal
        // Numeric pt values
        assert_eq!(content_font_byte("8"),  0x00); // normal
        assert_eq!(content_font_byte("11"), 0x00); // normal
        assert_eq!(content_font_byte("13"), 0x00); // normal (< 14)
        assert_eq!(content_font_byte("14"), 0x10); // tall
        assert_eq!(content_font_byte("17"), 0x10); // tall (< 18)
        assert_eq!(content_font_byte("18"), 0x01); // wide
        assert_eq!(content_font_byte("21"), 0x01); // wide (< 22)
        assert_eq!(content_font_byte("22"), 0x11); // large
        assert_eq!(content_font_byte("24"), 0x11); // large
    }

    // ── char_width ────────────────────────────────────────────────────────────

    #[test]
    fn char_width_58mm() {
        assert_eq!(char_width("58mm"), 32);
    }

    #[test]
    fn char_width_75mm() {
        // TM-U220 (9-pin dot-matrix, 76mm paper) prints 40 columns per line in Font A.
        // 42 caused " [ ]" to overflow: "[" at end of line, "]" on next line.
        assert_eq!(char_width("75mm"), 40);
    }

    #[test]
    fn char_width_80mm_default() {
        assert_eq!(char_width("80mm"), 48);
        assert_eq!(char_width("unknown"), 48);
        assert_eq!(char_width(""), 48);
    }

    // ── format_item_line ─────────────────────────────────────────────────────

    #[test]
    fn format_item_line_empty_produces_dot_row() {
        let lines = format_item_line("", 32);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].ends_with(CHECKBOX));
        // All chars before checkbox should be dots
        let dots_part = &lines[0][..lines[0].len() - CHECKBOX.len()];
        assert!(dots_part.chars().all(|c| c == '.'));
    }

    #[test]
    fn format_item_line_short_fits_single_line() {
        let item = "2 sak beras";
        let lines = format_item_line(item, 32);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with(item));
        assert!(lines[0].ends_with(CHECKBOX));
        assert_eq!(lines[0].len(), 32);
    }

    #[test]
    fn format_item_line_long_wraps_and_checkbox_only_on_last() {
        // Width 32, CHECKBOX is 4 chars → text body 28 chars
        // "A very long item name that wraps" is >28 chars
        let item = "Pesanan dengan nama barang yang sangat panjang";
        let lines = format_item_line(item, 32);
        assert!(lines.len() >= 2, "expected wrap");
        // Only the last line should have the checkbox
        for line in &lines[..lines.len() - 1] {
            assert!(!line.ends_with(CHECKBOX), "continuation line has checkbox: {line}");
        }
        assert!(lines.last().unwrap().ends_with(CHECKBOX));
    }

    #[test]
    fn format_item_line_last_line_total_width_equals_total_width() {
        let item = "Short";
        let width = 48usize;
        let lines = format_item_line(item, width);
        assert_eq!(lines.last().unwrap().len(), width);
    }

    // ── build_receipt_lines ───────────────────────────────────────────────────

    #[test]
    fn receipt_lines_line0_is_raw_customer_name() {
        // build_receipt_lines returns the raw name on line 0.
        // Centering is handled by the frontend CSS (.preview-cname { text-align: center }),
        // so the Rust layer must NOT add any padding here.
        let order = make_order("Pak Budi", "1 sak beras");
        let settings = default_settings();
        let lines = build_receipt_lines(&order, &settings);
        assert_eq!(lines[0], "Pak Budi");
    }

    #[test]
    fn receipt_lines_line1_is_tanggal() {
        let order = make_order("X", "item");
        let settings = default_settings();
        let lines = build_receipt_lines(&order, &settings);
        assert!(lines[1].contains("Tanggal"), "line 1 should contain 'Tanggal'");
        assert!(lines[1].contains("2026-04-24 10:00:00"));
    }

    #[test]
    fn receipt_lines_blank_line_before_items() {
        let order = make_order("X", "item");
        let settings = default_settings();
        let lines = build_receipt_lines(&order, &settings);
        assert_eq!(lines[2], "");
    }

    #[test]
    fn receipt_lines_items_contain_checkbox() {
        let order = make_order("X", "2 sak beras\n1 sak terigu");
        let settings = default_settings();
        let lines = build_receipt_lines(&order, &settings);
        let item_lines: Vec<&String> = lines.iter().filter(|l| l.contains("[ ]")).collect();
        assert_eq!(item_lines.len(), 2);
    }

    #[test]
    fn receipt_lines_footer_text_appears_when_set() {
        let order = make_order("X", "item");
        let mut settings = default_settings();
        settings.footer_text = "Terima kasih".to_string();
        let lines = build_receipt_lines(&order, &settings);
        assert!(lines.iter().any(|l| l.contains("Terima kasih")));
    }

    #[test]
    fn receipt_lines_store_name_appears_after_items() {
        let order = make_order("X", "item");
        let mut settings = default_settings();
        settings.store_name = "Toko Maju".to_string();
        let lines = build_receipt_lines(&order, &settings);
        // Store name must appear after all item lines (which contain "[ ]")
        let last_item_idx = lines.iter().rposition(|l| l.contains("[ ]")).unwrap();
        let store_idx = lines.iter().rposition(|l| l.contains("Toko Maju")).unwrap();
        assert!(store_idx > last_item_idx);
    }

    #[test]
    fn receipt_lines_store_name_centering_respects_2space_prefix() {
        let order = make_order("X", "item");
        let mut settings = default_settings();
        settings.paper_size = "80mm".to_string(); // width = 48
        settings.store_name = "AB".to_string();   // 2 chars
        let lines = build_receipt_lines(&order, &settings);
        let store_line = lines.iter().find(|l| l.contains("AB")).unwrap();
        // "  " prefix + pad + "AB" — total line length should be ≤ 48 chars
        assert!(store_line.len() <= 48, "store line too wide: '{store_line}'");
        // There should be spaces before "AB" (centred, not left-aligned)
        assert!(store_line.trim_start().starts_with("AB") || store_line.contains(" AB"),
            "store name not padded: '{store_line}'");
    }

    #[test]
    fn receipt_lines_pc_name_appears_last_when_set() {
        let order = make_order("X", "item");
        let mut settings = default_settings();
        settings.pc_name = "Kasir 1".to_string();
        let lines = build_receipt_lines(&order, &settings);
        let last = lines.last().unwrap();
        assert!(last.contains("Kasir 1"));
        assert!(last.starts_with("PC:"));
    }

    #[test]
    fn receipt_lines_wide_font_wraps_at_half_columns() {
        // 80mm = 48 cols, wide font → eff_width = 24; CHECKBOX = 4 → text body = 20
        let item = "A".repeat(25); // 25 chars — would fit in 48 but not in 24
        let order = make_order("X", &item);
        let mut settings = default_settings();
        settings.content_font_size = "wide".to_string();
        let lines = build_receipt_lines(&order, &settings);
        // At least one continuation line must exist (item wraps at 20 chars)
        let has_wrap = lines.iter().any(|l| {
            let content = l.strip_prefix('\x01').unwrap_or(l.as_str());
            content.contains("[ ]") && content.len() <= 24
        });
        assert!(has_wrap, "wide font item line should be ≤24 chars with checkbox");
    }

    #[test]
    fn receipt_lines_no_store_no_footer_no_pc_when_empty() {
        let order = make_order("X", "item");
        let settings = default_settings(); // all empty
        let lines = build_receipt_lines(&order, &settings);
        assert!(!lines.iter().any(|l| l.starts_with("PC:")));
    }

    // ── VecDriver ─────────────────────────────────────────────────────────────

    #[test]
    fn vec_driver_write_and_into_bytes() {
        let driver = VecDriver::new();
        driver.write(b"hello").unwrap();
        driver.write(b" world").unwrap();
        let bytes = driver.into_bytes();
        assert_eq!(bytes, b"hello world");
    }

    #[test]
    fn vec_driver_clone_shares_buffer() {
        let driver = VecDriver::new();
        let clone = driver.clone();
        driver.write(b"abc").unwrap();
        // clone shares the same Arc<Mutex<Vec<u8>>> — both see the write
        let bytes = clone.into_bytes();
        assert_eq!(bytes, b"abc");
    }

    #[test]
    fn vec_driver_flush_and_read_are_noop() {
        let driver = VecDriver::new();
        driver.flush().unwrap();
        let mut buf = [0u8; 4];
        let n = driver.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    // ── build_receipt ─────────────────────────────────────────────────────────

    #[test]
    fn build_receipt_returns_nonempty_bytes() {
        let order = make_order("Pak Budi", "2 sak beras");
        let settings = default_settings();
        let bytes = build_receipt(&order, &settings);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn build_receipt_starts_with_esc_pos_init() {
        let order = make_order("X", "item");
        let settings = default_settings();
        let bytes = build_receipt(&order, &settings);
        // ESC/POS init sequence: ESC @ = 0x1B 0x40
        assert!(
            bytes.windows(2).any(|w| w == [0x1B, 0x40]),
            "missing ESC @ init sequence"
        );
    }

    #[test]
    fn build_receipt_contains_center_alignment_for_name() {
        let order = make_order("Budi", "item");
        let settings = default_settings();
        let bytes = build_receipt(&order, &settings);
        // ESC a 1 = centre alignment: 0x1B 0x61 0x01
        assert!(
            bytes.windows(3).any(|w| w == [0x1B, 0x61, 0x01]),
            "missing ESC a 1 (centre alignment)"
        );
        // ESC a 0 = left alignment restored: 0x1B 0x61 0x00
        assert!(
            bytes.windows(3).any(|w| w == [0x1B, 0x61, 0x00]),
            "missing ESC a 0 (left alignment restore)"
        );
    }

    #[test]
    fn build_receipt_contains_double_height_command() {
        let order = make_order("Besar", "item");
        let settings = default_settings();
        let bytes = build_receipt(&order, &settings);
        // GS ! 0x10 = double height
        assert!(
            bytes.windows(3).any(|w| w == [0x1D, 0x21, 0x10]),
            "missing double-height GS!0x10"
        );
        // GS ! 0x00 = reset size
        assert!(
            bytes.windows(3).any(|w| w == [0x1D, 0x21, 0x00]),
            "missing size-reset GS!0x00"
        );
    }

    #[test]
    fn build_receipt_content_font_byte_emitted() {
        let order = make_order("X", "item");
        let mut settings = default_settings();
        settings.content_font_size = "large".to_string(); // GS ! 0x11
        let bytes = build_receipt(&order, &settings);
        // GS ! 0x11 = double width + double height for content
        assert!(
            bytes.windows(3).any(|w| w == [0x1D, 0x21, 0x11]),
            "missing GS!0x11 (large font)"
        );
    }

    #[test]
    fn build_receipt_auto_cut_emits_cut_command() {
        let order = make_order("X", "item");
        let mut settings = default_settings();
        settings.auto_cut = true;
        let bytes_cut = build_receipt(&order, &settings);
        settings.auto_cut = false;
        let bytes_no_cut = build_receipt(&order, &settings);
        // With auto_cut the byte stream should be longer (cut command appended)
        assert!(bytes_cut.len() > bytes_no_cut.len());
    }

    #[test]
    fn build_receipt_extra_feeds_changes_esc_d_value() {
        // feeds() emits ESC d n = [0x1B, 0x64, n] — 3 bytes, only n changes.
        // Verify the correct n = (3 + extra_feeds) appears in the byte stream.
        let order = make_order("X", "item");
        let mut settings = default_settings();
        for extra in [0u8, 1, 3, 5] {
            settings.extra_feeds = extra;
            let bytes = build_receipt(&order, &settings);
            let expected_n = 3 + extra;
            assert!(
                bytes.windows(3).any(|w| w == [0x1B, 0x64, expected_n]),
                "extra_feeds={extra}: expected ESC d {expected_n} ([0x1B,0x64,{expected_n:#04x}]) in output"
            );
        }
    }

    #[test]
    fn build_receipt_extra_feeds_zero_same_as_default() {
        let order = make_order("X", "item");
        let settings = default_settings(); // extra_feeds = 0
        let bytes_default = build_receipt(&order, &settings);
        let mut s2 = default_settings();
        s2.extra_feeds = 0;
        assert_eq!(bytes_default, build_receipt(&order, &s2));
    }

    // ── build_receipt_preview ─────────────────────────────────────────────────

    #[test]
    fn build_receipt_preview_contains_customer_name() {
        let order = make_order("DK PASAR", "5 sak beras");
        let settings = default_settings();
        let preview = build_receipt_preview(&order, &settings);
        assert!(preview.contains("DK PASAR"));
    }

    // ── tanda terima layout ───────────────────────────────────────────────────

    #[test]
    fn receipt_lines_tanda_terima_line0_is_header() {
        let order = make_receipt("Toko Maju", "Jenis : Retur\nGudang : A");
        let lines = build_receipt_lines(&order, &default_settings());
        assert_eq!(lines[0], "TANDA TERIMA");
    }

    #[test]
    fn receipt_lines_tanda_terima_has_diterima_dari() {
        let order = make_receipt("Pak Budi", "Jenis : Retur");
        let lines = build_receipt_lines(&order, &default_settings());
        assert!(
            lines.iter().any(|l| l.contains("Diterima dari") && l.contains("Pak Budi")),
            "expected 'Diterima dari : Pak Budi' line"
        );
    }

    #[test]
    fn receipt_lines_tanda_terima_has_signature_line() {
        let order = make_receipt("X", "Jenis : Titip");
        let lines = build_receipt_lines(&order, &default_settings());
        assert!(
            lines.iter().any(|l| l.contains("____________________")),
            "expected signature underline"
        );
        assert!(
            lines.iter().any(|l| l.contains("(Nama Terang)")),
            "expected '(Nama Terang)' label"
        );
    }

    #[test]
    fn receipt_lines_tanda_terima_no_store_no_footer_no_pc() {
        let order = make_receipt("X", "Jenis : Retur");
        let mut settings = default_settings();
        settings.store_name = "Toko".to_string();
        settings.footer_text = "Terima kasih".to_string();
        settings.pc_name = "Kasir 1".to_string();
        let lines = build_receipt_lines(&order, &settings);
        // store, footer, and pc_name must not appear in tanda terima
        assert!(!lines.iter().any(|l| l.contains("Toko")));
        assert!(!lines.iter().any(|l| l.contains("Terima kasih")));
        assert!(!lines.iter().any(|l| l.contains("PC:")));
    }

    #[test]
    fn build_receipt_tanda_terima_returns_nonempty_bytes() {
        let order = make_receipt("Pak Budi", "Jenis : Retur\nGudang : A");
        let bytes = build_receipt(&order, &default_settings());
        assert!(!bytes.is_empty());
    }

    #[test]
    fn build_receipt_tanda_terima_has_center_alignment() {
        let order = make_receipt("X", "item");
        let bytes = build_receipt(&order, &default_settings());
        assert!(
            bytes.windows(3).any(|w| w == [0x1B, 0x61, 0x01]),
            "missing ESC a 1 (centre)"
        );
    }
}

/// Build a simple test receipt.
pub fn build_test_receipt(settings: &AppSettings) -> Vec<u8> {
    let test_order = Order {
        id: 0,
        customer_name: "Test Print".to_string(),
        content: "2 sak aci\n1 sak terigu\n10 kg gula los\n40 kg minyak curah goreng kemasan besar ekonomis\nPesanan dengan nama barang yang sangat panjang sekali sampai harus lanjut baris berikutnya\n5 karton teh botol sosro".to_string(),
        order_type: "order".to_string(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    build_receipt(&test_order, settings)
}
