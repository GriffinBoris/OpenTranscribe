import type { OpenAiTranscriptionModel } from "@/types/domain";

export interface OpenAiModelOption {
  label: string;
  value: OpenAiTranscriptionModel;
  modelId: string;
}

export function openAiModelOptions(
  translate: (key: string) => string,
): OpenAiModelOption[] {
  return [
    {
      label: translate("openAiModels.accurate"),
      value: "gpt_transcribe",
      modelId: "gpt-transcribe",
    },
    {
      label: translate("openAiModels.speakers"),
      value: "gpt_4o_transcribe_diarize",
      modelId: "gpt-4o-transcribe-diarize",
    },
    {
      label: translate("openAiModels.fast"),
      value: "gpt_4o_mini_transcribe",
      modelId: "gpt-4o-mini-transcribe",
    },
  ];
}

export function openAiModelId(model: OpenAiTranscriptionModel): string {
  if (model === "gpt_4o_transcribe_diarize") {
    return "gpt-4o-transcribe-diarize";
  }

  if (model === "gpt_4o_mini_transcribe") {
    return "gpt-4o-mini-transcribe";
  }

  return "gpt-transcribe";
}
