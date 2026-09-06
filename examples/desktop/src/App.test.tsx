// @vitest-environment jsdom
import { fireEvent, render, screen, cleanup } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
vi.mock("./bridge", () => ({
  api: {
    native: true,
    readers: async () => ["Test reader"],
    present: async () => true,
    clear: async () => {},
    onRemoved: async () => () => {},
    read: async (requestId: number) => ({
      requestId,
      display: {
        fullNameEnglish: "Test Cardholder",
        fullNameArabic: "Arabic fixture",
        idNumber: "000-0000-0000000-0",
        genderCode: "M",
      },
      elapsedMs: 10,
      atr: "TEST",
      data: {
        readerName: "Test reader",
        cardGeneration: "v2",
        idNumber: "000000000000000",
        cardNumber: "000000000",
        photoJpeg: [255, 216, 255, 217],
        holderSignatureImage: null,
        nonModifiable: {
          fullNameEnglish: "Test,,Cardholder",
          dateOfBirth: "2008-07-12",
          fullNameArabic: "اسم تجريبي",
        },
        modifiable: {},
        readStatus: {
          identity: "read",
          nonModifiable: "read",
          photo: "read",
          modifiable: "not_requested",
          holderSignatureImage: "not_requested",
        },
      },
    }),
  },
}));
import { App } from "./App";
import { ReaderSelect } from "./ReaderSelect";
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});
it("reads automatically, removes settings, and resumes after stopping", async () => {
  URL.createObjectURL = vi.fn(() => "blob:fixture");
  URL.revokeObjectURL = vi.fn();
  localStorage.setItem(
    "emirates-id-reader.preferences.v1",
    '{"autoRead":false}',
  );
  const readStorage = vi.spyOn(Storage.prototype, "getItem");
  const writeStorage = vi.spyOn(Storage.prototype, "setItem");
  render(<App />);
  expect(
    screen
      .getByRole("button", { name: "Automatic" })
      .getAttribute("aria-pressed"),
  ).toBe("true");
  expect(screen.queryByRole("button", { name: "Settings" })).toBeNull();
  await screen.findAllByText("Test Cardholder");
  expect(screen.queryByText("Test,,Cardholder")).toBeNull();
  expect(screen.getAllByText("12 Jul 2008").length).toBe(2);
  fireEvent.click(screen.getByRole("button", { name: "Stop reading" }));
  expect(screen.queryByText("Test Cardholder")).toBeNull();
  expect(screen.getByRole("heading", { name: "Reading stopped" })).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Resume reading" }));
  await screen.findAllByText("Test Cardholder");
  fireEvent.click(screen.getByRole("button", { name: "Manual" }));
  expect(
    screen.getByRole("button", { name: "Manual" }).getAttribute("aria-pressed"),
  ).toBe("true");
  expect(readStorage).not.toHaveBeenCalled();
  expect(writeStorage).not.toHaveBeenCalled();
  expect(URL.revokeObjectURL).toHaveBeenCalled();
});
it("supports keyboard selection, Escape, and focus leaving the reader dropdown", () => {
  const select = vi.fn();
  render(
    <ReaderSelect
      readers={["First reader", "Second reader"]}
      selected="First reader"
      disabled={false}
      onSelect={select}
    />,
  );
  const trigger = screen.getByRole("combobox", { name: "Connected device" });
  fireEvent.keyDown(trigger, { key: "ArrowDown" });
  fireEvent.keyDown(trigger, { key: "End" });
  expect(trigger.getAttribute("aria-activedescendant")).toBe(
    screen.getByRole("option", { name: "Second reader" }).id,
  );
  fireEvent.keyDown(trigger, { key: "Enter" });
  expect(select).toHaveBeenCalledWith("Second reader");
  expect(screen.queryByRole("listbox")).toBeNull();
  fireEvent.click(trigger);
  fireEvent.keyDown(trigger, { key: "Escape" });
  expect(screen.queryByRole("listbox")).toBeNull();
  fireEvent.click(trigger);
  fireEvent.blur(trigger, { relatedTarget: document.body });
  expect(screen.queryByRole("listbox")).toBeNull();
});
