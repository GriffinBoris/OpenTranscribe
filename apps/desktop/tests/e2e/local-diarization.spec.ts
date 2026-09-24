import { expect, test } from "./fixtures";

test("installs shared speech weights and repairs a removed dependency", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 640 });
  await page.goto("/settings#models");
  const profile = page.getByRole("group", {
    name: "Whisper + Nemotron 3",
    exact: true,
  });
  const best = page.getByRole("group", { name: "Best", exact: true });

  await profile.getByRole("button", { name: "Download", exact: true }).click();
  await expect(profile.getByText("Installed", { exact: true })).toBeVisible();
  await expect(best.getByText("Installed", { exact: true })).toBeVisible();
  await best.getByRole("button", { name: "Remove", exact: true }).click();
  await expect(
    profile.getByText("Whisper Best required", { exact: true }),
  ).toBeVisible();
  await expect(
    profile.getByRole("button", { name: "Remove", exact: true }),
  ).toBeVisible();
  await profile.getByRole("button", { name: "Download", exact: true }).click();
  await expect(profile.getByText("Installed", { exact: true })).toBeVisible();
  await expect(best.getByText("Installed", { exact: true })).toBeVisible();
  await profile.getByRole("button", { name: "Remove", exact: true }).click();
  await expect(
    profile.getByText("Not installed", { exact: true }),
  ).toBeVisible();
  await expect(best.getByText("Installed", { exact: true })).toBeVisible();
});
