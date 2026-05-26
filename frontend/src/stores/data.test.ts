import { beforeEach, describe, expect, it } from "vitest";
import { matchesSearch, useDataStore } from "@/stores/data";

function resetStore() {
  useDataStore.setState({
    packets: [],
    displayFormat: "mixed",
    autoScroll: true,
    maxPackets: 100,
    searchQuery: "",
    searchOptions: { caseSensitive: false, useRegex: false },
    nextId: 1,
    exportOptions: {
      format: "txt",
      fields: {
        id: true,
        timestamp: true,
        direction: true,
        hex: true,
        ascii: false,
        decoded: false,
      },
      filteredOnly: true,
    },
    exportProgress: null,
  });
}

describe("matchesSearch", () => {
  const CI = { caseSensitive: false, useRegex: false };
  const CS = { caseSensitive: true, useRegex: false };
  const RX = { caseSensitive: false, useRegex: true };

  it("returns true when query is empty", () => {
    expect(matchesSearch("anything", "", CI)).toBe(true);
  });

  it("matches plain text case-insensitively", () => {
    expect(matchesSearch("Hello World", "hello", CI)).toBe(true);
    expect(matchesSearch("Hello World", "HELLO", CI)).toBe(true);
  });

  it("respects caseSensitive flag", () => {
    expect(matchesSearch("Hello", "hello", CS)).toBe(false);
    expect(matchesSearch("Hello", "Hello", CS)).toBe(true);
  });

  it("matches regex when useRegex is true", () => {
    expect(matchesSearch("abc123", "\\d+", RX)).toBe(true);
    expect(matchesSearch("hello", "^h.*o$", RX)).toBe(true);
  });

  it("falls back to plain search on invalid regex", () => {
    expect(
      matchesSearch("hello [world", "[world", {
        caseSensitive: false,
        useRegex: true,
      }),
    ).toBe(true);
  });
});

describe("useDataStore", () => {
  beforeEach(resetStore);

  // --- addPacket ---

  describe("addPacket", () => {
    it("adds a packet and increments nextId", () => {
      useDataStore
        .getState()
        .addPacket("port-1", "rx", [0x41, 0x42], Date.now());

      expect(useDataStore.getState().packets).toHaveLength(1);
      expect(useDataStore.getState().packets[0].data).toEqual([0x41, 0x42]);
      expect(useDataStore.getState().nextId).toBe(2);
    });

    it("respects maxPackets cap with FIFO eviction", () => {
      useDataStore.setState({ maxPackets: 3 });

      for (let i = 0; i < 5; i++) {
        useDataStore.getState().addPacket("port-1", "rx", [i], Date.now());
      }

      const packets = useDataStore.getState().packets;
      expect(packets).toHaveLength(3);
      expect(packets[0].data).toEqual([2]);
      expect(packets[2].data).toEqual([4]);
    });

    it("stores decoded string when provided", () => {
      useDataStore
        .getState()
        .addPacket("port-1", "rx", [0x01, 0x02], Date.now(), "OK");

      expect(useDataStore.getState().packets[0].decoded).toBe("OK");
    });
  });

  // --- clearBuffer ---

  describe("clearBuffer", () => {
    it("clears all packets when no portId", () => {
      useDataStore.getState().addPacket("port-1", "rx", [1], Date.now());
      useDataStore.getState().addPacket("port-2", "rx", [2], Date.now());

      useDataStore.getState().clearBuffer();

      expect(useDataStore.getState().packets).toHaveLength(0);
      expect(useDataStore.getState().nextId).toBe(1);
    });

    it("clears only packets for specified portId", () => {
      useDataStore.setState({ nextId: 10 });
      useDataStore.getState().addPacket("port-1", "rx", [1], Date.now());
      useDataStore.getState().addPacket("port-2", "rx", [2], Date.now());

      useDataStore.getState().clearBuffer("port-1");

      expect(useDataStore.getState().packets).toHaveLength(1);
      expect(useDataStore.getState().packets[0].portId).toBe("port-2");
      expect(useDataStore.getState().nextId).toBe(12);
    });
  });

  // --- simple setters ---

  describe("setDisplayFormat", () => {
    it("sets display format", () => {
      useDataStore.getState().setDisplayFormat("hex");
      expect(useDataStore.getState().displayFormat).toBe("hex");
    });
  });

  describe("toggleAutoScroll", () => {
    it("toggles autoScroll", () => {
      expect(useDataStore.getState().autoScroll).toBe(true);
      useDataStore.getState().toggleAutoScroll();
      expect(useDataStore.getState().autoScroll).toBe(false);
    });
  });

  describe("setSearchQuery", () => {
    it("sets search query", () => {
      useDataStore.getState().setSearchQuery("test");
      expect(useDataStore.getState().searchQuery).toBe("test");
    });
  });

  describe("setSearchOptions", () => {
    it("merges partial search options", () => {
      useDataStore.getState().setSearchOptions({ caseSensitive: true });
      const opts = useDataStore.getState().searchOptions;
      expect(opts.caseSensitive).toBe(true);
      expect(opts.useRegex).toBe(false);
    });
  });

  // --- getFilteredPackets ---

  describe("getFilteredPackets", () => {
    beforeEach(() => {
      // Packet with ASCII "OK" (0x4F 0x4B)
      useDataStore.setState({ nextId: 1 });
      useDataStore
        .getState()
        .addPacket("port-1", "rx", [0x4f, 0x4b], Date.now());
      // Packet with ASCII "ERR" (0x45 0x52 0x52)
      useDataStore
        .getState()
        .addPacket("port-1", "rx", [0x45, 0x52, 0x52], Date.now());
      // Packet on different port
      useDataStore
        .getState()
        .addPacket("port-2", "rx", [0x4f, 0x4b], Date.now());
    });

    it("returns all packets when no query", () => {
      const result = useDataStore.getState().getFilteredPackets();
      expect(result).toHaveLength(3);
    });

    it("filters by portId", () => {
      const result = useDataStore.getState().getFilteredPackets("port-1");
      expect(result).toHaveLength(2);
    });

    it("filters by search query matching hex", () => {
      useDataStore.getState().setSearchQuery("4f 4b");
      const result = useDataStore.getState().getFilteredPackets();
      expect(result).toHaveLength(2); // both OK packets
    });

    it("filters by search query matching ascii", () => {
      useDataStore.getState().setSearchQuery("ERR");
      const result = useDataStore.getState().getFilteredPackets();
      expect(result).toHaveLength(1);
      expect(result[0].data).toEqual([0x45, 0x52, 0x52]);
    });

    it("combines portId and search query", () => {
      useDataStore.getState().setSearchQuery("4f 4b");
      const result = useDataStore.getState().getFilteredPackets("port-1");
      expect(result).toHaveLength(1);
    });

    it("returns empty when nothing matches", () => {
      useDataStore.getState().setSearchQuery("ZZZZ");
      const result = useDataStore.getState().getFilteredPackets();
      expect(result).toHaveLength(0);
    });
  });

  // --- export options ---

  describe("setExportOptions", () => {
    it("merges partial export options", () => {
      useDataStore.getState().setExportOptions({ format: "csv" });
      expect(useDataStore.getState().exportOptions.format).toBe("csv");
    });

    it("merges partial fields", () => {
      useDataStore.getState().setExportOptions({ fields: { ascii: true } });
      expect(useDataStore.getState().exportOptions.fields.ascii).toBe(true);
      expect(useDataStore.getState().exportOptions.fields.hex).toBe(true);
    });
  });

  describe("setExportProgress", () => {
    it("sets export progress", () => {
      useDataStore.getState().setExportProgress(50);
      expect(useDataStore.getState().exportProgress).toBe(50);
    });
  });
});
