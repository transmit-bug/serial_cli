import { describe, expect, it } from "vitest";
import { splitHighlights } from "@/lib/highlight";

const CI = { caseSensitive: false, useRegex: false };
const CS = { caseSensitive: true, useRegex: false };
const RX = { caseSensitive: false, useRegex: true };
const RXCS = { caseSensitive: true, useRegex: true };

describe("splitHighlights", () => {
  // --- empty query ---

  it("returns single non-match segment when query is empty", () => {
    expect(splitHighlights("hello world", "", CI)).toEqual([
      { text: "hello world", match: false },
    ]);
  });

  // --- plain text matching ---

  it("highlights a single occurrence", () => {
    const result = splitHighlights("hello world", "world", CI);
    expect(result).toEqual([
      { text: "hello ", match: false },
      { text: "world", match: true },
    ]);
  });

  it("highlights multiple occurrences", () => {
    const result = splitHighlights("ab ab ab", "ab", CI);
    expect(result).toEqual([
      { text: "ab", match: true },
      { text: " ", match: false },
      { text: "ab", match: true },
      { text: " ", match: false },
      { text: "ab", match: true },
    ]);
  });

  it("returns non-match segment when query not found", () => {
    const result = splitHighlights("hello", "xyz", CI);
    expect(result).toEqual([{ text: "hello", match: false }]);
  });

  it("is case-insensitive by default", () => {
    const result = splitHighlights("Hello HELLO", "hello", CI);
    const matches = result.filter((s) => s.match);
    expect(matches).toHaveLength(2);
    expect(matches[0].text).toBe("Hello");
    expect(matches[1].text).toBe("HELLO");
  });

  it("respects caseSensitive flag", () => {
    const result = splitHighlights("Hello HELLO", "hello", CS);
    const matches = result.filter((s) => s.match);
    expect(matches).toHaveLength(0);
  });

  it("matches at start of text", () => {
    const result = splitHighlights("abc def", "abc", CI);
    expect(result).toEqual([
      { text: "abc", match: true },
      { text: " def", match: false },
    ]);
  });

  it("matches at end of text", () => {
    const result = splitHighlights("abc def", "def", CI);
    expect(result).toEqual([
      { text: "abc ", match: false },
      { text: "def", match: true },
    ]);
  });

  it("matches entire text", () => {
    const result = splitHighlights("hello", "hello", CI);
    expect(result).toEqual([{ text: "hello", match: true }]);
  });

  // --- regex matching ---

  it("highlights regex matches", () => {
    const result = splitHighlights("abc123def", "\\d+", RX);
    expect(result).toEqual([
      { text: "abc", match: false },
      { text: "123", match: true },
      { text: "def", match: false },
    ]);
  });

  it("handles regex case-insensitive by default", () => {
    const result = splitHighlights("Hello World", "hello", RX);
    expect(result).toEqual([
      { text: "Hello", match: true },
      { text: " World", match: false },
    ]);
  });

  it("respects caseSensitive with regex", () => {
    const result = splitHighlights("Hello hello", "Hello", RXCS);
    const matches = result.filter((s) => s.match);
    expect(matches).toHaveLength(1);
    expect(matches[0].text).toBe("Hello");
  });

  it("returns non-match segment when regex has no match", () => {
    const result = splitHighlights("hello", "^world$", RX);
    expect(result).toEqual([{ text: "hello", match: false }]);
  });

  // --- edge cases ---

  it("handles special regex characters as plain text", () => {
    const result = splitHighlights("price: $10.00", "$10.00", CI);
    expect(result).toEqual([
      { text: "price: ", match: false },
      { text: "$10.00", match: true },
    ]);
  });

  it("handles invalid regex gracefully", () => {
    const result = splitHighlights("hello [world", "[world", RX);
    // Invalid regex falls back to non-match
    expect(result).toEqual([{ text: "hello [world", match: false }]);
  });

  it("preserves original text casing in segments", () => {
    const result = splitHighlights("ABCdefGHI", "def", CI);
    expect(result).toEqual([
      { text: "ABC", match: false },
      { text: "def", match: true },
      { text: "GHI", match: false },
    ]);
  });
});
