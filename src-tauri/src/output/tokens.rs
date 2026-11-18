//! Estimativa aproximada de tokens e divisão em partes. Não importa nada do Tauri.

/// Estimativa grosseira: ≈ 1 token a cada 4 caracteres (arredondado para cima).
/// Não é a contagem de nenhum modelo; serve só para dimensionar o texto.
pub fn estimate(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// Agrupa `units` (blocos indivisíveis: aba, tabela, componente VBA) em partes de até
/// `max_tokens`. Nunca corta uma unidade: uma unidade maior que o limite fica sozinha na sua
/// parte. A junção das partes é idêntica à junção das unidades. `max_tokens == 0` vale 1.
pub fn split_units(units: &[String], max_tokens: usize) -> Vec<String> {
    let max = max_tokens.max(1);
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_tokens = 0;
    for unit in units {
        let t = estimate(unit);
        if !current.is_empty() && current_tokens + t > max {
            parts.push(std::mem::take(&mut current));
            current_tokens = 0;
        }
        current.push_str(unit);
        current_tokens += t;
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(sizes: &[usize]) -> Vec<String> {
        sizes
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{i}{}", "x".repeat(n * 4 - 1)))
            .collect()
    }

    #[test]
    fn estimativa_arredonda_para_cima_e_conta_caracteres() {
        assert_eq!(estimate(""), 0);
        assert_eq!(estimate("abcd"), 1);
        assert_eq!(estimate("abcde"), 2);
        assert_eq!(estimate("çãõé"), 1); // caracteres, não bytes
    }

    #[test]
    fn junta_ate_o_limite_sem_cortar_unidade() {
        let u = units(&[3, 3, 3, 3]); // 3 tokens cada
        let parts = split_units(&u, 6);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], format!("{}{}", u[0], u[1]));
        assert_eq!(parts[1], format!("{}{}", u[2], u[3]));
    }

    #[test]
    fn unidade_maior_que_o_limite_fica_sozinha() {
        let u = units(&[2, 10, 2]);
        let parts = split_units(&u, 5);
        assert_eq!(parts, vec![u[0].clone(), u[1].clone(), u[2].clone()]);
    }

    #[test]
    fn juncao_das_partes_reproduz_o_todo() {
        let u = units(&[1, 7, 2, 2, 9, 1, 3]);
        for max in [0, 1, 4, 8, 100] {
            assert_eq!(split_units(&u, max).concat(), u.concat(), "max={max}");
        }
    }

    #[test]
    fn sem_unidades_nao_ha_partes() {
        assert!(split_units(&[], 10).is_empty());
    }
}
