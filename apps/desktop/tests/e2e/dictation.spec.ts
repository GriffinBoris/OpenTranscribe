import { expect, test } from "./fixtures";

test("can cancel while transcription is pending without a later result", async ({
  page,
}) => {
  await page.clock.install();
  await page.goto("/?dictation=1");
  await page.getByRole("button", { name: "Start", exact: true }).click();
  await page.getByRole("button", { name: "Stop and transcribe" }).click();
  await expect(page.getByText("Transcribing locally…")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Start", exact: true }),
  ).not.toBeVisible();
  await page.keyboard.press("Escape");
  await page.clock.fastForward(2000);
  await expect(page.getByText("Ready for dictation")).toBeVisible();
  await expect(
    page.getByText("Copied — paste when you’re ready"),
  ).not.toBeVisible();
});

test("shows cleanup and an explicit copy result at panel dimensions", async ({
  page,
}) => {
  await page.setViewportSize({ width: 520, height: 220 });
  await page.clock.install();
  await page.goto("/?dictation=1&dictationCleanup=1");
  await page.getByRole("button", { name: "Start", exact: true }).click();
  await page.getByRole("button", { name: "Stop and transcribe" }).click();
  await page.clock.fastForward(700);
  await expect(page.getByText("Cleaning up with S1-mini…")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeVisible();
  await page.clock.fastForward(500);
  await expect(
    page.getByText("Copied — paste when you’re ready"),
  ).toBeVisible();
  await expect(page.getByText("Send the draft on Tuesday.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Copy dictation", exact: true }),
  ).toBeVisible();
});

test("keeps a failed dictation recoverable and applies panel appearance", async ({
  page,
}) => {
  await page.goto(
    "/?dictation=1&dictationResult=failed&theme=dark&reducedMotion=1",
  );
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await expect(page.locator("html")).toHaveAttribute(
    "data-reduced-motion",
    "true",
  );
  await page.getByRole("button", { name: "Start", exact: true }).click();
  await page.getByRole("button", { name: "Stop and transcribe" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Transcription could not finish",
  );
  await expect(
    page.getByRole("button", { name: "Retry", exact: true }),
  ).toBeVisible();
  await expect(page.getByText(/Audio kept for retry/)).toBeVisible();
});

test("offers S1-mini separately from speech models and shows missing setup", async ({
  page,
}) => {
  await page.goto("/settings");
  await expect(
    page.getByText("Choose a speech model", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Download S1-mini · 484 MB" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Download S1-mini · 484 MB" }).click();
  await expect(
    page.getByRole("switch", { name: "Clean up dictation", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("switch", { name: "Clean up dictation", exact: true })
    .check();
  await expect(
    page.getByRole("switch", { name: "Clean up dictation", exact: true }),
  ).toBeChecked();
  await expect(
    page.getByText(/It cannot transcribe audio on its own/),
  ).toBeVisible();
  await page
    .getByRole("combobox", { name: "Speech recognition model", exact: true })
    .click();
  await expect(
    page.getByRole("option", { name: "S1-mini by Superwhisper" }),
  ).not.toBeVisible();
});
