import type { Handler } from "../interceptor";

export const exportHandlers: Record<string, Handler> = {
  export_data: ({ path, format, data }) => {
    console.log(`[mock] Exported ${data} items to ${path} as ${format}`);
  },
};
