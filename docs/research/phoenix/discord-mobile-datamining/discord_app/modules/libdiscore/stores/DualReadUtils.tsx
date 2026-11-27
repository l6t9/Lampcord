// discord_app/modules/libdiscore/stores/DualReadUtils.tsx
import LastFewActionsAll from "../../../../discord_common/js/packages/flux/LastFewActions.tsx";
import discord_common_shallowEqual from "../../../../discord_common/js/packages/shallow-equal/shallowEqual.tsx";
import Constants from "../../../Constants.tsx";
import AnalyticsUtilsDefault from "../../../utils/AnalyticsUtils.tsx";
import SetUtils from "../../../../discord_common/js/shared/utils/SetUtils.tsx";
import size from "../../../../_runtime/metro/00002__.js";

function deepEqualImpl(getTime, getTime2, map, arg3) {
  if (getTime === getTime2) {
    return true;
  } else {
    if (arg3) {
      if (undefined === getTime) {
        if (null === getTime2) {
          return true;
        }
      }
    }
    if (typeof getTime === "object") {
      if (typeof getTime2 === "object") {
        if (null !== getTime) {
          if (null !== getTime2) {
            if (map.has(getTime)) {
              return map.get(getTime) === getTime2;
            } else {
              const result = map.set(getTime, getTime2);
              const _Date = Date;
              if (getTime instanceof Date) {
                const _Date2 = Date;
                if (getTime2 instanceof Date) {
                  const time = getTime.getTime();
                  return time === getTime2.getTime();
                }
              }
              const _Set = Set;
              if (getTime instanceof Set) {
                const _Set2 = Set;
                if (getTime2 instanceof Set) {
                  return SetUtils.areSetsEqual(getTime, getTime2);
                }
              }
              const _Array = Array;
              if (Array.isArray(getTime)) {
                const _Array2 = Array;
                if (Array.isArray(getTime2)) {
                  if (getTime.length !== getTime2.length) {
                    return false;
                  } else {
                    let num4 = 0;
                    if (0 < getTime.length) {
                      while (deepEqualImpl(getTime[num4], getTime2[num4], map, false)) {
                        num4 = num4 + 1;
                      }
                      return false;
                    }
                    return true;
                  }
                }
              }
              const _Array3 = Array;
              if (!Array.isArray(getTime)) {
                const _Array4 = Array;
                if (!Array.isArray(getTime2)) {
                  const _Object = Object;
                  const keys = Object.keys(getTime);
                  const _Object2 = Object;
                  if (keys.length !== Object.keys(getTime2).length) {
                    return false;
                  } else {
                    const iter = keys[Symbol.iterator]();
                    const nextResult = iter.next();
                    while (iter !== undefined) {
                      let tmp7 = nextResult;
                      let _Object3 = Object;
                      hasOwnProperty = Object.prototype.hasOwnProperty;
                      let call = hasOwnProperty.call;
                      if (typeof call === "unknown" ? hasOwnProperty(nextResult) : call(getTime2, nextResult)) {
                        let flag = false;
                        if (deepEqualImpl(getTime[tmp7], getTime2[tmp7], map, false)) {
                          continue;
                        } else {
                          iter.return();
                          return false;
                        }
                      } else {
                        iter.return();
                        return false;
                      }
                    }
                    return true;
                  }
                }
              }
              return false;
            }
          }
        }
      }
    }
    return false;
  }
}
function deepEqual(getTime, getTime2) {
  return deepEqualImpl(getTime, getTime2, new Map(), true);
}
function doDualReadValidation(items, derived, derived2) {
  if (derived.derived.length !== derived2.derived.length) {
    const obj = {
      type: "length-mismatch",
      primaryLength: derived.derived.length,
      shadowLength: derived2.derived.length,
    };
    items.push(obj);
  }
  const keys = Object.keys(derived.root);
  const keys1 = Object.keys(derived2.root);
  const iter = keys[Symbol.iterator]();
  const nextResult = iter.next();
  while (iter !== undefined) {
    let tmp5 = nextResult;
    let _Object = Object;
    hasOwnProperty = Object.prototype.hasOwnProperty;
    let call = hasOwnProperty.call;
    let root = derived2.root;
    if (typeof call === "unknown" ? hasOwnProperty(nextResult) : call(root, nextResult)) {
      let tmp10 = validateRecord(tmp5, derived.root[tmp5], derived2.root[tmp5]);
      if (null != tmp10) {
        let arr4 = items.push(tmp11);
      }
    } else {
      let obj2 = { type: "missing-record", key: null };
      obj2.key = tmp5;
      let arr5 = items.push(obj2);
    }
    continue;
  }
  for (const item10060 of keys1) {
    let _Object2 = Object;
    let call2 = hasOwnProperty2.call;
    let root2 = arg1.root;
    if (typeof call2 === "unknown") {
      let hasOwnProperty2Result = hasOwnProperty2(item10060);
    } else {
      hasOwnProperty2Result = call2(root2, item10060);
    }
    if (!hasOwnProperty2Result) {
      let obj3 = { type: "extra-record", key: item10060 };
      let arr6 = arg0.push(obj3);
    }
    continue;
  }
}
function validateRecord(key, primaryRecord, shadowRecord) {
  items = [];
  for (const key10007 in arg1) {
    let _Object2 = Object;
    let call2 = hasOwnProperty2.call;
    if (typeof call2 === "unknown") {
      let hasOwnProperty2Result = hasOwnProperty2(key10007);
    } else {
      hasOwnProperty2Result = call2(arg1, key10007);
    }
    if (!hasOwnProperty2Result) {
      continue;
    } else {
      let tmp2 = arg1[key10007];
      let _Object = Object;
      hasOwnProperty = Object.prototype.hasOwnProperty;
      let call = hasOwnProperty.call;
      if (typeof call === "unknown") {
        let hasOwnPropertyResult = hasOwnProperty(key10007);
      } else {
        hasOwnPropertyResult = call(arg2, key10007);
      }
      if (hasOwnPropertyResult) {
        let tmp5 = arg2[key10007];
        let _Map = Map;
        let tmp7 = new.target;
        let tmp8 = new.target;
        map = new Map();
        let flag = true;
        if (deepEqualImpl(tmp2, tmp5, map, true)) {
          continue;
        } else {
          let obj2 = { type: "value-mismatch", field: key10007, primaryValue: tmp2, shadowValue: tmp5 };
          let arr = items.push(obj2);
          continue;
        }
        continue;
      } else {
        let obj = { type: "field-missing", field: key10007 };
        let arr3 = items.push(obj);
        continue;
      }
      continue;
    }
    continue;
  }
  if (items.length > 0) {
    const obj3 = { type: "record-mismatch", key, primaryRecord, shadowRecord, mismatches: items };
    return obj3;
  }
}
function isPlainObject(obj) {
  let isArray = typeof obj !== "object";
  if (typeof obj === "object") {
    isArray = null === obj;
  }
  if (!isArray) {
    const _Array = Array;
    isArray = Array.isArray(obj);
  }
  if (!isArray) {
    const _Date = Date;
    isArray = obj instanceof Date;
  }
  if (!isArray) {
    const _Set = Set;
    isArray = obj instanceof Set;
  }
  return !isArray;
}
function logErrorsToAnalytics(store_name, items) {
  if (0 !== items.length) {
    const lastResult = LastFewActionsAll.last();
    if (null != lastResult) {
      value = map.get(store_name);
      if (value == null) {
        let obj = { mismatchesReported: 0, mismatchesByLastAction: null, visitedEntries: null, seenMismatches: null };
        const _Map = Map;
        map = new Map();
        obj.mismatchesByLastAction = map;
        const _WeakSet = WeakSet;
        const weakSet = new WeakSet();
        obj.visitedEntries = weakSet;
        let _Set = Set;
        let set = new Set();
        obj.seenMismatches = set;
        value = obj;
      }
      let result = map.set(store_name, value);
      if (value.mismatchesReported < 15) {
        const mismatchesByLastAction = value.mismatchesByLastAction;
        let num2 = mismatchesByLastAction.get(lastResult);
        if (num2 == null) {
          num2 = 0;
        }
        if (num2 < 3) {
          let tmp21 = (function generateErrorReport(value, items) {
            function appendMismatch() {
              items = [...arguments];
              const iter = items[Symbol.iterator]();
              const nextResult = iter.next();
              while (iter !== undefined) {
                let _HermesInternal = HermesInternal;
                let str = "";
                let str2 = ":";
                let str3 = ":";
                let tmp2 = nextResult;
                let combined = "" + nextResult.fieldName + ":" + nextResult.primaryType + ":" + nextResult.shadowType;
                let seenMismatches = value.seenMismatches;
                let tmp4 = combined;
                if (!seenMismatches.has(combined)) {
                  let seenMismatches2 = value.seenMismatches;
                  let addResult = seenMismatches2.add(tmp4);
                  let mismatchedFields = obj.mismatchedFields;
                  let arr = mismatchedFields.push(tmp2);
                }
                continue;
              }
            }
            function appendDeepMismatches(combined, primaryValue, shadowValue) {
              if (set === undefined) {
                const _Set = Set;
                set = new Set();
              }
              const keys = Object.keys(primaryValue);
              const iter = keys[Symbol.iterator]();
              const nextResult = iter.next();
              while (iter !== undefined) {
                let _HermesInternal = HermesInternal;
                let tmp6 = nextResult;
                combined = "" + combined + "." + nextResult;
                let tmp8 = primaryValue[nextResult];
                let _Object = Object;
                hasOwnProperty = Object.prototype.hasOwnProperty;
                let call = hasOwnProperty.call;
                if (typeof call === "unknown" ? hasOwnProperty(nextResult) : call(shadowValue, nextResult)) {
                  let tmp15 = shadowValue[tmp6];
                  let tmp16 = tmp15;
                  if (tmp8 !== tmp15) {
                    if (typeof tmp8 === "object") {
                      if (typeof tmp16 === "object") {
                        let _Array = Array;
                        if (Array.isArray(tmp8)) {
                          let _Array2 = Array;
                          if (Array.isArray(tmp16)) {
                            let tmp43 = appendArrayMismatches(combined, tmp8, tmp16);
                          }
                        }
                        let tmp28 = null != tmp8;
                        if (tmp28) {
                          tmp28 = null != tmp16;
                        }
                        if (tmp28) {
                          if (!set.has(tmp8)) {
                            let addResult = set.add(tmp8);
                            let tmp38 = appendDeepMismatches(combined, tmp8, tmp16, set);
                          }
                        }
                      }
                    }
                    obj = { fieldName: null, primaryType: null, shadowType: null };
                    obj.fieldName = combined;
                    obj.primaryType = getType(tmp8);
                    obj.shadowType = getType(tmp16);
                    let tmp24 = appendMismatch(obj);
                  }
                } else {
                  let obj2 = { fieldName: null, primaryType: null, shadowType: "missing" };
                  obj2.fieldName = combined;
                  obj2.primaryType = getType(tmp8);
                  let tmp13 = appendMismatch(obj2);
                }
                continue;
              }
            }
            function appendArrayMismatches(combined, primaryValue, shadowValue) {
              if (!obj.areArraysShallowEqual(primaryValue, shadowValue)) {
                const obj2 = {
                  fieldName: combined,
                  primaryType: "array",
                  shadowType: "array",
                  primaryArrayLength: primaryValue.length,
                  secondaryArrayLength: shadowValue.length,
                };
                appendMismatch(obj2);
              }
              obj = discord_common_shallowEqual;
            }
            let obj = { numExtraKeys: 0, numMissingKeys: 0, mismatchedFields: [] };
            let iter = items[Symbol.iterator]();
            let nextResult = iter.next();
            while (iter !== undefined) {
              let tmp2 = nextResult;
              let type = nextResult.type;
              if ("extra-record" === type) {
                obj.numExtraKeys = obj.numExtraKeys + 1;
              } else if ("missing-record" === type) {
                obj.numMissingKeys = obj.numMissingKeys + 1;
              } else {
                if ("record-mismatch" === type) {
                  let visitedEntries2 = value.visitedEntries;
                  if (visitedEntries2.has(tmp2.primaryRecord)) {
                    continue;
                  } else {
                    let visitedEntries = value.visitedEntries;
                    let addResult = visitedEntries.add(tmp2.primaryRecord);
                    let mismatches = tmp2.mismatches;
                    for (const item10034 of mismatches) {
                      let str = item10034.field;
                      let str1 = str.toString();
                      let type2 = item10034.type;
                      if ("field-missing" === type2) {
                        let mismatchedFields = obj.mismatchedFields;
                        let obj2 = { fieldName: null, primaryType: null, shadowType: "missing" };
                        obj2.fieldName = str1;
                        obj2.primaryType = closure_12(tmp2.primaryRecord[item10034.field]);
                        let arr = mismatchedFields.push(obj2);
                      } else if ("value-mismatch" === type2) {
                        if (null !== item10034.primaryValue) {
                          if (null !== item10034.shadowValue) {
                            if (typeof item10034.primaryValue === "object") {
                              if (typeof item10034.shadowValue === "object") {
                                let _Array = Array;
                                if (Array.isArray(item10034.primaryValue)) {
                                  let _Array2 = Array;
                                  if (Array.isArray(item10034.shadowValue)) {
                                    let result = appendArrayMismatches(
                                      str1,
                                      item10034.primaryValue,
                                      item10034.shadowValue,
                                    );
                                  }
                                }
                                let appendDeepMismatchesResult = appendDeepMismatches(
                                  str1,
                                  item10034.primaryValue,
                                  item10034.shadowValue,
                                );
                              }
                            }
                          }
                        }
                        let obj3 = { fieldName: null, primaryType: null, shadowType: null };
                        obj3.fieldName = str1;
                        obj3.primaryType = closure_12(item10034.primaryValue);
                        obj3.shadowType = closure_12(item10034.shadowValue);
                        let appendMismatchResult = appendMismatch(obj3);
                      }
                      continue;
                    }
                  }
                }
                continue;
              }
              continue;
            }
            if (obj.mismatchedFields.length + obj.numExtraKeys + obj.numMissingKeys === 0) {
              return null;
            } else {
              return obj;
            }
          })(value, items);
          if (null != tmp21) {
            const mismatchesByLastAction2 = value.mismatchesByLastAction;
            const result1 = mismatchesByLastAction2.set(lastResult, num2 + 1);
            value.mismatchesReported = value.mismatchesReported + 1;
            const obj6 = {
              store_name,
              action_type: lastResult,
              num_missing_keys: null,
              num_extra_keys: null,
              mismatched_fields: null,
            };
            ({ numMissingKeys: obj5.num_missing_keys, numExtraKeys: obj5.num_extra_keys } = tmp21);
            const _JSON = JSON;
            obj6.mismatched_fields = JSON.stringify(tmp21.mismatchedFields);
            AnalyticsUtilsDefault.track(AnalyticEvents.LIBDISCORE_KV_DUAL_READ_ERROR, obj6);
          }
        }
      }
    }
  }
}
function getType(obj) {
  if (null === obj) {
    return "null";
  } else if (typeof obj === "object") {
    const _Array = Array;
    let str = "object";
    if (Array.isArray(obj)) {
      str = "array";
    }
  } else {
    const tmp2 = typeof obj;
  }
}
const AnalyticEvents = Constants.AnalyticEvents;
let map = new Map();
let result = size.fileFinishedImporting("modules/libdiscore/stores/DualReadUtils.tsx");

export function runDualReadValidation(store_name, Kkv, fn) {
  items = [];
  fn((derived, derived2) => {
    doDualReadValidation(items, derived, derived2);
  });
  closure_129_0 = undefined;
  closure_129_1 = undefined;
  closure_129_2 = undefined;
  if (0 !== items.length) {
    let items1 = [];
    closure_129_0 = items1;
    let items2 = [];
    closure_129_1 = items2;
    closure_129_2 = 0;
    let item = items.forEach((type) => {
      if (closure_2 < 5) {
        closure_2 = tmp + 1;
        type = type.type;
        if ("length-mismatch" !== type) {
          if ("missing-record" === type) {
            items.push(type.key);
          } else if ("extra-record" === type) {
            importDefault.push(type.key);
          } else if ("record-mismatch" === type) {
            const mismatches = type.mismatches;
            const item = mismatches.forEach((type) => {
              if ("field-missing" !== type.type) {
                function impl(primaryValue, shadowValue) {
                  if (!closure_2_6(primaryValue, shadowValue)) {
                    if (typeof primaryValue === "object") {
                      if (null !== primaryValue) {
                        if (typeof shadowValue === "object") {
                          if (null !== shadowValue) {
                            if (map.has(primaryValue)) {
                              value = map.get(primaryValue);
                            } else {
                              const result = map.set(primaryValue, shadowValue);
                            }
                          }
                        }
                      }
                    }
                    const _Date = Date;
                    if (!(primaryValue instanceof Date)) {
                      const _Set = Set;
                      if (primaryValue instanceof Set) {
                        const _Set2 = Set;
                        if (shadowValue instanceof Set) {
                          items = [];
                          HermesBuiltin.arraySpread(primaryValue, 0);
                          const items1 = [];
                          const found = items.filter((item) => !shadowValue.has(item));
                          HermesBuiltin.arraySpread(shadowValue, 0);
                          items1.filter((item) => !primaryValue.has(item)).length;
                        }
                      }
                      const _Array = Array;
                      if (Array.isArray(primaryValue)) {
                        const _Array2 = Array;
                        if (Array.isArray(shadowValue)) {
                          const _Math = Math;
                          const bound = Math.min(primaryValue.length, shadowValue.length);
                          for (let num3 = 0; num3 < bound; num3 = num3 + 1) {
                            if (!closure_2_6(primaryValue[num3], shadowValue[num3])) {
                              let tmp35 = impl(primaryValue[num3], shadowValue[num3]);
                            }
                          }
                        }
                      }
                      if (closure_2_9(primaryValue)) {
                        if (closure_2_9(shadowValue)) {
                          const _Set3 = Set;
                          const _Object = Object;
                          const set = new Set(Object.keys(primaryValue));
                          const _Set4 = Set;
                          const _Object2 = Object;
                          const set1 = new Set(Object.keys(shadowValue));
                          const _Set5 = Set;
                          const items2 = [];
                          HermesBuiltin.arraySpread(set1, HermesBuiltin.arraySpread(set, 0));
                          const set2 = new Set(items2);
                          const _Array3 = Array;
                          const sorted = Array.from(set2).sort();
                          for (const item10068 of sorted) {
                            let hasItem = set.has(item10068);
                            if (hasItem) {
                              hasItem = set1.has(item10068);
                            }
                            if (hasItem) {
                              if (!closure_2_6(arg0[item10068], arg1[item10068])) {
                                let tmp30 = impl(arg0[item10068], arg1[item10068]);
                              }
                            }
                            continue;
                          }
                          const arr = Array.from(set2);
                        }
                      }
                    } else {
                      const _Date2 = Date;
                    }
                  }
                }
                const _Map = Map;
                ({ primaryValue, shadowValue } = type);
                map = new Map();
                impl(primaryValue, shadowValue);
              }
            });
          }
        }
      }
    });
  }
  logErrorsToAnalytics(store_name, items);
}
export { doDualReadValidation };
export { logErrorsToAnalytics };
