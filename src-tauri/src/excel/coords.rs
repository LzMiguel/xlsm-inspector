/// Coluna (base 0) → letras: 0→A, 25→Z, 26→AA, 702→AAA.
pub fn col_to_letters(col: u32) -> String {
    let mut n = col + 1;
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    out.reverse();
    String::from_utf8_lossy(&out).into_owned()
}

/// (linha, coluna), ambas base 0 → "A1".
pub fn to_a1(row: u32, col: u32) -> String {
    format!("{}{}", col_to_letters(col), row + 1)
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
    }
}
