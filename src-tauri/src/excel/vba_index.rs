use crate::model::VbaProcedure;

const MODIFIERS: [&str; 5] = ["public", "private", "friend", "static", "global"];

fn strip_strings_and_comments(line: &str) -> String {
    let mut code = String::with_capacity(line.len());
    let mut in_str = false;
    for c in line.chars() {
        match c {
            '"' => {
                in_str = !in_str;
                code.push(' ');
            }
            '\'' if !in_str => break,
            _ if in_str => code.push(' '),
            _ => code.push(c),
        }
    }
    code
}

fn words(line: &str) -> Vec<String> {
    let code = strip_strings_and_comments(line);
    let mut out = Vec::new();
    for stmt in code.split(':') {
        let stmt_words: Vec<String> = stmt
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        if stmt_words
            .first()
            .is_some_and(|f| f.eq_ignore_ascii_case("rem"))
        {
            break;
        }
        out.extend(stmt_words);
    }
    out
}

fn logical_lines(code: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut start = 0usize;
    let mut end = 0usize;

    for (i, line) in code.lines().enumerate() {
        let n = i + 1;
        let clean = strip_strings_and_comments(line);
        let trimmed = clean.trim_end();
        let continues = trimmed == "_"
            || trimmed
                .strip_suffix('_')
                .is_some_and(|before| before.ends_with(char::is_whitespace));
        let part = if continues {
            trimmed.strip_suffix('_').unwrap_or(trimmed).trim_end()
        } else {
            trimmed
        };

        if start == 0 {
            start = n;
        }
        if !cur.is_empty() && !part.is_empty() {
            cur.push(' ');
        }
        cur.push_str(part);
        end = n;

        if !continues {
            out.push((start, end, std::mem::take(&mut cur)));
            start = 0;
        }
    }
    if start != 0 {
        out.push((start, end, cur));
    }
    out
}

fn header(w: &[String]) -> Option<(String, String, String)> {
    let low = |i: usize| w.get(i).map(|s| s.to_lowercase());
    let mut i = 0;
    let mut vis = "Public";
    while let Some(m) = low(i).filter(|m| MODIFIERS.contains(&m.as_str())) {
        match m.as_str() {
            "private" => vis = "Private",
            "friend" => vis = "Friend",
            _ => {}
        }
        i += 1;
    }
    let (kind, name_at) = match low(i)?.as_str() {
        "sub" => ("Sub".to_string(), i + 1),
        "function" => ("Function".to_string(), i + 1),
        "property" => {
            let acc = match low(i + 1)?.as_str() {
                "get" => "Get",
                "let" => "Let",
                "set" => "Set",
                _ => return None,
            };
            (format!("Property {acc}"), i + 2)
        }
        _ => return None,
    };
    Some((kind, w.get(name_at)?.clone(), vis.to_string()))
}

fn closes(w: &[String], base: &str) -> bool {
    w.windows(2)
        .any(|p| p[0].eq_ignore_ascii_case("end") && p[1].eq_ignore_ascii_case(base))
}

pub fn index(code: &str) -> Vec<VbaProcedure> {
    let mut out: Vec<VbaProcedure> = Vec::new();
    let mut open: Option<String> = None;
    let mut last = 0;
    for (start_n, end_n, line) in logical_lines(code) {
        last = end_n;
        let w = words(&line);
        if open.is_none()
            && let Some((kind, name, visibility)) = header(&w)
        {
            open = Some(kind.split(' ').next().unwrap_or("sub").to_lowercase());
            out.push(VbaProcedure {
                name,
                kind,
                visibility,
                start_line: start_n,
                end_line: end_n,
            });
        }
        if let Some(base) = &open
            && closes(&w, base)
        {
            if let Some(p) = out.last_mut() {
                p.end_line = end_n;
            }
            open = None;
        }
    }
    if open.is_some()
        && let Some(p) = out.last_mut()
    {
        p.end_line = last;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    type Row = (String, String, String, usize, usize);

    fn rows(code: &str) -> Vec<Row> {
        index(code)
            .into_iter()
            .map(|p| (p.name, p.kind, p.visibility, p.start_line, p.end_line))
            .collect()
    }

    #[test]
    fn tipos_e_visibilidade() {
        let code = "Option Explicit\n\
Public Sub A()\n  x = 1\nEnd Sub\n\
Private Function B(ByVal n As Long) As Long\n  B = n\nEnd Function\n\
Static Sub C()\nEnd Sub\n\
Friend Property Get D() As Long\nEnd Property\n\
Property Let D(v As Long)\nEnd Property\n\
Public Property Set E(o As Object)\nEnd Property";
        let ex = |n: &str, k: &str, v: &str, s, e| -> Row { (n.into(), k.into(), v.into(), s, e) };
        assert_eq!(
            rows(code),
            vec![
                ex("A", "Sub", "Public", 2, 4),
                ex("B", "Function", "Private", 5, 7),
                ex("C", "Sub", "Public", 8, 9),
                ex("D", "Property Get", "Friend", 10, 11),
                ex("D", "Property Let", "Public", 12, 13),
                ex("E", "Property Set", "Public", 14, 15),
            ]
        );
    }

    #[test]
    fn end_sub_em_comentario_e_string_nao_fecha() {
        let code = "Sub A()\n  ' End Sub\n  Rem End Sub\n  s = \"End Sub\"\nEnd Sub";
        assert_eq!(rows(code)[0].4, 5);
    }

    #[test]
    fn declare_nao_e_procedimento() {
        let code = "Private Declare Function GetTickCount Lib \"kernel32\" () As Long\n\
Public Declare PtrSafe Sub Sleep Lib \"kernel32\" (ByVal ms As Long)\nSub Real()\nEnd Sub";
        let got = rows(code);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, "Real");
    }

    #[test]
    fn procedimento_de_uma_linha_e_sem_end() {
        let code = "Sub Um(): x = 1: End Sub\nFunction Dois()\n  Dois = 1";
        let lines: Vec<_> = rows(code).into_iter().map(|r| (r.3, r.4)).collect();
        assert_eq!(lines, vec![(1, 1), (2, 3)]);
    }

    #[test]
    fn nomes_com_acento_e_sem_codigo() {
        assert_eq!(rows("Sub Ação()\nEnd Sub")[0].0, "Ação");
        assert!(index("").is_empty());
        assert!(index("Dim x As Long\n' Sub Fake()").is_empty());
    }

    #[test]
    fn continuacao_de_linha_e_rem_apos_dois_pontos() {
        let code = "Public Sub _\n  MeuProc()\n  x = 1: Rem End Sub\n  y = 2\nEnd Sub";
        let got = rows(code);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, "MeuProc");
        assert_eq!((got[0].3, got[0].4), (1, 5));
    }
}
