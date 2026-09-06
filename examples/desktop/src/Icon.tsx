export function Icon({
  type = "card",
  size = 20,
}: {
  type?:
    | "card"
    | "refresh"
    | "arrow"
    | "shield"
    | "person"
    | "copy"
    | "chevron"
    | "check";
  size?: number;
}) {
  const paths = {
    chevron: <path d="m6 9.5 6 6 6-6" />,
    copy: (
      <>
        <rect x="8" y="8" width="12" height="13" rx="2" />
        <path d="M16 8V3H3v13h5" />
      </>
    ),
    check: <path d="m5 12 4 4L19 6" />,
    card: (
      <>
        <rect x="3" y="5" width="18" height="14" rx="3" />
        <path d="M7 10h3v4H7zM14 10h3m-3 4h3" />
      </>
    ),
    // A single arc reads more clearly than the two-arrow form at 16px.
    refresh: (
      <>
        <path d="M20.5 13a8.5 8.5 0 1 1-2.2-7.1L21 8.5" />
        <path d="M21 4v4.5h-4.5" />
      </>
    ),
    arrow: <path d="M5 12h14m-5-5 5 5-5 5" />,
    shield: (
      <>
        <path d="m12 3 8 3v6c0 4-5 7-8 9-3-2-8-5-8-9V6z" />
        <path d="m8 12 3 3 5-6" />
      </>
    ),
    person: (
      <>
        <circle cx="12" cy="8" r="4" />
        <path d="M4 21v-2a8 8 0 0 1 16 0v2" />
      </>
    ),
  };
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {paths[type]}
    </svg>
  );
}
