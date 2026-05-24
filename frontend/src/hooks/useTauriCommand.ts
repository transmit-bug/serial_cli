import { useCallback, useState } from "react";

export function useTauriCommand<TResult>(command: () => Promise<TResult>) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const invoke = useCallback(async (): Promise<TResult | null> => {
    setLoading(true);
    setError(null);
    try {
      const result = await command();
      setLoading(false);
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      setLoading(false);
      return null;
    }
  }, [command]);

  return { invoke, loading, error, setError };
}
