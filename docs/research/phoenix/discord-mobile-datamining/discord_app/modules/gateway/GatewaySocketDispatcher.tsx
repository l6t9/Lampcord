// discord_app/modules/gateway/GatewaySocketDispatcher.tsx
import LoggerDefault from "../debug/Logger.tsx";
import TimeUtils from "../../../discord_common/js/packages/time-utils/TimeUtils.tsx";
import WorkSchedulerTelemetry from "WorkSchedulerTelemetry.tsx";
import GatewaySocketAnalytics from "GatewaySocketAnalytics.tsx";
import VoiceServerUpdateImmediateExperiment from "VoiceServerUpdateImmediateExperiment.tsx";
import ConnectionStateDefault from "ConnectionState.tsx";
import ActionBatcherDefault from "ActionBatcher.tsx";
import _slicedToArray from "../../../_runtime/metro/00032__.js";

require = fn;
let closure_4 = fn(14900).DISPATCHER_IDEAL_TIME_LIMIT_MS;
let closure_5 = new LoggerDefault("GatewaySocket");
const set = new Set(["INITIAL_GUILD", "READY"]);
const set1 = new Set(["READY", "INITIAL_GUILD"]);
const set2 = new Set(["VOICE_SERVER_UPDATE", "STREAM_SERVER_UPDATE"]);
const set3 = new Set(["READY", "READY_SUPPLEMENTAL", "RESUMED"]);
const set4 = new Set([
  "READY",
  "INITIAL_GUILD",
  "READY_SUPPLEMENTAL",
  "RESUMED",
  "VOICE_CHANNEL_SELECT",
  "VOICE_STATE_UPDATE",
  "VOICE_SERVER_UPDATE",
  "RTC_CONNECTION_STATE",
  "RTC_CONNECTION_VIDEO",
  "RTC_CONNECTION_CLIENT_CONNECT",
  "RTC_CONNECTION_PING",
  "MEDIA_SESSION_JOINED",
  "MEDIA_ENGINE_PERMISSION",
  "SESSIONS_REPLACE",
  "STREAM_CREATE",
  "STREAM_SERVER_UPDATE",
  "STREAM_DELETE",
  "STREAM_UPDATE",
]);
let closure_11 = { NotStarted: 0, [0]: "NotStarted", Loading: 1, [1]: "Loading", Loaded: 2, [2]: "Loaded" };
let closure_12 = {};
const size = fn(2);
let result = size.fileFinishedImporting("modules/gateway/GatewaySocketDispatcher.tsx");
class GatewaySocketDispatcher {
  constructor(arg0) {
    obj1 = Object.create(new.target.prototype);
    closure_0 = obj1;
    obj = closure_0(closure_2[3]);
    obj1.scheduler = obj.createDispatcherWorkScheduler();
    obj1.queue = [];
    obj1.paused = true;
    obj2 = closure_0(closure_2[4]);
    obj1.resumeAnalytics = obj2.createResumeAnalytics();
    obj1.getDispatchHandler = null;
    obj1.flush = function flush(arg0) {
      if (obj3.paused) {
        return true;
      } else {
        const _performance = performance;
        let num2 = 0;
        let obj = obj3;
        if (0 < obj3.queue.length) {
          let num4 = 0;
          num2 = 0;
          obj = obj3;
          if (obj3.queue[0].status === closure_11.Loaded) {
            const sum = num4 + 1;
            num2 = sum;
            obj = obj3;
            while (sum < obj3.queue.length) {
              num4 = sum;
              num2 = sum;
              obj = obj3;
              if (obj3.queue[sum].status !== closure_11.Loaded) {
                break;
              }
            }
          }
        }
        if (0 === num2) {
          return true;
        } else {
          const queue = obj.queue;
          const spliceResult = queue.splice(0, num2);
          const dispatchMultipleResult = obj.dispatchMultiple(spliceResult, arg0);
          if (dispatchMultipleResult) {
            const telemetry = obj.scheduler.telemetry;
            telemetry.timeEnd(WorkSchedulerTelemetry.WorkSchedulerTelemetryTiming.TIME_TO_QUEUE_EMPTY);
          }
          const _performance2 = performance;
          const diff = performance.now() - nowResult;
          if (tmp13) {
            const _HermesInternal = HermesInternal;
            logger.log("Dispatched " + spliceResult.length + " messages in " + diff + "ms");
          }
          return dispatchMultipleResult;
        }
        nowResult = performance.now();
      }
    };
    obj1.socket = global;
    return obj1;
  }
}
const prototype = GatewaySocketDispatcher.prototype;
prototype["hasStuffToDispatchNow"] = function hasStuffToDispatchNow() {
  let tmp = this.queue.length > 0;
  if (tmp) {
    tmp = this.queue[0].status === closure_11.Loaded;
  }
  return tmp;
};
prototype["processFirstQueuedDispatch"] = function processFirstQueuedDispatch(set) {
  const self = this;
  items = [];
  if (this.queue.length > 0) {
    if (set.has(self.queue[0].type)) {
      if (self.queue[0].status === closure_11.Loaded) {
        const queue = self.queue;
        items.push(queue.shift());
        while (self.queue.length > 0) {
          if (!set.has(self.queue[0].type)) {
            break;
          } else if (self.queue[0].status !== closure_11.Loaded) {
            break;
          }
        }
      }
    }
  }
  self.dispatchMultiple(items);
};
prototype["unpauseDispatchQueue"] = function unpauseDispatchQueue() {
  const self = this;
  this.paused = false;
  for (const item10007 of tmp) {
    let maybePreloadResult = self.maybePreload(item10007);
    continue;
  }
  self.flush();
};
prototype["receiveDispatch"] = function receiveDispatch(data, type, compressionAnalytics) {
  const self = this;
  if (null == this.getDispatchHandler) {
    const _Error = Error;
    throw Error("getDispatchHandler needs to be passed in first!");
  } else {
    const obj = {
      data,
      type,
      compressionAnalytics,
      status: closure_11.NotStarted,
      preloadPromise: null,
      preloadedData: null,
      receivedAt: TimeUtils.now(),
    };
    const queue = self.queue;
    queue.push(obj);
    if (!self.maybePreload(obj)) {
      self.scheduleFlush(type);
    }
  }
};
prototype["maybePreload"] = function maybePreload(item10007) {
  const self = this;
  if (this.paused) {
    if (!set.has(item10007.type)) {
      return false;
    }
  }
  if (item10007.status === Loaded.NotStarted) {
    const dispatchHandler = self.getDispatchHandler(item10007.type);
    let preloadResult;
    if (dispatchHandler != null) {
      preloadResult = dispatchHandler.preload(item10007.data);
    }
    item10007.status = null == preloadResult ? Loaded.Loaded : Loaded.Loading;
    item10007.preloadPromise = preloadResult;
    if (null != preloadResult) {
      preloadResult
        .then((preloadedData) => {
          item10007.preloadedData = preloadedData;
          item10007.status = Loaded.Loaded;
          self.scheduleFlush(item10007.type);
        })
        .catch((error) => {
          const socket = self.socket;
          return socket.resetSocketOnDispatchError({ error, action: item10007.type });
        });
      return true;
    }
  }
  return false;
};
prototype["shouldFlushImmediately"] = function shouldFlushImmediately(arg0) {
  let hasItem = set1.has(arg0);
  if (!hasItem) {
    let result = set2.has(arg0);
    if (result) {
      result = VoiceServerUpdateImmediateExperiment.isVoiceServerUpdateImmediateEnabled("GatewaySocketDispatcher");
    }
    hasItem = result;
  }
  return hasItem;
};
prototype["scheduleFlush"] = function scheduleFlush(arg0) {
  const self = this;
  if (!this.paused) {
    const scheduler = self.scheduler;
    if (self.shouldFlushImmediately(arg0)) {
      scheduler.clearWorkTimeout();
      self.flush();
    } else if (!scheduler.hasWorkScheduled) {
      const scheduler2 = self.scheduler;
      const workTimeout = scheduler2.requestWorkTimeout(self.flush);
    }
    if (set4.has(arg0)) {
      const scheduler3 = self.scheduler;
      const result = scheduler3.markCriticalWorkScheduled();
    }
  }
};
prototype["getDispatchTimings"] = function getDispatchTimings() {
  return closure_12;
};
prototype["getSchedulerTelemetry"] = function getSchedulerTelemetry() {
  return this.scheduler.telemetry;
};
prototype["getIsSchedulerBackgrounded"] = function getIsSchedulerBackgrounded() {
  return this.scheduler.isBackgrounded;
};
prototype["toggleRequestIdleCallback"] = function toggleRequestIdleCallback(arg0) {
  const scheduler = this.scheduler;
  const result = scheduler.toggleRequestIdleCallback(arg0);
};
prototype["getIsRequestIdleCallbackEnabled"] = function getIsRequestIdleCallbackEnabled() {
  return this.scheduler.isRequestIdleCallbackEnabled;
};
prototype["dispatchMultiple"] = function dispatchMultiple(items, arg1) {
  const self = this;
  importDefault = items;
  dependencyMap = arg1;
  if (0 === items.length) {
    return true;
  } else {
    let type = "none";
    let hasItem = false;
    const telemetry2 = self.scheduler.telemetry;
    telemetry2.measure(
      require("WorkSchedulerTelemetry").WorkSchedulerTelemetryMeasurement.COUNT_INITIAL_DISPATCHS_LENGTH,
      items.length,
    );
    try {
      closure_5 = [];
      if (self.socket.connectionState === ConnectionStateDefault.RESUMING) {
        const Emitter = tmp2(504).Emitter;
        Emitter.pause(150);
      }
      _require = 0;
      const Emitter2 = tmp2(504).Emitter;
      Emitter2.batched(() => {
        let arr;
        let sum;
        let num = 0;
        if (0 < items.length) {
          while (true) {
            arr = items;
            let tmp = items[num];
            type = tmp.type;
            if (!hasItem) {
              hasItem = set3.has(tmp.type);
            }
            let _performance = performance;
            let nowResult = performance.now();
            let dispatchOneResult = self.dispatchOne(tmp);
            let _performance2 = performance;
            closure_0 = performance.now() - nowResult;
            type = tmp.type;
            items = closure_12[type];
            let tmp8 = closure_0;
            if (items == null) {
              items = [0, 0];
            }
            let tmp11 = _slicedToArray(items, 2);
            let tmp12 = tmp11[1];
            let items1 = [(tmp11[0] * tmp12 + tmp8) / (tmp12 + 1), tmp12 + 1];
            closure_12[type] = items1;
            let flag = false;
            if (null != closure_2) {
              let diff = arr.length - 1;
              let tmp14 = null;
              if (num < diff) {
                tmp14 = arr[num + 1];
              }
              let num2;
              if (closure_2 != null) {
                num2 = closure_2.timeRemaining();
              }
              if (num2 == null) {
                num2 = 0;
              }
              let type1;
              if (tmp14 != null) {
                type1 = tmp14.type;
              }
              let tmp16 = null != closure_2 && num2 <= 0 && arr[num].type !== type1 && num !== diff;
              flag = tmp16;
            }
            sum = num + 1;
            if (flag) {
              break;
            } else {
              num = sum;
            }
          }
          closure_5 = arr.slice(sum);
          if (tmp18) {
            const telemetry = self.scheduler.telemetry;
            telemetry.timeTrack(
              WorkSchedulerTelemetry.WorkSchedulerTelemetryTiming.TIME_OVER_DEADLINE,
              closure_2.timeSinceExpiration,
            );
          }
          tmp18 = null != closure_2 && closure_2.timeRemaining() <= 0;
        }
        ActionBatcherDefault.flush();
      });
      if (hasItem) {
        const Emitter3 = tmp2(504).Emitter;
        Emitter3.resume();
      }
      if (closure_5.length > 0) {
        let telemetry = self.scheduler.telemetry;
        telemetry.measure(
          tmp21(14903).WorkSchedulerTelemetryMeasurement.COUNT_DISPATCHES_LEFT_AFTER_YIELD,
          closure_5.length,
        );
        const queue = self.queue;
        const unshift = queue.unshift;
        items = [];
        HermesBuiltin.arraySpread(closure_5, 0);
        HermesBuiltin.apply(items, queue);
        const scheduler = self.scheduler;
        const workTimeout = scheduler.requestWorkTimeout(self.flush, true);
        return false;
      } else {
        return true;
      }
    } catch (tmp18) {
      const socket = tmp.socket;
      const obj = { error: tmp18, action: type };
      const result = socket.resetSocketOnDispatchError(obj);
    }
    tmp21 = _require;
  }
};
prototype["dispatchOne"] = function dispatchOne(arg0) {
  const self = this;
  ({ data, type, compressionAnalytics, preloadedData, receivedAt } = arg0);
  const nowResult = performance.now();
  if (this.socket.connectionState !== ConnectionStateDefault.RESUMING) {
    ActionBatcherDefault.flush(type, data);
    if ("READY" === type) {
      const readyPayloadByteSizeAnalytics = GatewaySocketAnalytics.getReadyPayloadByteSizeAnalytics(data);
      const dispatchHandler = self.getDispatchHandler(type);
      if (dispatchHandler != null) {
        dispatchHandler.dispatch(data, type, preloadedData, receivedAt);
      }
      const tmp16Result = GatewaySocketAnalytics;
      const result = tmp16Result.logReadyPayloadReceived(
        self.socket,
        data,
        nowResult,
        compressionAnalytics,
        readyPayloadByteSizeAnalytics,
      );
    } else if ("RESUMED" === type) {
      const dispatchHandler1 = self.getDispatchHandler(type);
      if (dispatchHandler1 != null) {
        dispatchHandler1.dispatch(data, type, preloadedData, receivedAt);
      }
      GatewaySocketAnalytics.logResumeAnalytics(self.resumeAnalytics);
      const socket = self.socket;
      const result1 = socket.handleResumeDispatched();
      self.resumeAnalytics = GatewaySocketAnalytics.createResumeAnalytics();
    } else {
      const dispatchHandler2 = self.getDispatchHandler(type);
      if (dispatchHandler2 != null) {
        dispatchHandler2.dispatch(data, type, preloadedData, receivedAt);
      }
    }
    if (self.socket.connectionState === ConnectionStateDefault.RESUMING) {
      const resumeAnalytics3 = self.resumeAnalytics;
      const _performance = performance;
      resumeAnalytics3.dispatchTime = resumeAnalytics3.dispatchTime + (performance.now() - nowResult);
    }
    const tmp2Result = ActionBatcherDefault;
  } else {
    const diff = nowResult - self.resumeAnalytics.lastUpdateTime;
    if (0 === self.resumeAnalytics.numEvents) {
      self.resumeAnalytics.initialWaitTime = diff;
    } else if (diff > self.resumeAnalytics.largestWaitTime) {
      self.resumeAnalytics.largestWaitTime = diff;
    }
    const resumeAnalytics = self.resumeAnalytics;
    resumeAnalytics.totalWaitTime = resumeAnalytics.totalWaitTime + diff;
    self.resumeAnalytics.lastUpdateTime = nowResult;
    const resumeAnalytics2 = self.resumeAnalytics;
    resumeAnalytics2.numEvents = resumeAnalytics2.numEvents + 1;
  }
};
prototype["clear"] = function clear() {
  this.paused = false;
  this.queue.length = 0;
};

export default GatewaySocketDispatcher;
