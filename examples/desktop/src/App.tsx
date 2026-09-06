import { useEffect, useState, useSyncExternalStore } from "react";
import { displayData } from "./presentation";
import { Icon } from "./Icon";
import { ReaderSelect } from "./ReaderSelect";
import { api } from "./bridge";
import { detailGroups, groupOrder, pairFields } from "./details";
import { ReaderController, label, type Fields, type Scan } from "./model";

import "./style.css";
import "./card.css";

const controller = new ReaderController(api);
controller.setAutoRead(true);

function useImage(bytes: number[] | null, enabled: boolean, mime: string) {
  const [src, setSrc] = useState<string>();
  useEffect(() => {
    if (!bytes?.length || !enabled || !mime) {
      setSrc(undefined);
      return;
    }
    const url = URL.createObjectURL(
      new Blob([Uint8Array.from(bytes)], { type: mime }),
    );
    setSrc(url);
    return () => {
      URL.revokeObjectURL(url);
    };
  }, [bytes, enabled, mime]);
  return enabled ? src : undefined;
}

function Photo({ bytes, reveal }: { bytes: number[] | null; reveal: boolean }) {
  const src = useImage(
    bytes,
    reveal,
    bytes?.[0] === 137 && bytes?.[1] === 80 ? "image/png" : "image/jpeg",
  );
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [src]);
  return (
    <div className="portrait">
      {src && !failed ? (
        <img
          src={src}
          alt="Cardholder photograph"
          onError={() => setFailed(true)}
        />
      ) : (
        <>
          <Icon type="person" size={32} />
          <span>{!reveal ? "Hidden" : failed ? "No preview" : "No photo"}</span>
        </>
      )}
    </div>
  );
}

function Signature({
  bytes,
  reveal,
}: {
  bytes: number[] | null;
  reveal: boolean;
}) {
  const mime =
    bytes?.[0] === 255 && bytes?.[1] === 216
      ? "image/jpeg"
      : bytes?.[0] === 137 && bytes?.[1] === 80
        ? "image/png"
        : "";
  const src = useImage(bytes, reveal, mime);
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [src]);
  if (!bytes?.length || !reveal) return null;
  return (
    <section className="signature">
      <h3>Holder signature</h3>
      {src && !failed ? (
        <img src={src} alt="Holder signature" onError={() => setFailed(true)} />
      ) : (
        <p className="subtle">
          Payload available. This image format cannot be previewed.
        </p>
      )}
    </section>
  );
}

function Value({
  value,
  reveal = true,
}: {
  value?: string | null;
  reveal?: boolean;
}) {
  return (
    <span dir="auto" className={!value || !reveal ? "missing" : ""}>
      {!reveal ? "Hidden" : value || "Not available"}
    </span>
  );
}
function Copy({ value, name }: { value?: string | null; name: string }) {
  const [result, setResult] = useState("");
  useEffect(() => {
    if (!result) return;
    const timer = setTimeout(() => setResult(""), 1800);
    return () => clearTimeout(timer);
  }, [result]);
  if (!value) return null;
  return (
    <span className="copy-control">
      <button
        className="copy-button"
        title={`Copy ${name}`}
        aria-label={`Copy ${name}`}
        onClick={() => {
          void navigator.clipboard
            .writeText(value)
            .then(() => setResult("Copied"))
            .catch(() => setResult("Unable to copy"));
        }}
      >
        <Icon type={result === "Copied" ? "check" : "copy"} size={14} />
      </button>
      <span className="copy-feedback" role="status">
        {result}
      </span>
    </span>
  );
}
function CardField({
  name,
  arabic,
  value,
  reveal,
  numeric = false,
}: {
  name: string;
  arabic?: string;
  value?: string | null;
  reveal: boolean;
  numeric?: boolean;
}) {
  return (
    <div className={`card-field ${numeric ? "numeric" : ""}`}>
      <div className="card-label">
        <span>{name}</span>
        <span lang="ar" dir="rtl">
          {arabic}
        </span>
      </div>
      <div className="card-value">
        <Value value={value} reveal={reveal} />
        {numeric && reveal && <Copy value={value} name={name} />}
      </div>
    </div>
  );
}
function Overview({ scan, reveal }: { scan: Scan; reveal: boolean }) {
  const d = displayData(scan),
    f = d.nonModifiable;
  return (
    <div className="overview">
      <div className="card-stage">
        <section
          className="emirates-card"
          aria-label="Emirates ID cardholder overview"
        >
          <div className="card-lines" aria-hidden="true" />
          <header className="card-heading">
            <div>
              <span className="english-country-art">
                <img
                  src="/assets/english-country-heading.webp"
                  alt="United Arab Emirates"
                />
              </span>
              <span>Identity Card</span>
            </div>
            <img
              className="card-emblem"
              src="/assets/uae-emblem.svg"
              alt="United Arab Emirates emblem"
            />
            <div lang="ar" dir="rtl">
              <span className="arabic-country-art">
                <img
                  src="/assets/arabic-country-heading.webp"
                  alt="الإمارات العربية المتحدة"
                />
              </span>
              <span>بطاقة هوية</span>
            </div>
          </header>
          <div className="card-content">
            <div className="card-portrait">
              <Photo bytes={d.photoJpeg} reveal={reveal} />
              <div className="card-signature">
                <Signature
                  bytes={scan.signaturePreviewPng ?? d.holderSignatureImage}
                  reveal={reveal}
                />
                {(!d.holderSignatureImage?.length || !reveal) && (
                  <span className="missing">
                    {reveal ? "Not available" : "Hidden"}
                  </span>
                )}
                <div>
                  Signature /{" "}
                  <span lang="ar" dir="rtl">
                    التوقيع
                  </span>
                </div>
              </div>
            </div>
            <div className="card-person">
              <div className="card-id-row">
                <CardField
                  name="ID Number"
                  arabic="رقم الهوية"
                  value={d.idNumber}
                  reveal={reveal}
                  numeric
                />
                <span className="card-wave-flag">
                  <img
                    src="/assets/uae-flag-sheet.png"
                    alt="United Arab Emirates flag"
                  />
                </span>
              </div>
              <div className="card-names">
                <div className="name-arabic" lang="ar" dir="rtl">
                  <span className="name-label">الاسم: </span>
                  <Value value={f.fullNameArabic} reveal={reveal} />
                </div>
                <div className="name-english">
                  <span className="name-label">Name: </span>
                  <Value value={f.fullNameEnglish} reveal={reveal} />
                </div>
              </div>
              <div className="card-birth">
                <CardField
                  name="Date of Birth"
                  arabic="تاريخ الميلاد"
                  value={f.dateOfBirth}
                  reveal={reveal}
                  numeric
                />
              </div>
              <div className="card-nationality">
                <div>
                  <span className="name-label">Nationality: </span>
                  <Value
                    value={f.nationalityEnglish || f.nationalityCode}
                    reveal={reveal}
                  />
                </div>
                <div lang="ar" dir="rtl">
                  <span className="name-label">الجنسية: </span>
                  <Value value={f.nationalityArabic} reveal={reveal} />
                </div>
              </div>
              <div className="card-lower">
                <div className="card-validity">
                  <CardField
                    name="Issuing Date"
                    arabic="تاريخ الإصدار"
                    value={f.issueDate}
                    reveal={reveal}
                    numeric
                  />
                  <CardField
                    name="Expiry Date"
                    arabic="تاريخ الانتهاء"
                    value={f.expiryDate}
                    reveal={reveal}
                    numeric
                  />
                </div>
                <div className="card-sex">
                  <div lang="ar" dir="rtl">
                    الجنس:{" "}
                    <Value
                      value={
                        f.gender === "M"
                          ? "ذكر"
                          : f.gender === "F"
                            ? "أنثى"
                            : f.gender
                      }
                      reveal={reveal}
                    />
                  </div>
                  <div>
                    Sex: <Value value={f.gender} reveal={reveal} />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}

function FieldList({
  title,
  fields,
  filter,
  availableOnly,
}: {
  title: string;
  fields: Fields;
  filter: string;
  availableOnly: boolean;
}) {
  const entries = pairFields(fields, groupOrder(title)).filter(
    (row) =>
      [label(row.key), row.value, row.arabic].some((value) =>
        value?.toLowerCase().includes(filter.toLowerCase()),
      ) &&
      (!availableOnly || row.value || row.arabic),
  );
  return (
    <section className="field-group">
      <h3>
        {title}
        <span>{entries.length}</span>
      </h3>
      {entries.length ? (
        <div className="field-list-grid">
          {[
            entries.slice(0, Math.ceil(entries.length / 2)),
            entries.slice(Math.ceil(entries.length / 2)),
          ]
            .filter((column) => column.length)
            .map((column, index) => (
              <dl key={index}>
                {column.map((row) => (
                  <div
                    className={`field-row ${row.bilingual ? "bilingual-row" : ""} ${/date/i.test(row.key) ? "date-row" : ""}`}
                    key={row.key}
                  >
                    <dt>{label(row.key)}</dt>
                    <dd>
                      {row.bilingual ? (
                        // Both scripts start at the same edge and are tagged, so the
                        // eye never has to jump between opposite margins to compare.
                        <div className="bilingual-values">
                          {row.value && (
                            <div className="value-line">
                              <span className="lang-tag">EN</span>
                              <span lang="en" dir="ltr">
                                {row.value}
                              </span>
                              <Copy
                                value={row.value}
                                name={`${label(row.key)} in English`}
                              />
                            </div>
                          )}
                          {row.arabic && (
                            <div className="value-line">
                              <span className="lang-tag">AR</span>
                              <span lang="ar" dir="rtl">
                                {row.arabic}
                              </span>
                              <Copy
                                value={row.arabic}
                                name={`${label(row.key)} in Arabic`}
                              />
                            </div>
                          )}
                          {!row.value && !row.arabic && (
                            <div className="value-line">
                              <span className="lang-tag" aria-hidden="true" />
                              <Value value={null} />
                            </div>
                          )}
                        </div>
                      ) : (
                        // The empty tag keeps single-language values on the same
                        // starting edge as the tagged bilingual ones.
                        <div className="value-line">
                          <span className="lang-tag" aria-hidden="true" />
                          <Value value={row.value} />
                          <Copy value={row.value} name={label(row.key)} />
                        </div>
                      )}
                    </dd>
                  </div>
                ))}
              </dl>
            ))}
        </div>
      ) : (
        <p className="subtle">No matching fields in this group.</p>
      )}
    </section>
  );
}

function Inspector({
  scan,
  autoRead,
  paused,
}: {
  scan: Scan | null;
  autoRead: boolean;
  paused: boolean;
}) {
  const [filter, setFilter] = useState("");
  const [availableOnly, setAvailableOnly] = useState(false);
  useEffect(() => {
    setFilter("");
  }, [scan]);
  return (
    <section className="inspector panel">
      <div className="inspector-heading">
        <div className="title-line">
          <h2>Card details</h2>
          {scan && scan.data.cardGeneration.toLowerCase() !== "unknown" && (
            <span className="chip">
              {scan.data.cardGeneration.toUpperCase()}
            </span>
          )}
        </div>
      </div>
      <div id="details-panel" className="details-body">
        {!scan ? (
          <div className="empty-state">
            {/* Decoding a multi-frame animation can block the main thread;
                async lets the rest of the panel paint first. */}
            {!paused && (
              <img
                className="waiting-animation"
                src="/assets/card-reader-clean.webp"
                alt="Card entering a reader"
                decoding="async"
                width={280}
                height={350}
              />
            )}
            <h3>{paused ? "Reading stopped" : "Ready for a card"}</h3>
            <p>
              {paused ? (
                "Select Resume reading to continue."
              ) : autoRead ? (
                "Insert a card to read it automatically."
              ) : (
                <>
                  Connect a reader and select <strong>Read card</strong>.
                </>
              )}
              <br />
              Your results will appear here.
            </p>
          </div>
        ) : (
          <>
            <div className="field-tools">
              <input
                type="search"
                aria-label="Find a field"
                placeholder="Find a field…"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
              />
              <label>
                <input
                  type="checkbox"
                  checked={availableOnly}
                  onChange={(e) => setAvailableOnly(e.target.checked)}
                />
                Available only
              </label>
            </div>
            {scan.data.readStatus.modifiable === "not_requested" && (
              <p className="subtle">
                Extended details were not requested for this read.
              </p>
            )}
            <div className="field-columns">
              {detailGroups(displayData(scan)).map((group) => (
                <FieldList
                  key={group.title}
                  title={group.title}
                  fields={group.fields}
                  filter={filter}
                  availableOnly={availableOnly}
                />
              ))}
            </div>
          </>
        )}
      </div>
    </section>
  );
}

export function App() {
  const state = useSyncExternalStore(
    controller.subscribe,
    controller.getSnapshot,
  );
  useEffect(() => {
    document.documentElement.scrollTop = 0;
    document.body.scrollTop = 0;
  }, [state.scan]);
  const [autoRead, setAutoRead] = useState(true);
  const [ready, setReady] = useState(false);
  const [monitorError, setMonitorError] = useState("");
  function changeReadingMode(enabled: boolean) {
    setAutoRead(enabled);
    controller.setAutoRead(enabled);
  }
  useEffect(() => {
    let disposed = false,
      unlisten: (() => void) | undefined;
    void api
      .onRemoved(controller.removed)
      .then((stop) => {
        if (disposed) {
          stop();
          return;
        }
        unlisten = stop;
        setReady(true);
        void controller.refresh();
      })
      .catch(() =>
        setMonitorError("Card detection could not start. Restart the app."),
      );
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  useEffect(() => {
    if (!ready || monitorError) return;
    let disposed = false,
      timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      await controller.pollPresence();
      if (!disposed) timer = setTimeout(poll, 500);
    };
    void poll();
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [ready, monitorError]);
  useEffect(() => {
    const clear = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (!(e.target instanceof HTMLInputElement)) controller.stop();
    };
    window.addEventListener("keydown", clear);
    return () => window.removeEventListener("keydown", clear);
  }, []);
  return (
    <div className="app-shell reader-page">
      <main className={`workspace ${state.scan ? "with-card" : ""}`}>
        <aside className="reader-panel panel">
          <div className="section-heading">
            <h2>Card reader</h2>
          </div>
          <div className="reader-device">
            <span className="input-label">Connected device</span>
            <div className="select-row">
              <ReaderSelect
                readers={state.readers}
                selected={state.selected}
                disabled={!api.native || state.busy || !state.readers.length}
                onSelect={(name) => controller.select(name)}
              />
              <button
                className="icon-button"
                title="Refresh readers"
                aria-label="Refresh readers"
                disabled={!api.native || state.busy}
                onClick={() => void controller.refresh()}
              >
                <Icon type="refresh" size={16} />
              </button>
            </div>
            <p className="device-hint">
              {!api.native
                ? "Reader access needs the desktop app."
                : state.readers.length
                  ? `${state.readers.length} reader${state.readers.length === 1 ? "" : "s"} connected`
                  : "No reader detected. Connect one, then refresh."}
            </p>
          </div>
          {/* Changing the mode is a one-click choice, so it belongs here
                rather than behind a trip to the settings page. */}
          <div className="reading-mode">
            <span className="input-label">Reading mode</span>
            <div className="segmented" role="group" aria-label="Reading mode">
              {(
                [
                  [false, "Manual", "Read when you press Read card."],
                  [true, "Automatic", "Read as soon as a card is inserted."],
                ] as const
              ).map(([value, title, hint]) => (
                <button
                  key={title}
                  type="button"
                  className={autoRead === value ? "selected" : ""}
                  aria-pressed={autoRead === value}
                  title={hint}
                  onClick={() => changeReadingMode(value)}
                >
                  {title}
                </button>
              ))}
            </div>
            <p className="device-hint">
              {autoRead
                ? "Insert a card and its details appear."
                : "You decide when to read a card."}
            </p>
          </div>
          <div className="reader-actions">
            <button
              className="primary"
              disabled={
                !api.native ||
                !ready ||
                !!monitorError ||
                !state.selected ||
                state.busy
              }
              onClick={() =>
                void (state.paused ? controller.resume() : controller.read())
              }
            >
              {state.busy ? (
                <>
                  <span className="spinner" />
                  Reading card…
                </>
              ) : (
                <>
                  {state.paused ? "Resume reading" : "Read card"}
                  <Icon type="arrow" size={17} />
                </>
              )}
            </button>
            <button
              className="secondary stop-button"
              disabled={!api.native || state.paused}
              onClick={() => controller.stop()}
            >
              Stop reading
            </button>
          </div>
          {(state.error || monitorError) && (
            <p className="notice error" role="alert">
              {monitorError || state.notice}
            </p>
          )}
        </aside>
        {state.scan && !state.busy ? (
          <Overview scan={state.scan} reveal={true} />
        ) : (
          <div
            className={`overview scan-placeholder ${state.busy ? "scanning" : ""}`}
            role="status"
            aria-label={state.busy ? "Reading card" : "Waiting for a card"}
          >
            <div className="id-skeleton" aria-hidden="true">
              <div className="sk-header sk-header-left">
                <i />
                <i />
                <i />
              </div>
              <div className="sk-header sk-header-right">
                <i />
                <i />
                <i />
              </div>
              <div className="sk-portrait">
                <svg viewBox="0 0 100 130" fill="none">
                  <path
                    d="M9 130v-14c0-21 17-31 30-34V71h22v11c13 3 30 13 30 34v14"
                    fill="currentColor"
                  />
                  <ellipse
                    cx="50"
                    cy="48"
                    rx="24"
                    ry="31"
                    fill="currentColor"
                  />
                </svg>
              </div>
              <div className="sk-signature">
                <i />
              </div>
              <div className="sk-id">
                <i />
                <i />
              </div>
              <div className="sk-flag" />
              <i className="sk-name-en" />
              <div className="sk-birth">
                <i />
                <i />
              </div>
              <i className="sk-nationality-en" />
              <div className="sk-dates">
                <i />
                <i />
                <i />
                <i />
              </div>
              <div className="sk-sex">
                <i />
              </div>
            </div>
          </div>
        )}
        <Inspector
          scan={state.scan}
          paused={state.paused}
          autoRead={autoRead}
        />
      </main>
    </div>
  );
}
