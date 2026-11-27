// discord_app/modules/user_settings/phoenix_launch/native/PhoenixRolloutExperiment.tsx
import LaunchClient from "LaunchClient.tsx";
import ApexExperiment from "../../../experiments/apex/index.tsx";
import size from "../../../../../_runtime/metro/00002__.js";

const obj = { name: "2026-11-phoenix-rollout", kind: "user", defaultConfig: { forcedClient: null }, variations: null };
const obj2 = { 0: { forcedClient: null }, 1: { forcedClient: LaunchClient.LaunchClient.PHOENIX }, 2: null };
const obj3 = { forcedClient: LaunchClient.LaunchClient.PHOENIX };
obj2[2] = { forcedClient: LaunchClient.LaunchClient.LEGACY };
obj.variations = obj2;
const apexExperiment = ApexExperiment.createApexExperiment(obj);
const result = size.fileFinishedImporting("modules/user_settings/phoenix_launch/native/PhoenixRolloutExperiment.tsx");

export default apexExperiment;
