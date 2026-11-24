use std::fmt::Write;

fn col_bytes(col: u32, buf: &mut [u8; 8]) -> usize {
    let mut n = u64::from(col) + 1;
    let mut i = buf.len();
    while n > 0 {
        n -= 1;
        i -= 1;
        buf[i] = b'A' + (n % 26) as u8;
        n /= 26;
    }
    i
}

pub fn col_to_letters(col: u32) -> String {
    let mut buf = [0u8; 8];
    let start = col_bytes(col, &mut buf);
    std::str::from_utf8(&buf[start..])
        .unwrap_or_default()
        .to_string()
}

pub fn to_a1(row: u32, col: u32) -> String {
    let mut buf = [0u8; 8];
    let start = col_bytes(col, &mut buf);
    let letters = std::str::from_utf8(&buf[start..]).unwrap_or_default();
    let mut out = String::with_capacity(letters.len() + 10);
    out.push_str(letters);
    let _ = write!(out, "{}", u64::from(row) + 1);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colunas() {
        assert_eq!(col_to_letters(0), "A");
        assert_eq!(col_to_letters(25), "Z");
        assert_eq!(col_to_letters(26), "AA");
        assert_eq!(col_to_letters(701), "ZZ");
        assert_eq!(col_to_letters(702), "AAA");
        assert_eq!(col_to_letters(16383), "XFD");
    }

    #[test]
    fn a1() {
        assert_eq!(to_a1(0, 0), "A1");
        assert_eq!(to_a1(1, 1), "B2");
        assert_eq!(to_a1(1048575, 16383), "XFD1048576");
        assert!(!to_a1(u32::MAX, u32::MAX).is_empty());
    }
}
