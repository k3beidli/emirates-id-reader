import { useEffect, useRef, useId, useState } from "react";
import { Icon } from "./Icon";

// A native select hands its open list to the engine, which renders it with the
// platform's own metrics. Reader names are long, so the list is built here to
// keep it readable and consistent across WebView2 versions.
export function ReaderSelect({
  readers,
  selected,
  disabled,
  onSelect,
}: {
  readers: string[];
  selected: string;
  disabled: boolean;
  onSelect: (name: string) => void;
}) {
  const listId = useId();
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const container = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const outside = (event: MouseEvent) => {
      if (!container.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", outside);
    return () => document.removeEventListener("mousedown", outside);
  }, [open]);
  useEffect(() => {
    if (open) setActive(Math.max(0, readers.indexOf(selected)));
  }, [open, readers, selected]);
  useEffect(() => {
    if (disabled) setOpen(false);
  }, [disabled]);
  useEffect(() => {
    if (open)
      document
        .getElementById(`${listId}-${active}`)
        ?.scrollIntoView?.({ block: "nearest" });
  }, [open, active, listId]);
  const choose = (name: string) => {
    onSelect(name);
    setOpen(false);
  };
  const keys = (event: React.KeyboardEvent) => {
    if (disabled) return;
    if (event.key === "Tab") setOpen(false);
    if (event.key === "Escape" && open) {
      event.preventDefault();
      // Otherwise the window handler would clear the card behind the list.
      event.stopPropagation();
      setOpen(false);
      return;
    }
    if (!open) {
      if (["Enter", " ", "ArrowDown", "ArrowUp"].includes(event.key)) {
        event.preventDefault();
        setOpen(true);
      }
      return;
    }
    if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      setActive(event.key === "Home" ? 0 : readers.length - 1);
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      setActive((index) =>
        Math.min(Math.max(index + step, 0), readers.length - 1),
      );
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (readers[active]) choose(readers[active]);
    }
  };
  return (
    <div
      className="reader-select"
      ref={container}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node))
          setOpen(false);
      }}
    >
      <button
        type="button"
        className={`reader-select-trigger ${open ? "open" : ""}`}
        disabled={disabled}
        role="combobox"
        aria-label="Connected device"
        aria-controls={listId}
        aria-activedescendant={open ? `${listId}-${active}` : undefined}
        aria-haspopup="listbox"
        aria-expanded={open}
        title={selected || undefined}
        onClick={() => setOpen((value) => !value)}
        onKeyDown={keys}
      >
        <span className={selected ? "" : "placeholder"}>
          {selected || "No readers available"}
        </span>
        <Icon type="chevron" size={14} />
      </button>
      {open && readers.length > 0 && (
        <ul
          className="reader-options"
          role="listbox"
          id={listId}
          aria-label="Connected readers"
        >
          {readers.map((name, index) => (
            <li
              key={name}
              id={`${listId}-${index}`}
              role="option"
              aria-selected={name === selected}
              className={index === active ? "active" : ""}
              onMouseDown={(event) => event.preventDefault()}
              onMouseEnter={() => setActive(index)}
              onClick={() => choose(name)}
            >
              <span>{name}</span>
              {name === selected && <Icon type="check" size={14} />}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
