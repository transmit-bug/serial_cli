import type { SearchOptions } from "@/stores/data";

/** Split text into segments, marking which ones match the query. */
export function splitHighlights(
  text: string,
  query: string,
  options: SearchOptions,
): { text: string; match: boolean }[] {
  if (!query) return [{ text, match: false }];
  try {
    const flags = options.caseSensitive ? "g" : "gi";
    const re = options.useRegex ? new RegExp(query, flags) : undefined;

    if (re) {
      const segments: { text: string; match: boolean }[] = [];
      let last = 0;
      for (const m of text.matchAll(re)) {
        if (m.index > last)
          segments.push({ text: text.slice(last, m.index), match: false });
        segments.push({ text: m[0], match: true });
        last = m.index + m[0].length;
      }
      if (last < text.length)
        segments.push({ text: text.slice(last), match: false });
      return segments.length ? segments : [{ text, match: false }];
    }

    // Plain text search
    const lower = options.caseSensitive ? text : text.toLowerCase();
    const q = options.caseSensitive ? query : query.toLowerCase();
    const segments: { text: string; match: boolean }[] = [];
    let last = 0;
    let idx = lower.indexOf(q, 0);
    while (idx !== -1) {
      if (idx > last)
        segments.push({ text: text.slice(last, idx), match: false });
      segments.push({ text: text.slice(idx, idx + q.length), match: true });
      last = idx + q.length;
      idx = lower.indexOf(q, last);
    }
    if (last < text.length)
      segments.push({ text: text.slice(last), match: false });
    return segments.length ? segments : [{ text, match: false }];
  } catch {
    return [{ text, match: false }];
  }
}
