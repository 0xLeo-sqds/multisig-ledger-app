use arrayvec::ArrayString;
use core::fmt::Write;

/// Known token decimals for human-readable display.
#[allow(dead_code)]
pub struct TokenInfo {
    pub symbol: &'static str,
    pub decimals: u8,
}

/// Native SOL uses 9 decimals.
pub const SOL_DECIMALS: u8 = 9;

// Known token mint addresses (verified via base58 decode)
// USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
#[allow(dead_code)]
const USDC_MINT: [u8; 32] = [
    0xc6, 0xfa, 0x7a, 0xf3, 0xbe, 0xdb, 0xad, 0x39, 0x95, 0x86, 0x91, 0x9b, 0x22, 0x2b, 0x3f, 0x60,
    0xe2, 0x11, 0xf0, 0xbf, 0xbe, 0x38, 0x1a, 0x0a, 0x7c, 0x1c, 0x14, 0x0a, 0x62, 0xa6, 0xf6, 0x12,
];
// USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
#[allow(dead_code)]
const USDT_MINT: [u8; 32] = [
    0xce, 0x01, 0x0e, 0x60, 0xaf, 0xea, 0xc5, 0xe2, 0xa4, 0x6c, 0x78, 0xd4, 0xb5, 0x88, 0x2f, 0x7a,
    0x1e, 0x2e, 0x0c, 0xac, 0x97, 0x63, 0xf2, 0x62, 0xa4, 0x78, 0x54, 0x19, 0x47, 0xf3, 0xd3, 0xf3,
];

/// Hardcoded token registry for clear signing display.
#[allow(dead_code)]
pub fn lookup_token(mint: &[u8; 32]) -> Option<TokenInfo> {
    match mint {
        m if *m == USDC_MINT => Some(TokenInfo {
            symbol: "USDC",
            decimals: 6,
        }),
        m if *m == USDT_MINT => Some(TokenInfo {
            symbol: "USDT",
            decimals: 6,
        }),
        _ => None,
    }
}

/// Format a u64 amount with the given number of decimal places.
/// Strips trailing zeros after the decimal point.
/// Uses integer-only arithmetic, never f64.
///
/// Examples:
///   format_amount(50_000_000_000, 9) -> "50"
///   format_amount(1_500_000_000, 9) -> "1.5"
///   format_amount(1_234_567, 6) -> "1.234567"
///   format_amount(100, 6) -> "0.0001"
///   format_amount(0, 9) -> "0"
pub fn format_amount(raw: u64, decimals: u8) -> ArrayString<32> {
    let mut buf = ArrayString::<32>::new();

    if decimals == 0 || raw == 0 {
        let _ = write!(&mut buf, "{}", raw);
        return buf;
    }

    let divisor = 10u64.pow(decimals as u32);
    let integer_part = raw / divisor;
    let frac_part = raw % divisor;

    if frac_part == 0 {
        // No fractional part — just show integer
        let _ = write!(&mut buf, "{}", integer_part);
    } else {
        // Format with leading zeros, then strip trailing zeros
        let _ = write!(&mut buf, "{}.", integer_part);

        // Build fractional string with leading zeros
        let mut frac_buf = ArrayString::<20>::new();
        let _ = write!(
            &mut frac_buf,
            "{:0>width$}",
            frac_part,
            width = decimals as usize
        );

        // Strip trailing zeros
        let frac_str = frac_buf.as_str().trim_end_matches('0');
        let _ = buf.try_push_str(frac_str);
    }

    buf
}

/// Format a SOL amount from lamports, stripping trailing zeros.
pub fn format_sol(lamports: u64) -> ArrayString<32> {
    let mut amount = format_amount(lamports, SOL_DECIMALS);
    let _ = amount.try_push_str(" SOL");
    amount
}
