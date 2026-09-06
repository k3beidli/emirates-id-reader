import { expect, it } from "vitest";
import { detailGroups, pairFields } from "./details";
import type { CardData } from "./model";

it("pairs languages regardless of order and preserves a lone Arabic value", () => {
  expect(
    pairFields({
      fullNameArabic: "Arabic fixture",
      fullNameEnglish: "English fixture",
      occupationArabic: "Occupation fixture",
    }),
  ).toEqual([
    {
      key: "fullName",
      bilingual: true,
      value: "English fixture",
      arabic: "Arabic fixture",
    },
    {
      key: "occupation",
      bilingual: true,
      value: null,
      arabic: "Occupation fixture",
    },
  ]);
});
it("organizes fields without dropping unknown future keys", () => {
  const data = {
    idNumber: "000",
    cardNumber: "001",
    nonModifiable: { fullNameEnglish: "Fixture", issueDate: null },
    modifiable: {
      occupationEnglish: "Job",
      familyId: "002",
      passportNumber: "003",
      futureField: "kept",
    },
  } as unknown as CardData;
  const groups = detailGroups(data);
  expect(groups.map((g) => g.title)).toEqual([
    "Personal information",
    "Card information",
    "Occupation & employer",
    "Family",
    "Passport",
    "Other details",
  ]);
  expect(Object.assign({}, ...groups.map((g) => g.fields))).toEqual({
    ...data.nonModifiable,
    ...data.modifiable,
    idNumber: "000",
    cardNumber: "001",
  });
});
