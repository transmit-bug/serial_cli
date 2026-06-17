import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / 1024 ** i;
  return `${val < 10 ? val.toFixed(1) : Math.round(val)}${units[i]}`;
}

export function formatTimestamp(ms: number): string {
  const d = new Date(ms);
  return d.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    fractionalSecondDigits: 3,
  });
}

export function bytesToHex(data: number[]): string {
  if (!Array.isArray(data)) return "";
  return data
    .map((b) => b.toString(16).padStart(2, " ").toUpperCase())
    .join("");
}

export function bytesToAscii(data: number[]): string {
  if (!Array.isArray(data)) return "";
  return data
    .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : "."))
    .join("");
}

export function hexToBytes(hex: string): number[] {
  const cleaned = hex.replace(/\s+/g, "");
  if (cleaned.length % 2 !== 0) throw new Error("Invalid hex string");
  const bytes: number[] = [];
  for (let i = 0; i < cleaned.length; i += 2) {
    const byte = Number.parseInt(cleaned.slice(i, i + 2), 16);
    if (Number.isNaN(byte)) throw new Error(`Invalid hex at position ${i}`);
    bytes.push(byte);
  }
  return bytes;
}

export function formatDuration(ms: number): string {
  const secs = Math.floor(ms / 1000);
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}
