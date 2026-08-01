import { expect, test } from "./fixtures";

test("keeps the shared search icon centered inside its input", async ({
  page,
}) => {
  await page.goto("/inbox");

  const searchField = page.locator(".app-search-input");
  const searchIcon = searchField.locator(".app-search-input__icon svg");

  await expect(page.locator(".library-toolbar")).toHaveClass(/mb-4/);
  await expect(searchField).not.toHaveClass(/mb-4/);
  await expect(searchIcon).toHaveCSS("width", "16px");
  await expect(searchIcon).toHaveCSS("height", "16px");

  const centerOffset = await searchField.evaluate((field) => {
    const input = field.querySelector("input");
    const icon = field.querySelector(".app-search-input__icon svg");

    if (!input || !icon) {
      throw new Error("Search input geometry is unavailable");
    }

    const inputBounds = input.getBoundingClientRect();
    const iconBounds = icon.getBoundingClientRect();

    return (
      iconBounds.top +
      iconBounds.height / 2 -
      (inputBounds.top + inputBounds.height / 2)
    );
  });

  expect(Math.abs(centerOffset)).toBeLessThanOrEqual(0.5);
});

test("overlays each library checkbox on its visual control", async ({
  page,
}) => {
  await page.goto("/inbox");

  const checkbox = page.getByLabel("Select Customer discovery — Rowan");
  const geometry = await checkbox.evaluate((input) => {
    const root = input.closest(".app-checkbox");
    const box = root?.querySelector(".app-checkbox__box");

    if (!root || !box) {
      throw new Error("Checkbox geometry is unavailable");
    }

    const rootBounds = root.getBoundingClientRect();
    const inputBounds = input.getBoundingClientRect();
    const boxBounds = box.getBoundingClientRect();

    return { rootBounds, inputBounds, boxBounds };
  });

  expect(geometry.rootBounds.width).toBe(20);
  expect(geometry.inputBounds.x).toBe(geometry.boxBounds.x);
  expect(geometry.inputBounds.y).toBe(geometry.boxBounds.y);

  await checkbox.click();
  await expect(checkbox.locator("xpath=..")).toHaveAttribute(
    "data-p-checked",
    "true",
  );
  await expect(checkbox.locator("xpath=..").locator("svg")).toBeVisible();
});

test("filters sessions within a project", async ({ page }) => {
  await page.goto("/projects/01KDEMOPROJECT");

  const search = page.getByPlaceholder("Search this project");
  await search.fill("weekly");
  await expect(page.getByText("Weekly product sync")).toBeVisible();

  await search.fill("missing session");
  await expect(page.getByText("No sessions match this search.")).toBeVisible();
});

test("shows a useful empty state when Inbox has no meetings", async ({
  page,
}) => {
  await page.goto("/inbox");

  await page.getByLabel("Select Customer discovery — Rowan").click();
  await page.getByRole("button", { name: "Move 1 meeting" }).click();
  const dialog = page.getByRole("dialog", { name: "Move meeting" });
  await dialog.getByLabel("Destination").click();
  await page.getByRole("option", { name: "Product", exact: true }).click();
  await dialog.getByRole("button", { name: "Move meetings" }).click();

  await expect(page.getByText("Inbox is empty")).toBeVisible();
  await expect(
    page.getByText("Record a meeting or import media to get started."),
  ).toBeVisible();
});

test("renders substantial smoothly composited processing progress", async ({
  page,
}) => {
  await page.goto("/processing");

  const progress = page.locator(".app-progress").first();
  const value = progress.locator(".app-progress__value");

  await expect(progress).toHaveCSS("height", "10px");
  await expect(value).toHaveCSS("transition-property", "transform");
  await expect
    .poll(() =>
      value.evaluate((element) => getComputedStyle(element).transform),
    )
    .not.toBe("none");
});

test("aligns processing columns and reserves room for job actions", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/processing");

  const headerCells = page.locator(".processing-table__header > span");
  const job = page.locator(".processing-row").first();
  const provider = job.locator(".processing-row__provider");
  const progress = job.locator(".processing-row__progress");
  const status = job.locator(".processing-row__status");
  const cancel = page.getByRole("button", { name: "Cancel" });
  const [
    headerProvider,
    headerProgress,
    headerStatus,
    providerBox,
    progressBox,
    statusBox,
    cancelBox,
  ] = await Promise.all([
    headerCells.nth(1).boundingBox(),
    headerCells.nth(2).boundingBox(),
    headerCells.nth(3).boundingBox(),
    provider.boundingBox(),
    progress.boundingBox(),
    status.boundingBox(),
    cancel.boundingBox(),
  ]);

  expect(headerProvider).not.toBeNull();
  expect(headerProgress).not.toBeNull();
  expect(headerStatus).not.toBeNull();
  expect(providerBox).not.toBeNull();
  expect(progressBox).not.toBeNull();
  expect(statusBox).not.toBeNull();
  expect(cancelBox).not.toBeNull();

  expect(Math.abs(headerProvider!.x - providerBox!.x)).toBeLessThanOrEqual(1);
  expect(Math.abs(headerProgress!.x - progressBox!.x)).toBeLessThanOrEqual(1);
  expect(Math.abs(headerStatus!.x - statusBox!.x)).toBeLessThanOrEqual(1);
  expect(cancelBox!.x + cancelBox!.width).toBeLessThanOrEqual(
    statusBox!.x + statusBox!.width,
  );
});

test("keeps processing and session actions inside compact workspaces", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await page.goto("/processing");

  const jobs = page.locator(".processing-page__jobs");
  const cancel = page.getByRole("button", { name: "Cancel" });
  const [jobsBox, cancelBox] = await Promise.all([
    jobs.boundingBox(),
    cancel.boundingBox(),
  ]);

  expect(jobsBox).not.toBeNull();
  expect(cancelBox).not.toBeNull();
  expect(cancelBox?.x + cancelBox?.width).toBeLessThanOrEqual(
    jobsBox!.x + jobsBox!.width,
  );

  await page.goto("/sessions/01KDEMOSESSION1");

  const title = page.getByRole("heading", { name: "Weekly product sync" });
  const trash = page.getByRole("button", {
    name: "Move session to trash",
  });
  const [titleBox, trashBox] = await Promise.all([
    title.boundingBox(),
    trash.boundingBox(),
  ]);

  expect(titleBox).not.toBeNull();
  expect(trashBox).not.toBeNull();
  expect(trashBox!.x).toBeGreaterThan(titleBox!.x + titleBox!.width);
  expect(Math.abs(trashBox!.y - titleBox!.y)).toBeLessThan(12);
  await page.setViewportSize({ width: 800, height: 760 });
  await expect(
    page.locator(".sidebar__primary-actions .app-button").first(),
  ).toHaveCSS("width", "38px");
  await expect(
    page.getByText("New recording", { exact: true }),
  ).not.toBeVisible();
});

test("moves a session between a project and the inbox", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  await page.getByRole("button", { name: "Move to project" }).click();
  const sessionMoveDialog = page.getByRole("dialog", { name: "Move meeting" });
  await sessionMoveDialog
    .getByRole("button", { name: "Move meetings" })
    .click();
  await expect(
    page.locator(".session-header__metadata").getByText("Inbox", {
      exact: true,
    }),
  ).toBeVisible();

  const notes = page.getByRole("textbox", { name: "Meeting notes" });
  await notes.click();
  await expect(notes).toHaveCSS("box-shadow", "none");

  await page.getByRole("link", { name: "Product", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).not.toBeVisible();

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).toBeVisible();

  await page.getByLabel("Select Customer discovery — Rowan").click();
  await page.getByRole("button", { name: "Move 1 meeting" }).click();
  const inboxMoveDialog = page.getByRole("dialog", { name: "Move meeting" });
  await inboxMoveDialog.getByLabel("Destination").click();
  await page.getByRole("option", { name: "Product", exact: true }).click();
  await inboxMoveDialog.getByRole("button", { name: "Move meetings" }).click();
  await expect(page.getByText("Customer discovery — Rowan")).not.toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("2");

  await page.getByRole("link", { name: "Product", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).toBeVisible();

  await page.getByLabel("Select Customer discovery — Rowan").click();
  await page.getByRole("button", { name: "Move 1 meeting" }).click();
  const projectMoveDialog = page.getByRole("dialog", { name: "Move meeting" });
  await projectMoveDialog
    .getByRole("button", { name: "Move meetings" })
    .click();
  await expect(page.getByText("Customer discovery — Rowan")).not.toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("1");

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).toBeVisible();
});

test("selects a meeting range before bulk moving it", async ({ page }) => {
  await page.goto("/projects/01KDEMOPROJECT");

  await page.getByLabel("Select Weekly product sync").click();
  await page
    .getByLabel("Select Architecture notes")
    .click({ modifiers: ["Shift"] });

  await expect(
    page.getByRole("button", { name: "Move 2 meetings" }),
  ).toBeEnabled();
});

test("selects a meeting from its focused checkbox", async ({ page }) => {
  await page.goto("/inbox");

  const checkbox = page.getByLabel("Select Customer discovery — Rowan");
  await checkbox.focus();
  await page.keyboard.press("Space");

  await expect(
    page.getByRole("button", { name: "Move 1 meeting" }),
  ).toBeEnabled();
});
