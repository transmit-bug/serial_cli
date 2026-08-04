import type { Handler } from "../interceptor";

export const serverHandlers: Record<string, Handler> = {
  start_server: (_args, state) => {
    state.server.running = true;
    state.server.socket_path = "/tmp/serial-cli-mock.sock";
    state.server.started_at = Date.now();
    return { ...state.server };
  },

  stop_server: (_args, state) => {
    state.server.running = false;
    state.server.socket_path = "";
    state.server.started_at = 0;
    state.server.active_connections = 0;
  },

  get_server_status: (_args, state) => {
    return { ...state.server };
  },
};
