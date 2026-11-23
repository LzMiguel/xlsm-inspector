use serde::{Deserialize, Serialize};

/// Textos do Markdown. O padrão reproduz byte a byte o formato da v1.0 (PRD §8).
/// `title`, `intro` e `footer` valem para `{arquivo}_celulas.md`; o VBA usa `vba_heading`.
/// Placeholders: `{arquivo}` (com extensão), `{data}`, `{aba}`, `{componente}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputTemplate {
    pub title: String,
    pub intro: String,
    pub sheet_heading: String,
    pub cell_header: String,
    pub content_header: String,
    pub formula_label: String,
    pub value_label: String,
    pub vba_heading: String,
    pub footer: String,
}

impl Default for OutputTemplate {
    fn default() -> Self {
        Self {
            title: "# Mapeamento de Estrutura: {arquivo}".into(),
            intro: String::new(),
            sheet_heading: "## Aba: {aba}".into(),
            cell_header: "Célula".into(),
            content_header: "Conteúdo Extraído".into(),
            formula_label: "Fórmula".into(),
            value_label: "Valor".into(),
            vba_heading: "## COMPONENTE: {componente}".into(),
            footer: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplatePreset {
    pub name: String,
    pub template: OutputTemplate,
}

/// Predefinições que acompanham o app (não editáveis).
pub fn builtin_presets() -> Vec<TemplatePreset> {
    vec![
        TemplatePreset { name: "Padrão".into(), template: OutputTemplate::default() },
        TemplatePreset {
            name: "Contexto para IA".into(),
            template: OutputTemplate {
                title: "# Planilha: {arquivo}".into(),
                intro: "Este documento descreve a estrutura da planilha `{arquivo}` (extraída em {data}). \
                        Cada aba lista as células com conteúdo; nas fórmulas aparecem a fórmula e o valor calculado. \
                        Use-o como contexto para responder perguntas e sugerir melhorias sobre a planilha."
                    .into(),
                content_header: "Conteúdo".into(),
                vba_heading: "## Módulo VBA: {componente}".into(),
                ..OutputTemplate::default()
            },
        },
    ]
}

/// Troca `{chave}` pelo valor em uma só passada (um valor que contenha `{outra}` não é expandido).
/// Chaves desconhecidas ficam como estão.
pub fn fill(text: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let hit = vars
            .iter()
            .find(|(k, _)| rest[1..].starts_with(k) && rest[1 + k.len()..].starts_with('}'));
        match hit {
            Some((k, v)) => {
                out.push_str(v);
                rest = &rest[k.len() + 2..];
            }
            None => {
                out.push('{');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Data local, `AAAA-MM-DD`.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_troca_conhecidas_e_mantem_desconhecidas() {
        let v = [("arquivo", "a.xlsx"), ("data", "2026-01-02")];
        assert_eq!(fill("{arquivo}_{data}", &v), "a.xlsx_2026-01-02");
        assert_eq!(fill("{x} { {arquivo", &v), "{x} { {arquivo");
        assert_eq!(fill("sem nada", &v), "sem nada");
        assert_eq!(fill("ç{arquivo}ã", &v), "ça.xlsxã");
    }

    #[test]
    fn fill_nao_reexpande_valores() {
        assert_eq!(fill("{a}{b}", &[("a", "{b}"), ("b", "X")]), "{b}X");
    }

    #[test]
    fn presets_embutidos() {
        let p = builtin_presets();
        assert_eq!(p[0].template, OutputTemplate::default());
        assert!(p[1].template.intro.contains("{arquivo}"));
    }

    #[test]
    fn today_tem_formato_de_data() {
        let d = today();
        assert_eq!(d.len(), 10);
        assert_eq!(d.matches('-').count(), 2);
    }
}
