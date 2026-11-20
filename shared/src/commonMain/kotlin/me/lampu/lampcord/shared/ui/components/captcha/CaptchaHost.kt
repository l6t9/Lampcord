package me.lampu.lampcord.shared.ui.components.captcha

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ContainedLoadingIndicator
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import dev.nucleusframework.webview.jsbridge.IJsMessageHandler
import dev.nucleusframework.webview.jsbridge.JsMessage
import dev.nucleusframework.webview.web.WebView
import dev.nucleusframework.webview.web.WebViewNavigator
import dev.nucleusframework.webview.jsbridge.rememberWebViewJsBridge
import dev.nucleusframework.webview.web.rememberWebViewNavigator
import dev.nucleusframework.webview.web.rememberWebViewStateWithHTMLData
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.SendChannel
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlin.math.roundToInt
import me.lampu.lampcord.shared.api.CaptchaChallenge
import me.lampu.lampcord.shared.api.CaptchaService
import me.lampu.lampcord.shared.api.RECAPTCHA_ENTERPRISE_ACTION
import me.lampu.lampcord.shared.captcha.CaptchaCoordinator
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_description
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_failed
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_retry
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_title
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_webview_missing
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.configureCaptchaWebView
import me.lampu.lampcord.shared.utils.isEmbeddedWebViewAvailable
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

internal const val CAPTCHA_BRIDGE_METHOD = "lampcord.captcha"
internal const val CAPTCHA_BASE_URL = "https://discord.com/login"

private const val CAPTCHA_USER_AGENT =
    "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36"
private const val COLLAPSED_FRAME_DP = 96
private const val EXPANDED_FRAME_DP = 480

internal val TERMINAL_CAPTCHA_EVENTS = setOf(
    "error",
    "expired",
    "challenge-expired",
    "execute-failed",
    "render-exception",
    "set-data-error",
    "script-load-error",
)

internal data class CaptchaBridgeEvent(
    val seq: Long,
    val kind: String,
    val value: String?,
)

private fun loggedValue(event: CaptchaBridgeEvent): String =
    if (event.kind == "token") "${event.value?.length ?: 0} chars" else event.value.orEmpty()

internal fun parseCaptchaBridgeEvent(params: String): CaptchaBridgeEvent? {
    val obj = runCatching { Json.parseToJsonElement(params).jsonObject }.getOrNull() ?: return null
    val kind = obj["kind"]?.jsonPrimitive?.contentOrNull ?: return null
    val seq = obj["seq"]?.jsonPrimitive?.longOrNull ?: return null
    val value = obj["value"]?.jsonPrimitive?.contentOrNull
    return CaptchaBridgeEvent(seq, kind, value)
}

@Composable
fun CaptchaHost(coordinator: CaptchaCoordinator = koinInject()) {
    val request by coordinator.pending.collectAsState()
    val current = request ?: return

    key(current) {
        Dialog(
            onDismissRequest = { coordinator.cancel() },
            properties = DialogProperties(
                usePlatformDefaultWidth = false,
                dismissOnClickOutside = false,
            ),
        ) {
            CaptchaDialogContent(
                challenge = current.challenge,
                onToken = { coordinator.complete(it) },
                onCancel = { coordinator.cancel() },
            )
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun CaptchaDialogContent(
    challenge: CaptchaChallenge,
    onToken: (String) -> Unit,
    onCancel: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val webViewAvailable = remember { isEmbeddedWebViewAvailable() }
    var expanded by remember { mutableStateOf(false) }
    var failure by remember { mutableStateOf<String?>(null) }
    var attempt by remember { mutableIntStateOf(0) }
    val outerPadding by animateDpAsState(
        targetValue = if (expanded) 8.dp else 24.dp,
        label = "captchaOuterPadding",
    )
    val innerPadding by animateDpAsState(
        targetValue = if (expanded) 12.dp else 24.dp,
        label = "captchaInnerPadding",
    )

    Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
        Surface(
            modifier = Modifier
                .widthIn(max = 420.dp)
                .fillMaxWidth()
                .padding(outerPadding),
            shape = RoundedCornerShape(28.dp),
            color = colors.surfaceContainerHigh,
            tonalElevation = 0.dp,
        ) {
            Column(Modifier.padding(horizontal = innerPadding, vertical = 24.dp)) {
                Text(
                    text = stringResource(Res.string.lampcord_captcha_title),
                    style = MaterialTheme.typography.headlineSmall,
                    color = colors.onSurface,
                )
                if (!expanded) {
                    Spacer(Modifier.height(8.dp))
                    Text(
                        text = stringResource(Res.string.lampcord_captcha_description),
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.onSurfaceVariant,
                    )
                }
                Spacer(Modifier.height(if (expanded) 12.dp else 20.dp))
                if (webViewAvailable) {
                    key(attempt) {
                        CaptchaFrame(
                            challenge = challenge,
                            background = colors.surfaceContainer,
                            expanded = expanded,
                            onExpandedChange = { expanded = it },
                            onFailure = { failure = it },
                            onToken = onToken,
                        )
                    }
                    failure?.let {
                        Spacer(Modifier.height(8.dp))
                        Text(
                            text = stringResource(Res.string.lampcord_captcha_failed),
                            style = MaterialTheme.typography.bodyMedium,
                            color = colors.error,
                        )
                    }
                } else {
                    Text(
                        text = stringResource(Res.string.lampcord_captcha_webview_missing),
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.error,
                    )
                }
                Spacer(Modifier.height(16.dp))
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    if (failure != null) {
                        TextButton(
                            onClick = {
                                failure = null
                                expanded = false
                                attempt++
                            },
                        ) {
                            Text(stringResource(Res.string.lampcord_captcha_retry), color = colors.primary)
                        }
                        Spacer(Modifier.width(8.dp))
                    }
                    TextButton(onClick = onCancel) {
                        Text(stringResource(Res.string.cancel), color = colors.primary)
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun CaptchaFrame(
    challenge: CaptchaChallenge,
    background: Color,
    expanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    onFailure: (String) -> Unit,
    onToken: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val events = remember { Channel<CaptchaBridgeEvent>(Channel.UNLIMITED) }
    val navigator = rememberWebViewNavigator()
    val jsBridge = rememberWebViewJsBridge(navigator)
    val isDark = background.luminance() < 0.5f
    val html = remember(challenge, background) {
        captchaPageHtml(
            challenge = challenge,
            background = background.toCssHex(),
            theme = if (isDark) "dark" else "light",
        )
    }
    val webViewState = rememberWebViewStateWithHTMLData(html, baseUrl = CAPTCHA_BASE_URL)
    var ready by remember { mutableStateOf(false) }

    LaunchedEffect(webViewState) {
        webViewState.webSettings.isJavaScriptEnabled = true
        webViewState.webSettings.customUserAgentString = CAPTCHA_USER_AGENT
    }

    LaunchedEffect(jsBridge) {
        jsBridge.register(CaptchaBridgeHandler(events))
    }

    LaunchedEffect(events) {
        var lastSeq = 0L
        for (event in events) {
            if (event.seq <= lastSeq) continue
            lastSeq = event.seq
            Logging.i("Captcha", "event: ${event.kind}: ${loggedValue(event)}")
            when (event.kind) {
                "ready" -> ready = true
                "open" -> onExpandedChange(true)
                "close" -> onExpandedChange(false)
                "token" -> event.value?.takeIf { it.isNotEmpty() }?.let(onToken)
            }
            if (event.kind in TERMINAL_CAPTCHA_EVENTS) onFailure(event.kind)
        }
    }

    val frameHeight by animateDpAsState(
        targetValue = if (expanded) EXPANDED_FRAME_DP.dp else COLLAPSED_FRAME_DP.dp,
        label = "captchaFrameHeight",
    )

    val shape = RoundedCornerShape(16.dp)
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(frameHeight)
            .clip(shape)
            .background(background)
            .border(1.dp, colors.outlineVariant, shape),
        contentAlignment = Alignment.Center,
    ) {
        WebView(
            state = webViewState,
            navigator = navigator,
            webViewJsBridge = jsBridge,
            modifier = Modifier.fillMaxSize(),
            onCreated = { native -> configureCaptchaWebView(native) },
        )
        if (!ready) {
            ContainedLoadingIndicator(indicatorColor = colors.primary)
        }
    }
}

private class CaptchaBridgeHandler(
    private val events: SendChannel<CaptchaBridgeEvent>,
) : IJsMessageHandler {
    override fun methodName(): String = CAPTCHA_BRIDGE_METHOD

    override fun handle(
        message: JsMessage,
        navigator: WebViewNavigator?,
        callback: (String) -> Unit,
    ) {
        parseCaptchaBridgeEvent(message.params)?.let { events.trySend(it) }
    }
}

internal fun captchaPageHtml(
    challenge: CaptchaChallenge,
    background: String,
    theme: String,
): String {
    val sitekey = jsString(challenge.sitekey)
    val rqdata = challenge.rqdata?.let { jsString(it) } ?: "null"
    val invisible = if (challenge.isInvisible) "true" else "false"
    val themeLiteral = jsString(theme)
    val actionLiteral = jsString(RECAPTCHA_ENTERPRISE_ACTION)
    val scriptSrc = jsString(captchaScriptSource(challenge))
    val serviceBlock = when (challenge.service) {
        CaptchaService.HCAPTCHA -> HCAPTCHA_SERVICE_JS
        CaptchaService.RECAPTCHA -> RECAPTCHA_SERVICE_JS
        CaptchaService.RECAPTCHA_ENTERPRISE -> RECAPTCHA_ENTERPRISE_SERVICE_JS
    }
    return """
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<style>
html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: $background; }
body { display: flex; align-items: center; justify-content: center; }
#captcha { display: flex; min-height: 78px; width: 100%; align-items: center; justify-content: center; }
</style>
</head>
<body>
<div id="captcha"></div>
<script>
(function () {
  var SITEKEY = $sitekey;
  var RQDATA = $rqdata;
  var INVISIBLE = $invisible;
  var THEME = $themeLiteral;
  var ACTION = $actionLiteral;
  var SRC = $scriptSrc;

  var widgetId = null;
  var executed = false;
  var finished = false;
  var seq = 0;
  var queue = [];

  function text(value) {
    if (value === null || value === undefined) { return 'null'; }
    if (typeof value === 'string') { return value; }
    if (value instanceof Error) { return value.name + ': ' + value.message; }
    try { return JSON.stringify(value); } catch (_) { return String(value); }
  }

  function flush() {
    while (queue.length > 0) {
      try {
        window.kmpJsBridge.callNative('lampcord.captcha', JSON.stringify(queue[0]), function () {});
      } catch (_) {
        return;
      }
      queue.shift();
    }
  }

  function send(kind, value) {
    seq += 1;
    queue.push({ seq: seq, kind: kind, value: value === undefined ? null : text(value) });
    flush();
  }

  function solved(token) {
    if (finished) { return; }
    finished = true;
    send('token', token);
  }

  setInterval(flush, 250);

  window.addEventListener('error', function (e) {
    send('window-error', (e.message || 'error') + ' line ' + e.lineno);
  });

  var originalConsoleError = console.error;
  console.error = function () {
    send('console-error', Array.prototype.map.call(arguments, function (v) { return text(v); }).join(' '));
    originalConsoleError.apply(console, arguments);
  };

  function loadScript() {
    var s = document.createElement('script');
    s.src = SRC;
    s.onload = function () { renderService(); };
    s.onerror = function () { send('script-load-error'); };
    document.body.appendChild(s);
  }

$serviceBlock

  loadScript();
})();
</script>
</body>
</html>
"""
}

private fun captchaScriptSource(challenge: CaptchaChallenge): String = when (challenge.service) {
    CaptchaService.HCAPTCHA -> "https://js.hcaptcha.com/1/api.js?render=explicit"
    CaptchaService.RECAPTCHA -> "https://www.google.com/recaptcha/api.js?render=explicit"
    CaptchaService.RECAPTCHA_ENTERPRISE -> "https://www.google.com/recaptcha/enterprise.js?render=${challenge.sitekey}"
}

private const val HCAPTCHA_SERVICE_JS = """
  function renderService() {
    if (!window.hcaptcha) { send('render-exception', 'hcaptcha missing'); return; }
    var opts = {
      sitekey: SITEKEY,
      theme: THEME,
      callback: solved,
      'open-callback': function () { send('open'); },
      'close-callback': function () { send('close'); },
      'error-callback': function (code) { send('error', code); },
      'expired-callback': function () { send('expired'); },
      'chalexpired-callback': function () { send('challenge-expired'); }
    };
    if (INVISIBLE) { opts.size = 'invisible'; }
    try {
      widgetId = hcaptcha.render('captcha', opts);
    } catch (e) {
      send('render-exception', e);
      return;
    }
    if (RQDATA !== null) {
      try { hcaptcha.setData(widgetId, { rqdata: RQDATA }); }
      catch (e) { send('set-data-error', e); }
    }
    send('ready');
    executeService();
  }

  function executeService() {
    if (!INVISIBLE || widgetId === null || executed) { return; }
    executed = true;
    try {
      var pending = hcaptcha.execute(widgetId);
      if (pending && typeof pending.catch === 'function') {
        pending.catch(function (e) { send('execute-failed', e); });
      }
    } catch (e) {
      send('execute-failed', e);
    }
  }
"""

private const val RECAPTCHA_SERVICE_JS = """
  function renderService() {
    if (!window.grecaptcha || typeof grecaptcha.render !== 'function') { send('render-exception', 'grecaptcha missing'); return; }
    var opts = {
      sitekey: SITEKEY,
      theme: THEME,
      callback: solved,
      'expired-callback': function () { send('expired'); },
      'error-callback': function () { send('error', 'recaptcha'); }
    };
    if (INVISIBLE) { opts.size = 'invisible'; }
    try {
      widgetId = grecaptcha.render('captcha', opts);
    } catch (e) {
      send('render-exception', e);
      return;
    }
    send('ready');
    executeService();
  }

  function executeService() {
    if (!INVISIBLE || widgetId === null || executed) { return; }
    executed = true;
    try {
      grecaptcha.execute(widgetId);
    } catch (e) {
      send('execute-failed', e);
    }
  }
"""

private const val RECAPTCHA_ENTERPRISE_SERVICE_JS = """
  function renderService() {
    if (!window.grecaptcha || !grecaptcha.enterprise) { send('render-exception', 'grecaptcha.enterprise missing'); return; }
    grecaptcha.enterprise.ready(function () {
      send('ready');
      executeService();
    });
  }

  function executeService() {
    if (executed) { return; }
    executed = true;
    try {
      grecaptcha.enterprise.execute(SITEKEY, { action: ACTION })
        .then(solved)
        .catch(function (e) { send('execute-failed', e); });
    } catch (e) {
      send('execute-failed', e);
    }
  }
"""

private fun jsString(value: String): String = JsonPrimitive(value).toString()

private fun Color.toCssHex(): String =
    "#" + listOf(red, green, blue).joinToString("") {
        (it * 255f).roundToInt().coerceIn(0, 255).toString(16).padStart(2, '0')
    }
