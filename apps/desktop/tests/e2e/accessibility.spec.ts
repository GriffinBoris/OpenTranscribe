import AxeBuilder from "@axe-core/playwright";
import type { Page } from "@playwright/test";

import { expect, test } from "./fixtures";

const applicationRoutes = [
  { path: "/", heading: "Home" },
  { path: "/inbox", heading: "Inbox" },
  { path: "/processing", heading: "Processing" },
  { path: "/projects/01KDEMOPROJECT", heading: "Product" },
  { path: "/sessions/01KDEMOSESSION1", heading: "Weekly product sync" },
  { path: "/settings", heading: "Settings" },
  { path: "/trash", heading: "Trash" },
];

async function expectNoAccessibilityViolations(page: Page) {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
    .analyze();

  expect(
    results.violations.length,
    results.violations
      .map(
        (violation) =>
          `${violation.id}: ${violation.help}\n${violation.nodes
            .map(
              (node) =>
                `  ${node.target.join(" ")}\n    ${node.failureSummary ?? ""}`,
            )
            .join("\n")}`,
      )
      .join("\n\n"),
  ).toBe(0);
}

for (const route of applicationRoutes) {
  test(`${route.heading} has no detectable light or dark WCAG A or AA violations`, async ({
    page,
  }) => {
    await page.goto(route.path);
    await expect(
      page.getByRole("heading", { name: route.heading, exact: true }),
    ).toBeVisible();

    await expectNoAccessibilityViolations(page);
    await page.evaluate(() => {
      document.documentElement.dataset.theme = "dark";
    });
    await page.waitForTimeout(200);
    await expectNoAccessibilityViolations(page);
  });
}

test("recording options remains accessible as a modal workflow", async ({
  page,
}) => {
  await page.goto("/");
  await page
    .getByRole("button", { name: "Recording options", exact: true })
    .click();

  const dialog = page.getByRole("dialog", { name: "Recording options" });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator(":focus")).toBeVisible();
  await expectNoAccessibilityViolations(page);

  await page.keyboard.press("Escape");
  await expect(dialog).not.toBeVisible();
  await expect(
    page.getByRole("button", { name: "Recording options", exact: true }),
  ).toBeFocused();
});

test("primary recording controls are reachable and operable by keyboard", async ({
  page,
}) => {
  await page.goto("/");

  const newRecording = page.getByRole("button", { name: "New recording" });
  await newRecording.focus();
  await page.keyboard.press("Enter");

  const notes = page.getByRole("textbox", { name: "Meeting notes" });
  await expect(notes).toBeVisible();
  await notes.focus();
  await page.keyboard.press("Control+Shift+M");
  await expect(notes).toHaveValue(/- \[\d+:\d{2}\] /);

  const stop = page.getByRole("button", { name: "Stop" });
  await stop.focus();
  await page.keyboard.press("Space");
  await expect(stop).not.toBeVisible();
});

test("global recording shortcut settings remain accessible when enabled", async ({
  page,
}) => {
  await page.goto("/settings#shortcuts");

  const globalRecordingToggle = page.getByRole("switch", {
    name: "Record from anywhere",
  });
  await globalRecordingToggle.focus();
  await page.keyboard.press("Space");
  await expect(page.getByLabel("Global recording shortcut")).toBeVisible();
  await expectNoAccessibilityViolations(page);
});
