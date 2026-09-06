import type { CardData, Fields, Scan } from "./model";

const months = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];
const dateFields = new Set([
  "dateOfBirth",
  "issueDate",
  "expiryDate",
  "residencyExpiryDate",
  "passportIssueDate",
  "passportExpiryDate",
  "dateOfGraduation",
]);

// Calendar-only formatting: no timezone conversion and no inferred dates.
export function formatDate(value: string | null): string | null {
  if (!value) return value;
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return value;
  const [, year, month, day] = match;
  const y = Number(year),
    m = Number(month),
    d = Number(day);
  const leap = y % 4 === 0 && (y % 100 !== 0 || y % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  if (y === 0 || m < 1 || m > 12 || d < 1 || d > days[m - 1]) return value;
  return `${day} ${months[m - 1]} ${year}`;
}

function displayFields(fields: Fields): Fields {
  return Object.fromEntries(
    Object.entries(fields).map(([key, value]) => {
      if (dateFields.has(key)) return [key, formatDate(value)];
      // These extended names have no library formatting accessor. This is an app-only
      // display choice; company names and other free text retain punctuation.
      if (/^(motherFullName(English|Arabic)|sponsorName)$/.test(key) && value) {
        return [
          key,
          value
            .split(",")
            .map((part) => part.trim())
            .filter(Boolean)
            .join(" ") || null,
        ];
      }
      return [key, value];
    }),
  );
}

// The bridge's raw library snapshot is never edited. Both UI views use this copy.
export function displayData(scan: Scan): CardData {
  return {
    ...scan.data,
    idNumber: scan.display.idNumber,
    nonModifiable: {
      ...displayFields(scan.data.nonModifiable),
      fullNameEnglish: scan.display.fullNameEnglish,
      fullNameArabic: scan.display.fullNameArabic,
      gender: scan.display.genderCode,
    },
    modifiable: displayFields(scan.data.modifiable),
  };
}
