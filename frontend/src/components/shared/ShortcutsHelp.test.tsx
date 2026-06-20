import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ShortcutsHelp } from "@/components/shared/ShortcutsHelp";

describe("ShortcutsHelp", () => {
  it("renders shortcut keys and descriptions", () => {
    render(<ShortcutsHelp onClose={() => {}} />);

    expect(screen.getByText("shortcuts.title")).toBeDefined();
    expect(screen.getByText("shortcuts.navTerminal")).toBeDefined();
    expect(screen.getByText("shortcuts.clearBuffer")).toBeDefined();
    expect(screen.getByText("shortcuts.showHelp")).toBeDefined();
    expect(screen.getByText("F1 – F12")).toBeDefined();
  });

  it("calls onClose when clicking overlay background", async () => {
    const onClose = vi.fn();
    const { container } = render(<ShortcutsHelp onClose={onClose} />);

    // The overlay is the outermost fixed div
    const overlay = container.firstElementChild as HTMLElement;
    await userEvent.click(overlay);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onClose when clicking X button", async () => {
    const onClose = vi.fn();
    render(<ShortcutsHelp onClose={onClose} />);

    const closeButton = screen.getByRole("button");
    await userEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledOnce();
  });
});
