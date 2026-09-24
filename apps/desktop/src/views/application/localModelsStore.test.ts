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
  required_model_id: null,
};
const nemotron: LocalModel = {
  id: "whisper-large-v3-turbo-nemotron-3",
  preset: "diarization",
  label: "Whisper + Nemotron 3",
  description: "",
  byte_count: 974_547_851,
  installed: true,
  required_model_id: best.id,
};

test("offers a ready diarization profile for meetings and only Whisper for dictation", () => {
  const store = useLocalModelsStore();
  store.models = [{ ...best }, { ...nemotron }];
  expect(store.installedModels.map((model) => model.id)).toEqual([
    best.id,
    nemotron.id,
  ]);
  expect(store.dictationModels.map((model) => model.id)).toEqual([best.id]);
});

test("keeps Nemotron removable but unavailable until its shared speech model is restored", () => {
  const store = useLocalModelsStore();
  store.models = [{ ...best, installed: false }, { ...nemotron }];
  expect(store.models[1]!.installed).toBe(true);
  expect(store.isReady(store.models[1]!)).toBe(false);
  expect(store.installedModels).toEqual([]);
  store.models[0]!.installed = true;
  expect(store.isReady(store.models[1]!)).toBe(true);
});
