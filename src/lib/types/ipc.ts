// Tipos para comunicación con el backend Rust

export interface IpcResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Comandos de biblioteca
export interface CreateSongPayload {
  title: string;
  artist?: string;
  audioPath: string;
}

export interface UpdateSongPayload {
  id: string;
  title?: string;
  artist?: string;
  audioPath?: string;
}
