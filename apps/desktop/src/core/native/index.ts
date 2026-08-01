import { desktopNative } from "@/core/native/desktopNative";
import { previewNative } from "@/core/native/previewNative";

export const native =
  "__TAURI_INTERNALS__" in window ? desktopNative : previewNative;
