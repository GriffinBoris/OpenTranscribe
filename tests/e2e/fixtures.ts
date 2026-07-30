import { expect, test as base } from "@playwright/test";

export const test = base.extend<{ browserErrors: string[] }>({
  browserErrors: [
    async ({ page }, use) => {
      const browserErrors: string[] = [];

      page.on("pageerror", (error) => {
        browserErrors.push(error.message);
      });
      page.on("console", (message) => {
        if (message.type() === "error") {
          browserErrors.push(message.text());
        }
      });

      await use(browserErrors);

      expect(browserErrors, browserErrors.join("\n\n")).toEqual([]);
    },
    { auto: true },
  ],
});

export { expect };
