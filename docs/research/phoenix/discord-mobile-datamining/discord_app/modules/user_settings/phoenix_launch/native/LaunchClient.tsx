// discord_app/modules/user_settings/phoenix_launch/native/LaunchClient.tsx
import size from "../../../../../_runtime/metro/00002__.js";

const LaunchClient = { PHOENIX: "phoenix", LEGACY: "legacy" };
const result = size.fileFinishedImporting("modules/user_settings/phoenix_launch/native/LaunchClient.tsx");

export { LaunchClient };
export function toLaunchClient(client) {
  return client === obj.PHOENIX ? obj.PHOENIX : obj.LEGACY;
}
