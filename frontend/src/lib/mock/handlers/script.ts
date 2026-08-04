import type { Handler } from "../interceptor";

export const scriptHandlers: Record<string, Handler> = {
  list_scripts: (_args, state) => state.scripts,

  load_script: ({ path }, state) => {
    const name =
      (path as string).split("/").pop()?.replace(".lua", "") ?? "unknown";
    const script = {
      name,
      description: `User script: ${name}`,
      built_in: false,
    };
    if (!state.scripts.find((s) => s.name === name)) {
      state.scripts.push(script);
    }
    return script;
  },

  unload_script: ({ name }, state) => {
    state.scripts = state.scripts.filter((s) => s.name !== name);
  },

  reload_script: (_args, _state) => {
    // No-op — script already in list
  },

  get_script_info: ({ name }, state) => {
    return state.scripts.find((s) => s.name === name) ?? null;
  },

  execute_script: (_args) => {
    return `[mock] Script executed successfully`;
  },

  validate_script: (_args) => {
    // Return empty errors — script is "valid"
    return [];
  },

  validate_script_detailed: (_args) => {
    return { warnings: [] };
  },

  validate_script_file: () => {
    // No-op
  },

  list_user_scripts: (_args, state) => {
    return Array.from(state.userScripts.values());
  },

  save_user_script: ({ name, content }, state) => {
    state.userScripts.set(name as string, {
      name: name as string,
      path: `/mock/scripts/${name}.lua`,
      size: (content as string)?.length ?? 0,
      modified: Date.now(),
    });
  },

  delete_user_script: ({ name }, state) => {
    state.userScripts.delete(name as string);
  },

  bind_script: ({ portId, scriptName }, state) => {
    state.attachScript(portId as string, scriptName as string);
  },

  script_encode: ({ data }) => {
    // Pass-through — no actual encoding
    return data;
  },

  script_decode: ({ data }) => {
    // Pass-through — no actual decoding
    return data;
  },

  save_script_file: ({ name, content }, state) => {
    state.userScripts.set(name as string, {
      name: name as string,
      path: `/mock/scripts/${name}.lua`,
      size: (content as string)?.length ?? 0,
      modified: Date.now(),
    });
    return `/mock/scripts/${name}.lua`;
  },

  get_hot_reload_status: (_args, state) => state.hotReloadEnabled,

  set_hot_reload_enabled: ({ enabled }, state) => {
    state.hotReloadEnabled = enabled as boolean;
  },
};
