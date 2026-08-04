import type { Handler } from "../interceptor";

export const serialScriptHandlers: Record<string, Handler> = {
  attach_script: ({ portId, scriptSource }, state) => {
    state.attachScript(portId as string, scriptSource as string);
  },

  detach_script: ({ portId }, state) => {
    state.detachScript(portId as string);
  },

  get_script_status: ({ portId }, state) => {
    return state.getScriptStatus(portId as string);
  },

  list_script_actions: () => {
    return [];
  },

  call_script_function: ({ functionName }) => {
    return `[mock] ${functionName} executed`;
  },

  list_standalone_script_actions: () => {
    return [];
  },

  call_standalone_script_function: ({ functionName }) => {
    return `[mock] ${functionName} executed`;
  },
};
