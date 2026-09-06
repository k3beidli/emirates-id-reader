import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ReaderApi, Scan, Removed } from "./model";

export const api: ReaderApi = {
  native: isTauri(),
  present: (reader) => invoke<boolean>("card_present", { reader }),
  readers: (requestId) => invoke<string[]>("refresh_readers", { requestId }),
  read: (requestId, reader, options) =>
    invoke<Scan>("read_card", { requestId, reader, options }),
  clear: (requestId) =>
    isTauri()
      ? invoke<void>("clear_session", { requestId })
      : Promise.resolve(),
  onRemoved: (callback) =>
    isTauri()
      ? listen<Removed>("card-removed", (event) => callback(event.payload))
      : Promise.resolve(() => {}),
};
