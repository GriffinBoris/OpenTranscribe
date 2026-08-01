import { expect, test } from "./fixtures";

test("opens the utility shell and starts a preview recording", async ({
  page,
}) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Home" })).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Main navigation" }),
  ).toBeVisible();
  const brandIcon = page.getByTestId("app-brand-icon");
  await expect(brandIcon).toBeVisible();
  await expect
    .poll(() =>
      brandIcon.evaluate((image: HTMLImageElement) => image.naturalWidth),
    )
    .toBeGreaterThan(0);
  await expect(page.locator(".window-titlebar")).not.toHaveAttribute(
    "data-tauri-drag-region",
  );
  const dragRegion = page.locator(".window-titlebar__drag-region");
  await expect(dragRegion).toHaveAttribute("data-tauri-drag-region", "");
  await expect(dragRegion).toHaveCSS("left", "88px");
  await expect(dragRegion).toHaveCSS("right", "120px");
  await expect(page.locator(".window-titlebar__handle")).toBeVisible();
  await expect(page.getByText("Recent sessions")).toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("2");
  await expect(
    page
      .getByRole("link", { name: "Customer interviews", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("0");

  const newRecording = page.getByRole("button", { name: "New recording" });
  await expect(newRecording).toHaveCSS("justify-content", "flex-start");
  await expect(newRecording.locator("kbd")).toHaveCount(0);

  await page.keyboard.press("Control+K");
  await expect(
    page.getByRole("dialog", { name: "Search library" }),
  ).toBeVisible();
  await page
    .getByRole("dialog", { name: "Search library" })
    .evaluate(async (dialog) =>
      Promise.all(
        dialog
          .getAnimations({ subtree: true })
          .map((animation) => animation.finished),
      ),
    );
  const librarySearch = page.getByPlaceholder(
    "Search sessions, transcripts, and notes",
  );
  const librarySearchForm = page.locator(".library-search");
  const searchButton = page
    .getByRole("dialog")
    .getByRole("button", { name: "Search" });
  const [formBox, inputBox, buttonBox] = await Promise.all([
    librarySearchForm.boundingBox(),
    librarySearch.boundingBox(),
    searchButton.boundingBox(),
  ]);
  expect(formBox).not.toBeNull();
  expect(inputBox).not.toBeNull();
  expect(buttonBox).not.toBeNull();
  expect(Math.round(inputBox!.x)).toBe(Math.round(formBox!.x));
  expect(Math.round(buttonBox!.x + buttonBox!.width)).toBe(
    Math.round(formBox!.x + formBox!.width),
  );
  expect(Math.round(inputBox!.height)).toBe(Math.round(buttonBox!.height));
  await expect(page.locator(".app-search-input__icon")).toHaveCSS(
    "position",
    "absolute",
  );
  const searchIcon = page.locator(".app-search-input__icon svg");
  await expect(searchIcon).toHaveCSS("width", "16px");
  await expect(searchIcon).toHaveCSS("height", "16px");
  await expect(librarySearch).toHaveCSS("padding-left", "36px");
  await librarySearch.fill("weekly");
  await searchButton.click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("button", { name: /Weekly product sync/ }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.locator(".library-search__results").evaluate((results) => {
        const form = results.previousElementSibling;
        return form
          ? Math.round(
              results.getBoundingClientRect().top -
                form.getBoundingClientRect().bottom,
            )
          : 0;
      }),
    )
    .toBe(14);

  await librarySearch.fill("no matching session");
  await searchButton.click();
  const noMatches = page.locator(".library-search + .empty-setting");
  await expect(noMatches).toBeVisible();
  await expect
    .poll(() =>
      page.locator(".library-search").evaluate((form) => {
        const noMatchesBox = form.nextElementSibling?.getBoundingClientRect();
        return noMatchesBox
          ? Math.round(noMatchesBox.top - form.getBoundingClientRect().bottom)
          : 0;
      }),
    )
    .toBe(14);
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: "Start recording" }).click();

  await expect(page.getByText("Recording safely to disk")).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();
});

test("shows the configured global recording shortcut beside the recording action", async ({
  page,
}) => {
  await page.goto("/?globalShortcut=command_or_control_shift_r");

  await expect(
    page.getByRole("button", { name: "New recording" }).locator("kbd"),
  ).toHaveText(/(⌘|Ctrl) (⇧|Shift) R/);
});

test("completes first-run setup with a timed source test", async ({ page }) => {
  await page.clock.install();
  await page.goto("/?firstRun=1");

  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).toBeVisible();
  await expect(page.getByText(/Recordings are saved in/)).toBeVisible();

  await page.getByRole("button", { name: "Start 10-second test" }).click();
  await expect(page.getByText(/remaining/)).toBeVisible();

  await page.clock.fastForward(10_500);

  await expect(
    page.getByRole("heading", { name: "Audio test ready" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Finish setup" }).click();

  await expect(page.getByText("Recent sessions")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).not.toBeVisible();
});
