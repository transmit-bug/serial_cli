import type { Handler } from "../interceptor";

const HEX = "0123456789ABCDEF";

/** Modbus RTU CRC16 (poly 0xA001, little-endian result). */
function crc16Modbus(data: number[]): number[] {
  let crc = 0xffff;
  for (const byte of data) {
    crc ^= byte;
    for (let i = 0; i < 8; i++) {
      crc = crc & 1 ? (crc >> 1) ^ 0xa001 : crc >> 1;
    }
  }
  return [crc & 0xff, (crc >> 8) & 0xff];
}

/** Modbus ASCII LRC: two's complement of the byte sum. */
function lrc(data: number[]): number {
  let sum = 0;
  for (const b of data) sum = (sum + b) & 0xff;
  return (256 - sum) & 0xff;
}

/** Protocols whose frame format is a raw byte stream + trailing CRC16. */
const CRC16_PROTOCOLS = new Set(["modbus_rtu", "pzem004t", "dlt645"]);

/**
 * Real encode for the mock layer. Known protocols get their actual frame
 * formats (Modbus RTU CRC16 / Modbus ASCII LRC). Unknown protocols get a
 * deterministic length-prefix + XOR-checksum frame so INPUT→OUTPUT still
 * visibly transforms (decode reverses it).
 */
function encodeBytes(name: string, data: number[]): number[] {
  const lower = name.toLowerCase();

  if (lower === "modbus_ascii") {
    // Frame: ':' HEX(LRC-protected bytes) CR LF
    const frame = [...data, lrc(data)];
    const hex = frame.map((b) => HEX[(b >> 4) & 0xf] + HEX[b & 0xf]).join("");
    const ascii = `:${hex}\r\n`;
    return Array.from(ascii).map((c) => c.charCodeAt(0));
  }

  if (CRC16_PROTOCOLS.has(lower)) {
    return [...data, ...crc16Modbus(data)];
  }

  // Fallback mock transform: [LEN] DATA... [XOR]
  let xorsum = 0;
  for (const b of data) xorsum ^= b;
  return [data.length & 0xff, ...data, xorsum];
}

function decodeBytes(name: string, data: number[]): number[] {
  const lower = name.toLowerCase();

  if (lower === "modbus_ascii") {
    // Strip frame delimiters, parse hex chars, verify LRC
    const str = String.fromCharCode(...data)
      .replace(/^:/, "")
      .replace(/\r?\n?$/, "");
    const bytes: number[] = [];
    for (let i = 0; i < str.length; i += 2) {
      const hex = str.slice(i, i + 2);
      const byte = Number.parseInt(hex, 16);
      if (!Number.isNaN(byte)) bytes.push(byte);
    }
    if (
      bytes.length > 1 &&
      lrc(bytes.slice(0, -1)) === bytes[bytes.length - 1]
    ) {
      return bytes.slice(0, -1);
    }
    return bytes;
  }

  if (CRC16_PROTOCOLS.has(lower)) {
    if (data.length >= 2) {
      const [lo, hi] = crc16Modbus(data.slice(0, -2));
      if (lo === data[data.length - 2] && hi === data[data.length - 1]) {
        return data.slice(0, -2);
      }
    }
    return data;
  }

  // Reverse the fallback encode: [LEN] DATA... [XOR]
  if (data.length >= 2) {
    const len = data[0];
    const payload = data.slice(1, 1 + len);
    let xorsum = 0;
    for (const b of payload) xorsum ^= b;
    if (data[1 + len] === xorsum) return payload;
  }
  return data;
}

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
    const scriptName = name as string;
    state.saveScriptContent(scriptName, (content as string) ?? "");
    state.userScripts.set(scriptName, {
      name: scriptName,
      path: `/mock/scripts/${scriptName}.lua`,
      size: (content as string)?.length ?? 0,
      modified: Date.now(),
    });
  },

  delete_user_script: ({ name }, state) => {
    state.userScripts.delete(name as string);
    state.userScriptContents.delete(name as string);
  },

  /** Read back a saved user script's full content (mock restore path). */
  read_user_script: ({ name }, state) => {
    return state.getScriptContent(name as string) ?? "";
  },

  bind_script: ({ portId, scriptName }, state) => {
    state.attachScript(portId as string, scriptName as string);
  },

  script_encode: ({ script, data }) => {
    return encodeBytes((script as string) ?? "", (data as number[]) || []);
  },

  script_decode: ({ script, data }) => {
    return decodeBytes((script as string) ?? "", (data as number[]) || []);
  },

  save_script_file: ({ name, content }, state) => {
    state.saveScriptContent(name as string, (content as string) ?? "");
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
