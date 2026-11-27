// discord_app/modules/libdiscore/FluxApi.tsx
import shim_mod from "../../../discord_common/js/packages/libdiscore/js_shim/js/shim.native.tsx";
import size from "../../../_runtime/metro/00002__.js";

let shim = shim_mod;
shim = shim.getFluxApi();
const result = size.fileFinishedImporting("modules/libdiscore/FluxApi.tsx");

export const FLUX_API = shim;
export function hasFluxApi() {
  return null != shim;
}
