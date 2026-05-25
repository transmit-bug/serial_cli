import { describe, it, expect, beforeEach } from "vitest";
import { useCommandStore } from "@/stores/commands";

describe("useCommandStore", () => {
  beforeEach(() => {
    localStorage.clear();
    useCommandStore.setState({
      commands: [{ label: "AT", data: "AT\r\n", format: "ascii", hotkey: "F1" }],
    });
  });

  it("adds a command and persists", () => {
    useCommandStore.getState().addCommand({
      label: "TEST",
      data: "TEST\r\n",
      format: "ascii",
    });

    const cmds = useCommandStore.getState().commands;
    expect(cmds).toHaveLength(2);
    expect(cmds[1].label).toBe("TEST");

    const stored = JSON.parse(localStorage.getItem("serial-cli-quick-commands") ?? "[]");
    expect(stored).toHaveLength(2);
  });

  it("updates a command at index", () => {
    useCommandStore.getState().updateCommand(0, {
      label: "AT+RST",
      data: "AT+RST\r\n",
      format: "ascii",
      hotkey: "F1",
    });

    expect(useCommandStore.getState().commands[0].label).toBe("AT+RST");
  });

  it("deletes a command at index", () => {
    useCommandStore.getState().deleteCommand(0);
    expect(useCommandStore.getState().commands).toHaveLength(0);
  });

  it("reorders commands", () => {
    useCommandStore.setState({
      commands: [
        { label: "A", data: "A", format: "ascii" },
        { label: "B", data: "B", format: "ascii" },
        { label: "C", data: "C", format: "ascii" },
      ],
    });

    useCommandStore.getState().reorderCommand(0, 2);

    const cmds = useCommandStore.getState().commands;
    expect(cmds.map((c) => c.label)).toEqual(["B", "C", "A"]);
  });
});
