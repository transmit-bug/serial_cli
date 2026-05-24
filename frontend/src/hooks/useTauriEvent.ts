import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

export function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    listen<T>(event, (e) => {
      handler(e.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [event, handler]);
}
