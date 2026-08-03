import { expect, test } from "./fixtures";

test("shows storage and shortcut utilities in settings", async ({ page }) => {
  await page.goto("/settings#storage");

  await expect(page.getByRole("heading", { name: "Storage" })).toBeVisible();
  await expect(page.getByText("~/Documents/OpenTranscribe")).toBeVisible();
  await expect(
    page.locator("#storage").getByRole("button", { name: "Change folder" }),
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
  await expect(
    page.locator("#shortcuts").getByText("Active", { exact: true }),
  ).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:recording"),
    );
  });
  await expect(page).toHaveURL(/\/sessions\/[^/]+$/);
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:recording"),
    );
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

test("captures and saves a custom global recording shortcut", async ({
  page,
}) => {
  await page.goto("/settings#shortcuts");

  const shortcutButton = page.getByRole("button", {
    name: "Global recording shortcut",
  });
  await shortcutButton.click();

  const dialog = page.getByRole("dialog", { name: "Set recording shortcut" });
  await expect(dialog).toContainText("Press a shortcut");
  await page.keyboard.press("Control+Shift+J");
  await expect(dialog).toContainText("J");

  await dialog.getByRole("button", { name: "Save shortcut" }).click();
  await expect(dialog).not.toBeVisible();
  await expect(shortcutButton).toContainText("J");
  await expect(page.getByText("Off", { exact: true })).toBeVisible();
});

test("suspends every global shortcut while capturing a replacement", async ({
  page,
}) => {
  await page.goto("/settings#shortcuts");

  await page.getByRole("switch", { name: "Record from anywhere" }).click();
  await page.getByRole("button", { name: "Global recording shortcut" }).click();
  await expect(
    page.getByRole("dialog", { name: "Set recording shortcut" }),
  ).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:recording"),
    );
  });
  await expect(page).toHaveURL(/\/settings/);

  await page.getByRole("button", { name: "Cancel" }).click();
  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:recording"),
    );
  });
  await expect(page).toHaveURL(/\/sessions\/[^/]+$/);
});

test("keeps Dictation shortcut registration separate from recording", async ({
  page,
}) => {
  await page.goto("/settings#dictation");

  await expect(page.getByRole("heading", { name: "Dictation" })).toBeVisible();
  await expect(
    page.locator("#dictation").getByText("Active", { exact: true }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Shortcuts" }).click();
  await page.getByRole("switch", { name: "Record from anywhere" }).click();

  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:dictation"),
    );
  });
  await expect(page).toHaveURL(/\/settings/);

  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:recording"),
    );
  });
  await expect(page).toHaveURL(/\/sessions\/[^/]+$/);
});

test("surfaces a replacement shortcut registration failure after capture closes", async ({
  page,
}) => {
  await page.goto("/settings#shortcuts");

  await page.getByRole("switch", { name: "Record from anywhere" }).click();
  const shortcutButton = page.getByRole("button", {
    name: "Global recording shortcut",
  });
  await shortcutButton.click();
  await page.keyboard.press("Control+Shift+J");
  await page.getByRole("button", { name: "Save shortcut" }).click();
  await expect(shortcutButton).toContainText("J");

  await page.evaluate(() => {
    window.history.replaceState(
      {},
      "",
      "/settings?globalShortcutError=1#shortcuts",
    );
  });
  await shortcutButton.click();
  await page.keyboard.press("Control+Shift+K");
  await page.getByRole("button", { name: "Save shortcut" }).click();

  await expect(
    page.getByRole("dialog", { name: "Set recording shortcut" }),
  ).not.toBeVisible();
  await expect(shortcutButton).toContainText("K");
  await expect(
    page.locator("#shortcuts").getByText("Unavailable", { exact: true }),
  ).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(
      new Event("opentranscribe:preview-global-shortcut:recording"),
    );
  });
  await expect(page).toHaveURL(/\/settings/);
});

test("keeps an active local-model download from being started twice", async ({
  page,
}) => {
  await page.goto("/settings#models");

  const downloadButton = page
    .getByRole("group", { name: "Fast" })
    .getByRole("button", { name: "Download" });
  await downloadButton.click();

  await expect(downloadButton).toBeDisabled();
  await expect(page.getByTestId("global-model-download")).toContainText(
    "Downloading Fast",
  );
  await expect(page.getByRole("alert")).not.toBeVisible();
});

test("confirms and updates the local model folder", async ({ page }) => {
  await page.goto("/settings#models");

  const models = page.locator("#models");
  await expect(models).toContainText(
    "/Users/you/Documents/OpenTranscribe/models",
  );

  await models.getByRole("button", { name: "Change folder" }).click();

  const dialog = page.getByRole("dialog", { name: "Move local models?" });
  await expect(dialog).toContainText(
    "/Users/you/Documents/OpenTranscribe models",
  );
  await dialog.getByRole("button", { name: "Move models" }).click();

  await expect(dialog).not.toBeVisible();
  await expect(models).toContainText(
    "/Users/you/Documents/OpenTranscribe models",
  );
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

test("resets settings without deleting the library or credentials", async ({
  page,
}) => {
  await page.goto("/settings?resetReady=1#application");

  await page.getByRole("button", { name: "Reset settings…" }).click();

  const dialog = page.getByRole("dialog", { name: "Reset settings?" });
  await expect(dialog).toContainText("recordings, projects, OpenAI API key");
  await expect(dialog).toContainText(
    "Operating-system permission grants are managed separately",
  );

  await dialog.getByRole("button", { name: "Reset settings" }).click();

  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).toBeVisible();
});

test("requires explicit confirmation before deleting all OpenTranscribe data", async ({
  page,
}) => {
  await page.goto("/settings?resetReady=1#application");

  await page.getByRole("button", { name: "Delete all data…" }).click();

  const dialog = page.getByRole("dialog", {
    name: "Delete all OpenTranscribe data?",
  });
  await expect(dialog).toContainText("This cannot be undone.");
  await expect(dialog).toContainText(
    "Other files in that folder are left alone.",
  );
  await expect(
    dialog.getByRole("button", { name: "Delete all data" }),
  ).toBeDisabled();

  await dialog.getByPlaceholder("DELETE").fill("DELETE");
  await dialog.getByRole("button", { name: "Delete all data" }).click();

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
      page
        .locator(".settings-content")
        .evaluate((scrollPane) =>
          Math.abs(
            scrollPane.scrollTop -
              (scrollPane.scrollHeight - scrollPane.clientHeight),
          ),
        ),
    )
    .toBeLessThanOrEqual(1);
  await expect
    .poll(() =>
      page
        .locator(".settings-content")
        .evaluate((scrollPane) =>
          Math.abs(
            scrollPane.scrollHeight -
              (document.getElementById("application")!.offsetTop +
                document.getElementById("application")!.offsetHeight),
          ),
        ),
    )
    .toBeLessThanOrEqual(1);

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

  await expect(
    page.locator("#shortcuts").getByText("Unavailable", { exact: true }),
  ).toBeVisible();
  await expect(
    page
      .locator("#shortcuts")
      .getByText(
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
