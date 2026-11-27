// discord_app/modules/markup_v2/NativeMarkdownExperiment.tsx
import LabFeatureStore from "../labs/LabFeatureStore.tsx";

let ApexExperiment = fn(1450);
const apexExperiment = ApexExperiment.createApexExperiment({
  name: "2025-04-native-markdown",
  kind: "user",
  defaultConfig: { enabled: false },
  variations: { 0: { enabled: false }, 1: { enabled: true } },
});
ApexExperiment = fn(1450);
const apexExperiment1 = ApexExperiment.createApexExperiment({
  name: "2026-10-native-markdown-setting",
  kind: "user",
  defaultConfig: { enabled: false },
  variations: { 0: { enabled: false }, 1: { enabled: true } },
});
const enable_native_markdown = "enable_native_markdown";
const size = fn(2);
const result = size.fileFinishedImporting("modules/markup_v2/NativeMarkdownExperiment.tsx");

export const NativeMarkdownExperiment = apexExperiment;
export const NativeMarkdownSettingExperiment = apexExperiment1;
export const NATIVE_MARKDOWN_LAB_FEATURE = "enable_native_markdown";
export function isNativeMarkdownEnabled(location, arg1) {
  const _location = location.location;
  if (apexExperiment1.getConfig({ location: _location }, arg1).enabled) {
    let enabled = LabFeatureStore.get(enable_native_markdown);
  } else {
    const obj = { location: _location };
    enabled = apexExperiment.getConfig(obj, arg1).enabled;
  }
  return enabled;
}
export function useNativeMarkdown(location) {
  const _location = location.location;
  let enabled = apexExperiment.useConfig({ location: _location }).enabled;
  if (apexExperiment1.useConfig({ location: _location }).enabled) {
    enabled = tmp;
  }
  return { enabled };
}
