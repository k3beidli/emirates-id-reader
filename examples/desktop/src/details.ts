import type { CardData, Fields } from "./model";

export type DetailRow = {
  key: string;
  value: string | null;
  arabic?: string | null;
  bilingual: boolean;
};

// Raw codes the card stores for machine use. The readable description is shown
// instead, so these would only add noise for a non-technical reader.
export const hiddenFields = new Set([
  "nationalityCode",
  "cardNumber",
  "occupationCode",
  "fieldOfStudyCode",
]);

// Explicit order within a group, by paired base name. Anything unlisted keeps
// its card order and follows the listed rows.
const fieldOrder: Record<string, string[]> = {
  "Personal information": [
    "fullName",
    "title",
    "gender",
    "dateOfBirth",
    "nationality",
    "placeOfBirth",
  ],
  "Card information": ["idNumber", "idType", "issueDate", "expiryDate"],
};

export function pairFields(fields: Fields, order: string[] = []): DetailRow[] {
  const seen = new Set<string>();
  const rows = Object.keys(fields).flatMap((key) => {
    const base = key.replace(/(English|Arabic)$/, "");
    const bilingual = base !== key;
    if (seen.has(base) || hiddenFields.has(base)) return [];
    seen.add(base);
    return [
      {
        key: base,
        bilingual,
        value: bilingual ? (fields[base + "English"] ?? null) : fields[key],
        ...(bilingual ? { arabic: fields[base + "Arabic"] ?? null } : {}),
      },
    ];
  });
  if (!order.length) return rows;
  const rank = (key: string) => {
    const index = order.indexOf(key);
    return index === -1 ? order.length : index;
  };
  return rows
    .map((row, index) => ({ row, index }))
    .sort((a, b) => rank(a.row.key) - rank(b.row.key) || a.index - b.index)
    .map(({ row }) => row);
}

export function groupOrder(title: string): string[] {
  return fieldOrder[title] ?? [];
}

export function detailGroups(data: CardData) {
  const groups = [
    "Personal information",
    "Card information",
    "Occupation & employer",
    "Family",
    "Passport",
    "Residency & sponsor",
    "Education",
    "Other details",
  ].map((title) => ({ title, fields: {} as Fields }));
  const all = {
    ...data.nonModifiable,
    ...data.modifiable,
    idNumber: data.idNumber,
    cardNumber: data.cardNumber,
  };
  for (const [key, value] of Object.entries(all)) {
    const index =
      /^(title|fullName|gender|nationality|dateOfBirth|placeOfBirth)/.test(key)
        ? 0
        : /^(idNumber|cardNumber|idType|issueDate|expiryDate|issuingPlace|issuePlace|cardType)$/.test(
              key,
            )
          ? 1
          : /^(occupation|company|employer)/.test(key)
            ? 2
            : /^(family|marital|husband|mother)/.test(key)
              ? 3
              : /^passport/.test(key)
                ? 4
                : /^(residency|sponsor)/.test(key)
                  ? 5
                  : /^(qualification|degree|fieldOfStudy|placeOfStudy|dateOfGraduation)/.test(
                        key,
                      )
                    ? 6
                    : 7;
    groups[index].fields[key] = value;
  }
  return groups.filter((group) => pairFields(group.fields).length > 0);
}
