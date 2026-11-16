use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Arquivo não encontrado: {0}")]
    FileNotFound(String),
    #[error("Arquivo bloqueado ou sem permissão de leitura (feche-o no Excel): {0}")]
    FileLocked(String),
    #[error("Formato não suportado: {0}. Use .xlsx ou .xlsm.")]
    UnsupportedFormat(String),
    #[error("Arquivo corrompido ou inválido: {0}")]
    CorruptedFile(String),
    #[error("O arquivo não contém projeto VBA.")]
    NoVbaProject,
    #[error("Falha ao gravar o arquivo: {0}")]
    WriteFailed(String),
    #[error("Nome de arquivo inválido: {0}")]
    InvalidName(String),
    #[error("Já existe na pasta: {0}")]
    FileExists(String),
    #[error("Erro interno: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound(_) => "FileNotFound",
            Self::FileLocked(_) => "FileLocked",
            Self::UnsupportedFormat(_) => "UnsupportedFormat",
            Self::CorruptedFile(_) => "CorruptedFile",
            Self::NoVbaProject => "NoVbaProject",
            Self::WriteFailed(_) => "WriteFailed",
            Self::InvalidName(_) => "InvalidName",
            Self::FileExists(_) => "FileExists",
            Self::Internal(_) => "Internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializa_code_e_message() {
        let json = serde_json::to_value(AppError::NoVbaProject).unwrap();
        assert_eq!(json["code"], "NoVbaProject");
        assert!(json["message"].as_str().unwrap().contains("VBA"));
    }
}
