import type { Handler } from "../interceptor";

export const logHandlers: Record<string, Handler> = {
  read_logs: ({ maxLines }, state) => {
    const limit = (maxLines as number) ?? 100;
    return state.logs.slice(-limit);
  },

  clear_logs: (_args, state) => {
    state.logs = [];
  },
};
