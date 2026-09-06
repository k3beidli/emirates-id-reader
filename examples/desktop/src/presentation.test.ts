import { expect, it } from "vitest";
import { displayData, formatDate } from "./presentation";
import type { Scan } from "./model";
it("formats calendar dates without timezone changes and preserves invalid input", () => {
  expect(formatDate("2008-07-12")).toBe("12 Jul 2008");
  expect(formatDate("2000-02-29")).toBe("29 Feb 2000");
  for (const date of [
    null,
    "",
    "1900-02-29",
    "2026-02-30",
    "unknown",
    "2026-13-01",
  ])
    expect(formatDate(date)).toBe(date);
});
it("uses library display names and identifiers in both views without changing raw data", () => {
  const scan = {
    display: {
      fullNameEnglish: "SYNTHETIC HOLDER",
      fullNameArabic: null,
      idNumber: "000-0000-0000000-0",
      genderCode: "M",
    },
    data: {
      idNumber: "000000000000000",
      nonModifiable: {
        fullNameEnglish: "SYNTHETIC,,HOLDER",
        fullNameArabic: ",,",
        dateOfBirth: "2008-07-12",
        gender: "m",
      },
      modifiable: {
        passportExpiryDate: "2030-01-02",
        motherFullNameEnglish: "TEST,,MOTHER",
        companyNameEnglish: "Company, Inc.",
        occupationCode: "001",
      },
    },
  } as unknown as Scan;
  const before = JSON.stringify(scan);
  const view = displayData(scan);
  expect(view.nonModifiable.fullNameEnglish).toBe("SYNTHETIC HOLDER");
  expect(view.nonModifiable.fullNameArabic).toBeNull();
  expect(view.nonModifiable.dateOfBirth).toBe("12 Jul 2008");
  expect(view.modifiable.passportExpiryDate).toBe("02 Jan 2030");
  expect(view.modifiable.motherFullNameEnglish).toBe("TEST MOTHER");
  expect(view.modifiable.companyNameEnglish).toBe("Company, Inc.");
  expect(view.modifiable.occupationCode).toBe("001");
  expect(view.idNumber).toBe(scan.display.idNumber);
  expect(JSON.stringify(scan)).toBe(before);
});
