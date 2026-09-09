import { expect, test } from "./fixtures";

test("recovers an unknown settings section and opens model setup", async ({
  page,
}) => {
  await page.goto("/settings#missing-section");
  await expect(page).toHaveURL(/#dictation$/);
  await expect(
    page.getByRole("heading", { name: "Dictation", exact: true }),
  ).toBeVisible();
  await page
    .locator("#dictation")
    .getByRole("button", { name: "Browse models", exact: true })
    .click();
  await expect(page).toHaveURL(/#models$/);
  await expect(
    page.getByRole("heading", { name: "Local models", exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("navigation", { name: "Settings sections" })
      .getByRole("link", { name: "Local models", exact: true }),
  ).toHaveAttribute("aria-current", "location");
});

test("shows partial selection and restores sessions after clearing a search", async ({
  page,
}) => {
  await page.goto("/projects/01KDEMOPROJECT");
  await page
    .getByRole("checkbox", { name: "Select Weekly product sync", exact: true })
    .check();
  await expect(
    page.getByRole("checkbox", { name: "Select all meetings" }),
  ).toBeChecked({ indeterminate: true });
  await expect(page.getByRole("status")).toHaveText("1 selected");
  await page
    .getByRole("searchbox", { name: "Search this project" })
    .fill("no matching session");
  await expect(page.getByText("No sessions match this search.")).toBeVisible();
  await page.getByRole("button", { name: "Clear search" }).click();
  await expect(
    page.getByRole("link", { name: "Weekly product sync", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("searchbox", { name: "Search this project" }),
  ).toHaveValue("");
});

test("opens the session associated with a processing job", async ({ page }) => {
  await page.goto("/processing");
  await page
    .getByRole("link", { name: "Architecture notes", exact: true })
    .click();
  await expect(page).toHaveURL(/\/sessions\/01KDEMOSESSION3$/);
  await expect(
    page.getByRole("heading", { name: "Architecture notes", exact: true }),
  ).toBeVisible();
});

test("keeps settings controls inside the minimum desktop window", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 640 });
  await page.goto("/settings");
  const model = page.getByRole("combobox", {
    name: "Local model",
    exact: true,
  });
  await expect(model).toBeVisible();
  const bounds = await model.boundingBox();
  expect(bounds).not.toBeNull();
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(900);
  const overflow = await page
    .locator(".settings-content")
    .evaluate((element) => element.scrollWidth - element.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
});
