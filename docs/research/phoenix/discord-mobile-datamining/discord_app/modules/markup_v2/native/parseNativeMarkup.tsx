// discord_app/modules/markup_v2/native/parseNativeMarkup.tsx
import Syntax from "../../../../_runtime/08813_Syntax.js";
import transformNativeMarkupNode from "transformNativeMarkupNode.tsx";
import apply from "../../../../_runtime/00012_apply.js";

require = fn;
let closure_2 = apply.once(() => Syntax.parse);
const size = fn(2);
let result = size.fileFinishedImporting("modules/markup_v2/native/parseNativeMarkup.tsx");

export default function parseNativeMarkupToAST(arg0, arg1, channelId) {
  let tmp = arg3;
  if (arg3 === undefined) {
    tmp = null;
  }
  const result = transformNativeMarkupNode.transformNativeBlocks(closure_2()(arg0), channelId);
  let tmpResult = result;
  if (null != tmp) {
    tmpResult = tmp(result, arg1, false);
  }
  return tmpResult;
}
