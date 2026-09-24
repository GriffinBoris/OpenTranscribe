import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, test, vi } from "vitest";

import type { LocalModel } from "@/types/domain";
import { useLocalModelsStore } from "./localModelsStore";

vi.mock("@/core/native", () => ({ native: {} }));

beforeEach(() => setActivePinia(createPinia()));

const best: LocalModel = {
  id: "whisper-large-v3-turbo-q5_0",
  preset: "best",
  label: "Best",
  description: "",
  byte_count: 574_041_195,
  installed: true,
};
const nemotron: LocalModel = {
  id: "nemotron-3-diarization",
  preset: "diarization",
  label: "Nemotron 3 speaker recognition",
  description: "",
  byte_count: 400_506_656,
  installed: true,
};

test("keeps the diarizer separate from speech and dictation models", () => {
  const store = useLocalModelsStore();
  store.models = [{ ...best }, { ...nemotron }];
  expect(store.installedModels.map((model) => model.id)).toEqual([best.id]);
  expect(store.diarizationModel?.id).toBe(nemotron.id);
});

test("supports speaker recognition with no speech model installed", () => {
  const store = useLocalModelsStore();
  store.models = [{ ...best, installed: false }, { ...nemotron }];
  expect(store.diarizationModel?.installed).toBe(true);
  expect(store.installedModels).toEqual([]);
});
