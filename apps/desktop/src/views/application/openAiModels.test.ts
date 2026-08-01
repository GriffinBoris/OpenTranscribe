import { describe, expect, test } from "vitest";

import { openAiModelId, openAiModelOptions } from "./openAiModels";

describe("OpenAI transcription models", () => {
  test("uses GPT Transcribe as the recommended default option", () => {
    const options = openAiModelOptions((key) => key);

    expect(options[0]).toMatchObject({
      value: "gpt_transcribe",
      modelId: "gpt-transcribe",
    });
    expect(openAiModelId("gpt_transcribe")).toBe("gpt-transcribe");
  });

  test("keeps standard, speaker, and fast file models explicit", () => {
    expect(openAiModelId("gpt_4o_transcribe")).toBe("gpt-4o-transcribe");
    expect(openAiModelId("gpt_4o_transcribe_diarize")).toBe(
      "gpt-4o-transcribe-diarize",
    );
    expect(openAiModelId("gpt_4o_mini_transcribe")).toBe(
      "gpt-4o-mini-transcribe",
    );
  });
});
