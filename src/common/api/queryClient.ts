import { QueryClient } from "@tanstack/react-query";

// Lo que cambia en Rust llega por un canal o tras una mutación: no hay sondeo.
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});
