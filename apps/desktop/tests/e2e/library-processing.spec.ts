import { expect, test } from "./fixtures";

test("keeps the shared search icon centered inside its input", async ({
  page,
}) => {
  await page.goto("/inbox");

  const searchField = page.locator(".app-search-input");
  const searchInput = searchField.getByRole("searchbox");
  const searchIcon = searchField.locator(".app-search-input__icon svg");

  await expect(searchField).toHaveClass(/mb-4/);
  await expect(searchInput).not.toHaveClass(/mb-4/);
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

test("filters sessions within a project", async ({ page }) => {
  await page.goto("/projects/01KDEMOPROJECT");

  const search = page.getByPlaceholder("Search this project");
  await search.fill("weekly");
  await expect(page.getByText("Weekly product sync")).toBeVisible();

  await search.fill("missing session");
  await expect(page.getByText("No sessions match this search.")).toBeVisible();
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

  const project = page.getByLabel("Move to project");
  await expect(project).toContainText("Product");
  await project.click();
  await page.getByRole("option", { name: "Inbox", exact: true }).click();
  await expect(project).toContainText("Inbox");

  const notes = page.getByRole("textbox", { name: "Meeting notes" });
  await notes.click();
  await expect(notes).toHaveCSS("box-shadow", "none");

  await page.getByRole("link", { name: "Product", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).not.toBeVisible();

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).toBeVisible();

  const inboxRowProject = page.getByLabel(
    "Move Customer discovery — Rowan to a project or Inbox",
  );
  await expect(inboxRowProject).toContainText("Inbox");
  await inboxRowProject.click();
  await page.getByRole("option", { name: "Product", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).not.toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("2");

  await page.getByRole("link", { name: "Product", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).toBeVisible();

  const projectRowProject = page.getByLabel(
    "Move Customer discovery — Rowan to a project or Inbox",
  );
  await expect(projectRowProject).toContainText("Product");
  await projectRowProject.click();
  await page.getByRole("option", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).not.toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("1");

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).toBeVisible();
});
