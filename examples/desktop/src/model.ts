export type GroupStatus =
  "read" | "not_requested" | "not_available" | "protected";
export type ReadOptions = {
  photo: boolean;
  modifiableData: boolean;
  holderSignatureImage: boolean;
};
export type Fields = Record<string, string | null>;
export type CardData = {
  readerName: string;
  cardGeneration: "v1" | "v2" | "unknown";
  idNumber: string;
  cardNumber: string;
  photoJpeg: number[] | null;
  holderSignatureImage: number[] | null;
  nonModifiable: Fields;
  modifiable: Fields;
  readStatus: {
    identity: GroupStatus;
    nonModifiable: GroupStatus;
    photo: GroupStatus;
    modifiable: GroupStatus;
    holderSignatureImage: GroupStatus;
  };
};
export type Scan = {
  display: {
    fullNameEnglish: string | null;
    fullNameArabic: string | null;
    idNumber: string;
    genderCode: string | null;
  };
  signaturePreviewPng?: number[] | null;
  requestId: number;
  data: CardData;
  elapsedMs: number;
  atr: string;
};
export type Removed = { requestId: number; reason: string };
export interface ReaderApi {
  native: boolean;
  present(reader: string): Promise<boolean>;
  readers(requestId: number): Promise<string[]>;
  read(requestId: number, reader: string, options: ReadOptions): Promise<Scan>;
  clear(requestId: number): Promise<void>;
  onRemoved(callback: (event: Removed) => void): Promise<() => void>;
}
export type ViewState = {
  readers: string[];
  selected: string;
  options: ReadOptions;
  scan: Scan | null;
  busy: boolean;
  paused: boolean;
  notice: string;
  error: boolean;
};

export class ReaderController {
  private requestId = Date.now() * 1000;
  private listeners = new Set<() => void>();
  private state: ViewState;
  private autoRead = false;
  private consumed = false;
  private polling = false;
  private epoch = 0;
  constructor(readonly api: ReaderApi) {
    this.state = {
      readers: [],
      selected: "",
      options: {
        photo: true,
        modifiableData: true,
        holderSignatureImage: true,
      },
      scan: null,
      busy: false,
      paused: false,
      error: false,
      notice: api.native
        ? "Choose a reader to get started."
        : "Open the desktop app to connect to a reader.",
    };
  }
  getSnapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
  private update(patch: Partial<ViewState>) {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach((fn) => fn());
  }
  private failure(id: number, error: unknown) {
    if (id === this.requestId)
      this.update({
        busy: false,
        error: true,
        notice: String(error),
        scan: null,
      });
  }
  setOptions(options: ReadOptions) {
    this.update({ options });
  }
  select(reader: string) {
    this.clear();
    this.consumed = false;
    this.update({ selected: reader });
  }
  setAutoRead(enabled: boolean) {
    this.autoRead = enabled;
    ++this.epoch;
  }
  // Poll only presence. A read is attempted once per insertion, including failures.
  async pollPresence() {
    if (
      !this.api.native ||
      this.polling ||
      this.state.paused ||
      this.state.busy ||
      !this.state.selected
    )
      return;
    this.polling = true;
    const reader = this.state.selected;
    const epoch = this.epoch;
    try {
      const present = await this.api.present(reader);
      if (
        epoch !== this.epoch ||
        reader !== this.state.selected ||
        this.state.busy
      )
        return;
      if (!present) {
        this.consumed = false;
        if (this.state.scan)
          this.removed({
            requestId: this.requestId,
            reason: "Card removed. Ready for the next card.",
          });
      } else if (this.autoRead && !this.consumed) {
        this.consumed = true;
        await this.read();
      }
    } catch (error) {
      if (epoch === this.epoch && reader === this.state.selected) {
        this.consumed = true;
        const id = ++this.requestId;
        void this.api.clear(id).catch(() => {});
        this.failure(id, error);
      }
    } finally {
      this.polling = false;
    }
  }
  async refresh() {
    if (!this.api.native) return;
    ++this.epoch;
    const id = ++this.requestId;
    this.update({
      scan: null,
      busy: true,
      error: false,
      notice: "Finding connected readers…",
    });
    try {
      const readers = await this.api.readers(id);
      if (id !== this.requestId) return;
      if (!readers.includes(this.state.selected)) this.consumed = false;
      this.update({
        readers,
        selected: readers.includes(this.state.selected)
          ? this.state.selected
          : (readers[0] ?? ""),
        busy: false,
        notice: readers.length
          ? this.autoRead
            ? "Reader available. Insert a card to read automatically."
            : "Reader available. Insert a card and select Read card."
          : "No reader found. Connect a reader, then refresh.",
      });
    } catch (error) {
      if (id === this.requestId) this.update({ readers: [], selected: "" });
      this.failure(id, error);
    }
  }
  async read() {
    if (!this.api.native || this.state.busy || !this.state.selected) return;
    ++this.epoch;
    this.consumed = true;
    const id = ++this.requestId;
    this.update({
      scan: null,
      busy: true,
      error: false,
      paused: false,
      notice: "Reading card. Keep it in the reader.",
    });
    try {
      const scan = await this.api.read(id, this.state.selected, {
        ...this.state.options,
      });
      if (id !== this.requestId || scan.requestId !== id) return;
      this.update({
        scan,
        busy: false,
        notice: "Read complete. Data clears when the card is removed.",
      });
    } catch (error) {
      this.failure(id, error);
    }
  }
  stop() {
    this.clear();
    this.update({ paused: true, notice: "Reading stopped." });
  }
  async resume() {
    this.consumed = false;
    ++this.epoch;
    this.update({ paused: false, error: false, notice: "Ready for a card." });
    if (this.autoRead) await this.pollPresence();
    else await this.read();
  }
  clear() {
    ++this.epoch;
    this.consumed = true;
    const id = ++this.requestId;
    this.update({
      scan: null,
      busy: false,
      error: false,
      notice: "Session cleared. Ready for another card.",
    });
    void this.api.clear(id).catch((error) => this.failure(id, error));
  }
  removed = (event: Removed) => {
    if (event.requestId !== this.requestId) return;
    ++this.epoch;
    this.consumed = false;
    ++this.requestId; // Also reject a read response arriving after its removal event.
    this.update({
      scan: null,
      busy: false,
      error: false,
      notice: event.reason,
    });
  };
}

export function label(key: string) {
  return key
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (c) => c.toUpperCase())
    .replace(/\bId\b/g, "ID");
}
