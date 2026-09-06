import { describe, expect, it, vi } from "vitest";
import { ReaderController, type ReaderApi, type Scan } from "./model";

export function fixture(requestId = 0): Scan {
  return {
    requestId,
    display: {
      fullNameEnglish: "Sample Cardholder",
      fullNameArabic: null,
      idNumber: "000-0000-0000000-0",
      genderCode: "M",
    },
    elapsedMs: 100,
    atr: "TEST",
    data: {
      readerName: "Test reader",
      cardGeneration: "v2",
      idNumber: "000000000000000",
      cardNumber: "000000000",
      photoJpeg: null,
      holderSignatureImage: null,
      nonModifiable: {
        fullNameEnglish: "Sample,Cardholder",
        dateOfBirth: "1990-01-01",
      },
      modifiable: { passportNumber: "TEST00000" },
      readStatus: {
        identity: "read",
        nonModifiable: "read",
        photo: "not_requested",
        modifiable: "read",
        holderSignatureImage: "not_requested",
      },
    },
  };
}
function harness() {
  let finish: (scan: Scan) => void = () => {};
  let requestId = 0;
  const api: ReaderApi = {
    native: true,
    present: async () => false,
    readers: async () => ["Test reader"],
    clear: async () => {},
    onRemoved: async () => () => {},
    read: (id) => {
      requestId = id;
      return new Promise((resolve) => {
        finish = resolve;
      });
    },
  };
  const controller = new ReaderController(api);
  return {
    controller,
    api,
    get requestId() {
      return requestId;
    },
    finish: () => finish(fixture(requestId)),
  };
}

describe("reader lifecycle", () => {
  it("does not restore a late read after Clear", async () => {
    const h = harness();
    await h.controller.refresh();
    const pending = h.controller.read();
    h.controller.clear();
    h.finish();
    await pending;
    expect(h.controller.getSnapshot().scan).toBeNull();
  });
  it("does not restore a read when removal arrives before its response", async () => {
    const h = harness();
    await h.controller.refresh();
    const pending = h.controller.read();
    h.controller.removed({ requestId: h.requestId, reason: "Removed" });
    h.finish();
    await pending;
    expect(h.controller.getSnapshot().scan).toBeNull();
    expect(h.controller.getSnapshot().notice).toBe("Removed");
  });
  it("clears the active snapshot when the current card is removed", async () => {
    const h = harness();
    await h.controller.refresh();
    const pending = h.controller.read();
    h.finish();
    await pending;
    h.controller.removed({ requestId: h.requestId, reason: "Removed" });
    expect(h.controller.getSnapshot().scan).toBeNull();
  });
});
describe("automatic reading", () => {
  it("does not read in manual mode", async () => {
    const h = harness();
    h.api.present = async () => true;
    h.api.read = vi.fn();
    await h.controller.refresh();
    await h.controller.pollPresence();
    expect(h.api.read).not.toHaveBeenCalled();
  });
  it("reads once per insertion and Clear suppresses reading until removal", async () => {
    const h = harness();
    let present = true;
    h.api.present = async () => present;
    h.api.read = vi.fn(async (id) => fixture(id));
    h.controller.setAutoRead(true);
    await h.controller.refresh();
    await h.controller.pollPresence();
    await h.controller.pollPresence();
    expect(h.api.read).toHaveBeenCalledTimes(1);
    h.controller.clear();
    await h.controller.pollPresence();
    expect(h.controller.getSnapshot().scan).toBeNull();
    expect(h.api.read).toHaveBeenCalledTimes(1);
    present = false;
    await h.controller.pollPresence();
    present = true;
    await h.controller.pollPresence();
    expect(h.api.read).toHaveBeenCalledTimes(2);
  });
  it("does not repeatedly retry a failed read", async () => {
    const h = harness();
    h.api.present = async () => true;
    h.api.read = vi.fn(async () => {
      throw new Error("Read failed");
    });
    h.controller.setAutoRead(true);
    await h.controller.refresh();
    await h.controller.pollPresence();
    await h.controller.pollPresence();
    expect(h.api.read).toHaveBeenCalledTimes(1);
    expect(h.controller.getSnapshot().error).toBe(true);
  });
  it("ignores an insertion result that arrives after Clear", async () => {
    const h = harness();
    let resolve: (value: boolean) => void = () => {};
    h.api.present = () =>
      new Promise((r) => {
        resolve = r;
      });
    h.api.read = vi.fn();
    h.controller.setAutoRead(true);
    await h.controller.refresh();
    const pending = h.controller.pollPresence();
    h.controller.clear();
    resolve(true);
    await pending;
    expect(h.api.read).not.toHaveBeenCalled();
  });
  it("ignores a pending detection after switching to manual", async () => {
    const h = harness();
    let resolve: (value: boolean) => void = () => {};
    h.api.present = () =>
      new Promise((r) => {
        resolve = r;
      });
    h.api.read = vi.fn();
    h.controller.setAutoRead(true);
    await h.controller.refresh();
    const pending = h.controller.pollPresence();
    h.controller.setAutoRead(false);
    resolve(true);
    await pending;
    expect(h.api.read).not.toHaveBeenCalled();
  });
});
it("Stop stays paused across insertions and refresh until explicitly resumed", async () => {
  const h = harness();
  let present = true;
  h.api.present = async () => present;
  h.api.read = vi.fn(async (id) => fixture(id));
  h.controller.setAutoRead(true);
  await h.controller.refresh();
  await h.controller.pollPresence();
  h.controller.stop();
  present = false;
  await h.controller.pollPresence();
  present = true;
  await h.controller.refresh();
  await h.controller.pollPresence();
  expect(h.api.read).toHaveBeenCalledTimes(1);
  expect(h.controller.getSnapshot().paused).toBe(true);
  expect(h.controller.getSnapshot().scan).toBeNull();
  await h.controller.resume();
  expect(h.api.read).toHaveBeenCalledTimes(2);
  expect(h.controller.getSnapshot().paused).toBe(false);
});
it("Stop rejects an in-flight read result", async () => {
  const h = harness();
  await h.controller.refresh();
  const pending = h.controller.read();
  h.controller.stop();
  h.finish();
  await pending;
  expect(h.controller.getSnapshot().scan).toBeNull();
  expect(h.controller.getSnapshot().paused).toBe(true);
});
