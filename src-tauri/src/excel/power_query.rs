use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};
use quick_xml::{Reader, XmlVersion, events::BytesStart, events::Event};
use zip::ZipArchive;

use super::workbook::io_err;
use crate::error::AppError;
use crate::model::{ConnectionInfo, PowerQuery, PowerQueryItem};

fn corrupt(e: impl std::fmt::Display) -> AppError {
    AppError::CorruptedFile(e.to_string())
}

/// Só leitura. Sem consultas nem conexões → `PowerQuery` vazio (não é erro).
pub fn extract_file(path: &Path) -> Result<PowerQuery, AppError> {
    let file = File::open(path).map_err(io_err)?;
    extract(ZipArchive::new(file).map_err(corrupt)?)
}

pub fn extract<R: Read + Seek>(mut zip: ZipArchive<R>) -> Result<PowerQuery, AppError> {
    let mut out = PowerQuery::default();

    let entries: Vec<String> = zip
        .file_names()
        .map(|name| name.map(|name| name.into_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(corrupt)?;

    let items: Vec<String> = entries
        .iter()
        .filter(|n| n.starts_with("customXml/item") && n.ends_with(".xml"))
        .cloned()
        .collect();

    if entries.iter().any(|n| n == "xl/connections.xml") {
        out.connections =
            parse_connections(&decode_text(&read_entry(&mut zip, "xl/connections.xml")?));
    }

    for name in items {
        let text = decode_text(&read_entry(&mut zip, &name)?);
        if let Some(b64) = datamashup_base64(&text) {
            out.queries = queries_from_mashup(&b64)?;
            break;
        }
    }

    Ok(out)
}

fn read_entry<R: Read + Seek>(zip: &mut ZipArchive<R>, name: &str) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    zip.by_name(name)
        .map_err(corrupt)?
        .read_to_end(&mut buf)
        .map_err(corrupt)?;
    Ok(buf)
}

/// Os `customXml/item*.xml` do Excel costumam ser UTF-16 com BOM.
fn decode_text(bytes: &[u8]) -> String {
    match bytes {
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn utf16(bytes: &[u8], f: fn([u8; 2]) -> u16) -> String {
    let units: Vec<u16> = bytes.chunks_exact(2).map(|c| f([c[0], c[1]])).collect();
    String::from_utf16_lossy(&units)
}

/// Texto do elemento `<DataMashup ...>BASE64</DataMashup>`.
fn datamashup_base64(xml: &str) -> Option<String> {
    let start = xml.find("<DataMashup")?;
    let open_end = start + xml[start..].find('>')? + 1;
    let close = open_end + xml[open_end..].find("</DataMashup")?;
    Some(xml[open_end..close].split_whitespace().collect())
}

fn queries_from_mashup(b64: &str) -> Result<Vec<PowerQueryItem>, AppError> {
    let data = STANDARD.decode(b64).map_err(corrupt)?;
    // 4 bytes de versão + 4 bytes (LE) com o tamanho do pacote OPC (ZIP).
    let len = data
        .get(4..8)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
        .ok_or_else(|| corrupt("DataMashup truncado"))?;
    let package = data
        .get(8..)
        .and_then(|d| d.get(..len))
        .ok_or_else(|| corrupt("DataMashup truncado"))?;
    let mut zip = ZipArchive::new(Cursor::new(package)).map_err(corrupt)?;
    let m = String::from_utf8_lossy(&read_entry(&mut zip, "Formulas/Section1.m")?).into_owned();
    Ok(split_queries(m.trim_start_matches('\u{feff}')))
}

/// Uma consulta por `shared <nome> = ...;` no início da linha.
/// shortcut: um `shared` no começo de linha dentro de texto multilinha vira consulta falsa; raro em M gerado pelo Excel.
pub fn split_queries(m: &str) -> Vec<PowerQueryItem> {
    let mut out: Vec<PowerQueryItem> = Vec::new();
    for line in m.lines() {
        if let Some(rest) = line.strip_prefix("shared ") {
            let (name, tail) = split_name(rest.trim_start());
            out.push(PowerQueryItem {
                name,
                code: tail.to_string(),
            });
        } else if let Some(q) = out.last_mut() {
            q.code.push('\n');
            q.code.push_str(line);
        }
    }
    for q in &mut out {
        let code = q.code.trim();
        let code = code.strip_suffix(';').unwrap_or(code);
        q.code = code.trim().to_string();
    }
    out
}

/// `#"Relatório Mensal" = ...` ou `Nome = ...` → (nome sem o `#""`, texto depois do `=`).
fn split_name(s: &str) -> (String, &str) {
    let (name, rest) = if let Some(quoted) = s.strip_prefix("#\"") {
        let mut end = quoted.len();
        let mut chars = quoted.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c != '"' {
                continue;
            }
            if matches!(chars.peek(), Some((_, '"'))) {
                chars.next(); // "" é aspa escapada
            } else {
                end = i;
                break;
            }
        }
        (
            quoted[..end].replace("\"\"", "\""),
            quoted.get(end + 1..).unwrap_or(""),
        )
    } else {
        let end = s
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(s.len());
        (s[..end].to_string(), &s[end..])
    };
    let rest = rest.trim_start();
    (name, rest.strip_prefix('=').unwrap_or(rest))
}

/// `<connection name description>` + o filho `dbPr`/`webPr`/`textPr` (`connection`, `url` ou `sourceFile`).
/// shortcut: outros tipos de conexão ficam com a string vazia; ampliar se aparecer caso real.
fn parse_connections(xml: &str) -> Vec<ConnectionInfo> {
    let mut reader = Reader::from_str(xml);
    let mut out: Vec<ConnectionInfo> = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let tag = e.local_name();
                let tag = tag.as_ref();
                if tag == "connection" {
                    out.push(ConnectionInfo {
                        name: attr(&e, "name"),
                        description: attr(&e, "description"),
                        connection: String::new(),
                    });
                } else if matches!(tag, "dbPr" | "webPr" | "textPr")
                    && let Some(c) = out.last_mut()
                {
                    c.connection = ["connection", "url", "sourceFile"]
                        .iter()
                        .map(|k| attr(&e, k))
                        .find(|v| !v.is_empty())
                        .unwrap_or_default();
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out
}

fn attr(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == key)
        .and_then(|a| {
            a.normalized_value(XmlVersion::Implicit1_0)
                .ok()
                .map(|v| v.into_owned())
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in files {
            w.start_file(*name, SimpleFileOptions::default()).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    const M: &str = "section Section1;\n\nshared Vendas = let\n    Fonte = 1\nin\n    Fonte;\n\nshared #\"Relatório Mensal\" = let\n    x = \"a\"\nin\n    x;\n";

    fn workbook(utf16_item: bool, connections: Option<&str>) -> Vec<u8> {
        let package = zip_of(&[("Formulas/Section1.m", M.as_bytes())]);
        let mut blob = vec![0, 0, 0, 0];
        blob.extend((package.len() as u32).to_le_bytes());
        blob.extend(&package);
        blob.extend([0u8; 12]); // permissões/metadados depois do pacote
        let xml = format!(
            "<DataMashup xmlns=\"http://schemas.microsoft.com/DataMashup\">{}</DataMashup>",
            STANDARD.encode(blob)
        );
        let item: Vec<u8> = if utf16_item {
            let mut v = vec![0xFF, 0xFE];
            v.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
            v
        } else {
            xml.into_bytes()
        };
        let mut files: Vec<(&str, &[u8])> = vec![("customXml/item1.xml", &item)];
        if let Some(c) = connections {
            files.push(("xl/connections.xml", c.as_bytes()));
        }
        zip_of(&files)
    }

    fn run(bytes: Vec<u8>) -> PowerQuery {
        extract(ZipArchive::new(Cursor::new(bytes)).unwrap()).unwrap()
    }

    #[test]
    fn separa_consultas_com_nome_acentuado() {
        for utf16 in [true, false] {
            let pq = run(workbook(utf16, None));
            let names: Vec<_> = pq.queries.iter().map(|q| q.name.as_str()).collect();
            assert_eq!(names, ["Vendas", "Relatório Mensal"]);
            assert_eq!(pq.queries[0].code, "let\n    Fonte = 1\nin\n    Fonte");
            assert!(pq.queries[1].code.starts_with("let"));
        }
    }

    #[test]
    fn le_conexoes() {
        let xml = r#"<connections xmlns="x"><connection id="1" name="Consulta - Vendas" description="Conexão &amp; dados"><dbPr connection="Provider=Microsoft.Mashup.OleDb.1;Location=Vendas" command="SELECT 1"/></connection></connections>"#;
        let pq = run(workbook(false, Some(xml)));
        assert_eq!(pq.connections.len(), 1);
        assert_eq!(pq.connections[0].name, "Consulta - Vendas");
        assert_eq!(pq.connections[0].description, "Conexão & dados");
        assert!(pq.connections[0].connection.contains("Location=Vendas"));
    }

    #[test]
    fn sem_consultas_e_vazio() {
        let pq = run(zip_of(&[("xl/workbook.xml", b"<workbook/>")]));
        assert_eq!(pq, PowerQuery::default());
    }

    #[test]
    fn datamashup_truncado_e_erro() {
        assert!(queries_from_mashup(&STANDARD.encode([0u8, 0, 0, 0, 255, 0, 0, 0])).is_err());
    }

    #[test]
    fn aspa_escapada_no_nome() {
        let q = split_queries("shared #\"a\"\"b\" = 1;");
        assert_eq!(q[0].name, "a\"b");
        assert_eq!(q[0].code, "1");
    }
}
