import { expect, test } from "./fixtures";

test("shows storage and shortcut utilities in settings", async ({ page }) => {
  await page.goto("/settings#storage");

  await expect(page.getByRole("heading", { name: "Storage" })).toBeVisible();
  await expect(page.getByText("~/Documents/OpenTranscribe")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Change folder" }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Shortcuts" }).click();

  await expect(page.getByRole("heading", { name: "Shortcuts" })).toBeVisible();
  await expect(
    page
      .locator("#shortcuts .section-heading")
      .getByText(
        "Control recording from anywhere and review shortcuts available in the app.",
      ),
  ).toBeVisible();
  await expect(page.getByText("Import media")).toBeVisible();
  await expect(page.getByText("⌘/Ctrl O")).toBeVisible();

  const globalRecordingToggle = page.getByRole("switch", {
    name: "Record from anywhere",
  });
  await globalRecordingToggle.focus();
  await page.keyboard.press("Space");
  await expect(page.getByLabel("Global recording shortcut")).toBeVisible();
  await expect(page.getByText("Active", { exact: true })).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(new Event("opentranscribe:preview-global-shortcut"));
  });
  await expect(page).toHaveURL(/\/sessions\/[^/]+$/);
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(new Event("opentranscribe:preview-global-shortcut"));
  });
  await expect(page.getByRole("button", { name: "Stop" })).not.toBeVisible();
});

test("keeps local model download progress visible across routes", async ({
  page,
}) => {
  await page.goto("/settings#models");

  await page
    .getByRole("group", { name: "Fast" })
    .getByRole("button", { name: "Download" })
    .click();

  const downloadStatus = page.getByTestId("global-model-download");
  await expect(downloadStatus).toContainText("Downloading Fast");
  await expect(
    downloadStatus.getByRole("progressbar", { name: "Downloading Fast" }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Home", exact: true }).click();

  await expect(page).toHaveURL(/\/$/);
  await expect(downloadStatus).toBeVisible();
  await expect(downloadStatus).not.toBeVisible();
});

test("reopens recording setup from application settings", async ({ page }) => {
  await page.goto("/settings?resetReady=1#application");

  await page.getByRole("button", { name: "Run setup" }).click();

  await expect(page).toHaveURL(/\/$/);
  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).toBeVisible();
  await expect(page.getByText("Audio sources")).toBeVisible();
  await expect(page.getByText("Audio test")).toBeVisible();
});

test("resets app-owned data without presenting the library for deletion", async ({
  page,
}) => {
  await page.goto("/settings?resetReady=1#application");

  await page.getByRole("button", { name: "Reset…" }).click();

  const dialog = page.getByRole("dialog", { name: "Reset OpenTranscribe?" });
  await expect(dialog).toContainText("Your library files will not be deleted.");
  await expect(dialog).toContainText(
    "Operating-system permission grants are managed separately",
  );

  await dialog.getByRole("button", { name: "Reset OpenTranscribe" }).click();

  await expect(page).toHaveURL(/firstRun=1/);
  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).toBeVisible();
});

test("uses quieter one-pixel dividers in settings", async ({ page }) => {
  await page.goto("/settings");

  const divider = page.locator(".setting-field").first();
  const control = page.locator(".app-input").first();
  await expect(divider).toBeVisible();
  await expect(control).toBeVisible();

  const [dividerStyle, controlStyle] = await Promise.all([
    divider.evaluate((element) => {
      const style = getComputedStyle(element);
      return { color: style.borderTopColor, width: style.borderTopWidth };
    }),
    control.evaluate((element) => {
      const style = getComputedStyle(element);
      return { color: style.borderTopColor, width: style.borderTopWidth };
    }),
  ]);

  expect(dividerStyle.width).toBe(controlStyle.width);
  expect(dividerStyle.width).toBe("1px");
  expect(dividerStyle.color).not.toBe(controlStyle.color);
});

test("keeps settings navigation within the natural scroll range", async ({
  page,
}) => {
  await page.goto("/settings");

  for (const [name, id] of [
    ["Storage", "storage"],
    ["Local models", "models"],
    ["OpenAI", "openai"],
    ["Shortcuts", "shortcuts"],
  ]) {
    await page.getByRole("button", { name, exact: true }).click();

    await expect
      .poll(() =>
        page.locator(`#${id}`).evaluate((section) => {
          const scrollPane = document.querySelector(".settings-content");
          if (!scrollPane) {
            return 0;
          }

          return Math.round(
            section.getBoundingClientRect().top -
              scrollPane.getBoundingClientRect().top,
          );
        }),
      )
      .toBe(20);
    await expect(page.locator(".settings-content-frame")).toHaveClass(
      /settings-content-frame--scrolled/,
    );
  }

  await page.getByRole("button", { name: "Application", exact: true }).click();
  await expect
    .poll(() =>
      page.locator(".settings-content").evaluate((scrollPane) => ({
        atBottom:
          Math.round(scrollPane.scrollTop) ===
          Math.round(scrollPane.scrollHeight - scrollPane.clientHeight),
        trailingSpace: Math.round(
          scrollPane.scrollHeight -
            (document.getElementById("application")!.offsetTop +
              document.getElementById("application")!.offsetHeight),
        ),
      })),
    )
    .toEqual({
      atBottom: true,
      trailingSpace: 0,
    });

  await page.getByRole("button", { name: "Recording", exact: true }).click();

  await expect
    .poll(() =>
      page
        .locator(".settings-content")
        .evaluate((scrollPane) => scrollPane.scrollTop),
    )
    .toBe(0);
  await expect(page.locator(".settings-content-frame")).not.toHaveClass(
    /settings-content-frame--scrolled/,
  );
});

test("keeps route scrolling inside each workspace", async ({ page }) => {
  await page.goto("/settings");

  await expect(page.locator(".application-main")).toHaveCSS(
    "overflow-y",
    "hidden",
  );
  await expect(page.locator(".settings-nav")).toHaveCSS("overflow-y", "auto");
  await expect(page.locator(".settings-content")).toHaveCSS(
    "overflow-y",
    "auto",
  );
  await expect
    .poll(() =>
      page.locator(".settings-content-frame").evaluate((element) => {
        const style = getComputedStyle(element, "::before");
        return {
          backdropFilter: style.backdropFilter,
          hasGradientMask: style.maskImage.includes("linear-gradient"),
        };
      }),
    )
    .toEqual({
      backdropFilter: "blur(4px)",
      hasGradientMask: true,
    });

  for (const path of ["/inbox", "/projects/01KDEMOPROJECT", "/processing"]) {
    await page.goto(path);
    const scrollFrame = page.locator(
      path === "/processing"
        ? ".processing-page__scroll"
        : ".library-page__scroll",
    );
    await expect(scrollFrame).toHaveCSS("overflow-y", "auto");
  }
});

test("surfaces a global shortcut registration conflict", async ({ page }) => {
  await page.goto("/settings?globalShortcutError=1#shortcuts");

  const globalRecordingToggle = page.getByRole("switch", {
    name: "Record from anywhere",
  });
  await globalRecordingToggle.focus();
  await page.keyboard.press("Space");

  await expect(page.getByText("Unavailable", { exact: true })).toBeVisible();
  await expect(
    page.getByText(
      "This shortcut could not be registered: This shortcut is already in use.",
    ),
  ).toBeVisible();
});

test("shows macOS system-audio permission status and recovery accurately", async ({
  page,
}) => {
  await page.goto("/settings?systemAudioPermission=granted");

  await expect(page.getByText("System audio ready")).toBeVisible();
  await expect(
    page.getByText(
      "Screen & System Audio Recording permission is granted for this build.",
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open System Settings" }),
  ).not.toBeVisible();

  await page.goto("/settings?systemAudioPermission=required");

  await expect(page.getByText("Permission required")).toBeVisible();
  await expect(
    page.getByText(
      "Allow OpenTranscribe under Screen & System Audio Recording, then relaunch it.",
    ),
  ).toBeVisible();
  await page.getByRole("button", { name: "Open System Settings" }).click();
  await expect(page.locator(".shell-state--error")).not.toBeVisible();

  await page.evaluate(() => {
    window.history.replaceState(
      {},
      "",
      "/settings?systemAudioPermission=granted",
    );
    window.dispatchEvent(new Event("focus"));
  });

  await expect(page.getByText("System audio ready")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open System Settings" }),
  ).not.toBeVisible();
});

test("enables system output when macOS capture is ready", async ({ page }) => {
  await page.goto("/settings?systemAudioPermission=granted");

  const systemOutput = page.getByRole("switch", {
    name: "Capture system output",
  });
  await expect(systemOutput).toBeEnabled();
  await expect(systemOutput).not.toBeChecked();

  await systemOutput.click();

  await expect(systemOutput).toBeChecked();
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await expect(page.getByText("System output · On")).toBeVisible();
});

test("keeps inline form controls on the medium design-token tier", async ({
  page,
}) => {
  await page.goto("/settings#openai");

  const apiKeyForm = page.locator(".openai-key-form");
  const input = apiKeyForm.locator(".app-input");
  const button = apiKeyForm.getByRole("button", { name: "Save to keychain" });
  const mediumControlHeight = await page.evaluate(() =>
    getComputedStyle(document.documentElement)
      .getPropertyValue("--control-height-medium")
      .trim(),
  );

  expect(mediumControlHeight).toBe("38px");
  await expect(input).toHaveCSS("min-height", mediumControlHeight);
  await expect(button).toHaveCSS("min-height", mediumControlHeight);
  await expect
    .poll(async () => {
      const [inputBox, buttonBox] = await Promise.all([
        input.boundingBox(),
        button.boundingBox(),
      ]);
      return [inputBox?.height, buttonBox?.height];
    })
    .toEqual([
      Number.parseFloat(mediumControlHeight),
      Number.parseFloat(mediumControlHeight),
    ]);
});
