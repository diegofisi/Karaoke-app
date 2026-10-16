/** Rust rechaza con un `String` en español (CONTRATO.md §1); se pinta tal cual. */
export const mensajeDeError = (error: unknown): string => {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "No se pudo completar la operación.";
};
