// discord_app/modules/user_settings/defs/native/PhoenixLaunchSetting.tsx
import util from "../../../../intl/index.native.tsx";
import MobilePhoneIcon from "../../../../design/components/Icon/native/redesign/generated/MobilePhoneIcon.tsx";
import useIsStaffOrDeveloperSettingPredicate from "../../dev_tools/native/useIsStaffOrDeveloperSettingPredicate.tsx";
import PhoenixSwitchEntrypointExperimentDefault from "../../phoenix_launch/native/PhoenixSwitchEntrypointExperiment.tsx";
import PhoenixRolloutExperimentDefault from "../../phoenix_launch/native/PhoenixRolloutExperiment.tsx";
import PhoenixLaunchState from "../../phoenix_launch/native/PhoenixLaunchState.tsx";
import SettingBuilders from "../../../settings/native/renderer/SettingBuilders.tsx";
import size from "../../../../../_runtime/metro/00002__.js";

const toggle = SettingBuilders.createToggle({
  useTitle() {
    const intl = util.intl;
    return intl.string(util.t.paDGCW);
  },
  useDescription() {
    const intl = util.intl;
    return intl.string(util.t.DXxeZL);
  },
  parent: null,
  IconComponent: MobilePhoneIcon.MobilePhoneIcon,
  useValue: PhoenixLaunchState.useShouldLaunchPhoenix,
  onValueChange: PhoenixLaunchState.setShouldLaunchPhoenix,
  usePredicate: function usePhoenixLaunchPredicate() {
    let enabled = useIsStaffOrDeveloperSettingPredicate.useStaffOrDeveloperSettingPredicate();
    const obj2 = PhoenixSwitchEntrypointExperimentDefault;
    const tmp =
      null !=
      PhoenixRolloutExperimentDefault.useConfig({ location: "PhoenixLaunchSetting" }, { autoTrackExposure: false })
        .forcedClient;
    let tmp3 = !tmp;
    const isPhoenixInstalled = PhoenixLaunchState.useIsPhoenixInstalled();
    if (!tmp) {
      if (!enabled) {
        enabled = obj2.useConfig({ location: "PhoenixLaunchSetting" }, { autoTrackExposure: false }).enabled;
      }
      tmp3 = enabled;
    }
    if (tmp3) {
      tmp3 = isPhoenixInstalled;
    }
    return tmp3;
  },
});
const result = size.fileFinishedImporting("modules/user_settings/defs/native/PhoenixLaunchSetting.tsx");

export default toggle;
