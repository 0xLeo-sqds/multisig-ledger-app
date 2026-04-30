use arrayvec::ArrayString;
use core::fmt::Write;

/// Known token decimals for human-readable display.
pub struct TokenInfo {
    pub symbol: &'static str,
    pub decimals: u8,
}

/// Well-known Solana token mints and their decimal places.
/// Native SOL uses 9 decimals.
pub const SOL_DECIMALS: u8 = 9;

/// Hardcoded token registry for clear signing display.
/// Returns None for unknown mints.
pub fn lookup_token(_mint: &[u8; 32]) -> Option<TokenInfo> {
    None
}

/// Format a u64 amount with the given number of decimal places.
/// Uses integer-only arithmetic — never f64.
///
/// Example: format_amount(1_500_000_000, 9) → "1.500000000"
pub fn format_amount(raw: u64, decimals: u8) -> ArrayString<32> {
    let mut buf = ArrayString::<32>::new();

    if decimals == 0 {
        let _ = write!(&mut buf, "{}", raw);
        return buf;
    }

    let divisor = 10u64.pow(decimals as u32);
    let integer_part = raw / divisor;
    let frac_part = raw % divisor;

    let _ = write!(&mut buf, "{}.{:0>width$}", integer_part, frac_part, width = decimals as usize);

    buf
}

/// Format a SOL amount from lamports.
pub fn format_sol(lamports: u64) -> ArrayString<32> {
    let mut amount = format_amount(lamports, SOL_DECIMALS);
    let _ = amount.try_push_str(" SOL");
    amount
}
