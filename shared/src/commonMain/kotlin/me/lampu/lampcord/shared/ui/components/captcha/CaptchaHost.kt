package me.lampu.lampcord.shared.ui.components.captcha

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import dev.nucleusframework.webview.web.LoadingState
import dev.nucleusframework.webview.web.WebView
import dev.nucleusframework.webview.web.rememberWebViewNavigator
import dev.nucleusframework.webview.web.rememberWebViewState
import kotlinx.coroutines.delay
import kotlinx.serialization.json.JsonPrimitive
import kotlin.math.roundToInt
import me.lampu.lampcord.shared.api.CaptchaChallenge
import me.lampu.lampcord.shared.captcha.CaptchaCoordinator
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_description
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_title
import me.lampu.lampcord.shared.generated.resources.lampcord_captcha_webview_missing
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.isEmbeddedWebViewAvailable
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

internal const val CAPTCHA_TITLE_PREFIX = "LAMPCORD_CAPTCHA:"
internal const val CAPTCHA_READY_TITLE = "LAMPCORD_READY"
internal const val CAPTCHA_EVENT_PREFIX = "LAMPCORD_EVENT:"
private const val CAPTCHA_PAGE_URL = "https://discord.com/login"

private const val HIDDEN_CARD_ALPHA = 0.02f
private const val HIDDEN_CARD_OFFSET_PX = 20_000
private const val PAGE_SETTLE_MS = 600L
private const val REVEAL_AFTER_MS = 5_000L

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
    var revealed by remember { mutableStateOf(!webViewAvailable) }
    LaunchedEffect(Unit) {
        delay(REVEAL_AFTER_MS)
        revealed = true
        expanded = webViewAvailable
    }
    val cardAlpha by animateFloatAsState(
        targetValue = if (revealed) 1f else HIDDEN_CARD_ALPHA,
        label = "captchaCardAlpha",
    )
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
                .padding(outerPadding)
                .offset { if (revealed) IntOffset.Zero else IntOffset(-HIDDEN_CARD_OFFSET_PX, 0) }
                .graphicsLayer { alpha = cardAlpha },
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
                    CaptchaFrame(
                        challenge = challenge,
                        background = colors.surfaceContainer,
                        expanded = expanded,
                        onExpandedChange = { expanded = it },
                        onReveal = { revealed = true },
                        onToken = {
                            Logging.i(
                                "Captcha",
                                if (revealed) "Solved after the dialog was shown" else "Solved silently, dialog never shown",
                            )
                            onToken(it)
                        },
                    )
                } else {
                    Text(
                        text = stringResource(Res.string.lampcord_captcha_webview_missing),
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.error,
                    )
                }
                Spacer(Modifier.height(16.dp))
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    TextButton(onClick = onCancel) {
                        Text(stringResource(Res.string.cancel), color = colors.primary)
                    }
                }
            }
        }

        if (!revealed) {
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .pointerInput(Unit) { detectTapGestures { } },
                contentAlignment = Alignment.Center,
            ) {
                ContainedLoadingIndicator(indicatorColor = colors.primary)
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
    onReveal: () -> Unit,
    onToken: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val navigator = rememberWebViewNavigator()
    val webViewState = rememberWebViewState(CAPTCHA_PAGE_URL)

    val isDark = background.luminance() < 0.5f
    val script = remember(challenge, background) {
        captchaInjectionScript(
            challenge = challenge,
            background = background.toCssHex(),
            theme = if (isDark) "dark" else "light",
        )
    }

    var injected by remember(script) { mutableStateOf(false) }
    var widgetReady by remember(script) { mutableStateOf(false) }
    var lastEvent by remember { mutableStateOf<String?>(null) }
    val pageLoaded = webViewState.loadingState is LoadingState.Finished

    LaunchedEffect(webViewState) {
        webViewState.webSettings.isJavaScriptEnabled = true
        navigator.loadUrl(CAPTCHA_PAGE_URL, emptyMap())
    }

    LaunchedEffect(pageLoaded, script) {
        if (pageLoaded) {
            delay(PAGE_SETTLE_MS)
            navigator.evaluateJavaScript(script)
            injected = true
        }
    }

    val initialTitle = remember { webViewState.pageTitle }
    val title = webViewState.pageTitle
    LaunchedEffect(title) {
        if (title == CAPTCHA_READY_TITLE) {
            widgetReady = true
        } else if (title != null && title.startsWith(CAPTCHA_EVENT_PREFIX)) {
            widgetReady = true
            val event = title.removePrefix(CAPTCHA_EVENT_PREFIX)
            if (event != lastEvent) {
                lastEvent = event
                Logging.i("Captcha", "hCaptcha: $event")
            }
            if (event == "open") {
                onReveal()
                onExpandedChange(true)
            }
            if (event == "close") onExpandedChange(false)
        } else if (title != null && title.startsWith(CAPTCHA_TITLE_PREFIX) && title != initialTitle) {
            widgetReady = true
            val token = title.removePrefix(CAPTCHA_TITLE_PREFIX)
            Logging.i("Captcha", "Received solved token, length=${token.length}")
            onToken(token)
        }
    }

    val frameHeight by animateDpAsState(
        targetValue = if (expanded) 480.dp else 96.dp,
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
            modifier = Modifier.fillMaxSize(),
        )
        if (!widgetReady) {
            ContainedLoadingIndicator(indicatorColor = colors.primary)
        }
    }
}

internal fun captchaInjectionScript(
    challenge: CaptchaChallenge,
    background: String,
    theme: String,
): String {
    val sitekey = jsString(challenge.sitekey)
    val rqdata = challenge.rqdata?.let { jsString(it) } ?: "null"
    val prefix = jsString(CAPTCHA_TITLE_PREFIX)
    val ready = jsString(CAPTCHA_READY_TITLE)
    val eventPrefix = jsString(CAPTCHA_EVENT_PREFIX)
    val bg = jsString(background)
    val themeLiteral = jsString(theme)
    return """
(function () {
  if (window.__lampcordTimer) { clearInterval(window.__lampcordTimer); }
  var stale = document.getElementById('lampcord-captcha');
  if (stale && stale.parentNode) { stale.parentNode.removeChild(stale); }
  var staleStyle = document.getElementById('lampcord-captcha-style');
  if (staleStyle && staleStyle.parentNode) { staleStyle.parentNode.removeChild(staleStyle); }

  var doc = document;

  var lock = doc.createElement('style');
  lock.id = 'lampcord-captcha-style';
  lock.textContent = 'html,body{overflow:hidden !important;width:100% !important;height:100% !important;' +
    'max-width:100% !important;margin:0 !important;overscroll-behavior:none !important;}' +
    '#lampcord-captcha{overflow:hidden;overscroll-behavior:none;}';
  (doc.head || doc.documentElement).appendChild(lock);

  var viewport = doc.querySelector('meta[name="viewport"]');
  if (!viewport) {
    viewport = doc.createElement('meta');
    viewport.name = 'viewport';
    (doc.head || doc.documentElement).appendChild(viewport);
  }
  viewport.content = 'width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no';

  var rendered = false;
  var widgetId = null;
  var executed = false;
  var executeTries = 0;
  var solved = false;
  var rq = $rqdata;

  var status = '';
  doc.title = status;

  var overlay = doc.createElement('div');
  overlay.id = 'lampcord-captcha';
  overlay.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;z-index:100000;' +
    'display:flex;align-items:center;justify-content:center;background:' + $bg + ';';
  var box = doc.createElement('div');
  box.style.cssText = 'max-width:100%;';
  overlay.appendChild(box);
  doc.documentElement.appendChild(overlay);

  function setStatus(value) {
    if (solved) { return; }
    status = value;
    doc.title = status;
  }
  function report(message) { setStatus($eventPrefix + message); }

  window.lampcordDone = function (token) {
    solved = true;
    status = $prefix + token;
    doc.title = status;
  };

  function render() {
    if (rendered || !window.hcaptcha) { return; }
    rendered = true;
    var opts = {
      sitekey: $sitekey,
      theme: $themeLiteral,
      callback: window.lampcordDone,
      'open-callback': function () { report('open'); },
      'close-callback': function () { report('close'); },
      'error-callback': function (code) { report('error:' + code); },
      'expired-callback': function () { report('expired'); },
      'chalexpired-callback': function () { report('challenge-expired'); }
    };
    if (rq) { opts.rqdata = rq; }
    try {
      widgetId = hcaptcha.render(box, opts);
      setStatus($ready);
    } catch (e) {
      report('render-exception:' + e);
    }
  }
  window.lampcordReady = render;

  function runExecute() {
    if (!rq || widgetId === null || executed || executeTries >= 5) { return; }
    executeTries++;
    try {
      var pending = hcaptcha.execute(widgetId, { rqdata: rq });
      executed = true;
      if (pending && typeof pending.catch === 'function') {
        pending.catch(function (e) { report('execute-error:' + e); });
      }
    } catch (e) {
      report('execute-failed:' + executeTries + ':' + e);
    }
  }

  function fit() {
    var natural = box.scrollWidth;
    var available = doc.documentElement.clientWidth - 24;
    if (natural > 0 && natural > available) {
      box.style.transformOrigin = 'center center';
      box.style.transform = 'scale(' + (available / natural) + ')';
    } else {
      box.style.transform = '';
    }
  }

  window.__lampcordTimer = setInterval(function () {
    if (!overlay.isConnected) { doc.documentElement.appendChild(overlay); }
    if (doc.title !== status) { doc.title = status; }
    if (window.scrollX !== 0 || window.scrollY !== 0) { window.scrollTo(0, 0); }
    render();
    runExecute();
    fit();
  }, 500);

  var s = doc.createElement('script');
  s.src = 'https://js.hcaptcha.com/1/api.js?render=explicit&onload=lampcordReady';
  s.onerror = function () { report('script-load-error'); };
  doc.documentElement.appendChild(s);
})();
"""
}

private fun jsString(value: String): String = JsonPrimitive(value).toString()

private fun Color.toCssHex(): String =
    "#" + listOf(red, green, blue).joinToString("") {
        (it * 255f).roundToInt().coerceIn(0, 255).toString(16).padStart(2, '0')
    }
