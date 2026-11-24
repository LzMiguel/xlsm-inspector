import type { AppError, AppErrorCode } from "../types/models";
import { Alert } from "./ui/Alert";

const TITLES: Record<AppErrorCode, string> = {
  FileNotFound: "Arquivo não encontrado",
  FileLocked: "Arquivo bloqueado",
  UnsupportedFormat: "Formato não suportado",
  CorruptedFile: "Arquivo corrompido, inválido ou protegido por senha",
  NoVbaProject: "Arquivo sem VBA",
  WriteFailed: "Falha ao gravar os arquivos",
  InvalidName: "Nome de arquivo inválido",
  FileExists: "Arquivo já existe",
  Internal: "Erro inesperado",
};

export function ErrorAlert({ error }: { error: AppError }) {
  return (
    <Alert variant="danger" title={TITLES[error.code]}>
      {error.message}
    </Alert>
  );
}
