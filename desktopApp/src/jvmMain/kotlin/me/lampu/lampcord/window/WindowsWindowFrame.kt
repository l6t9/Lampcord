package me.lampu.lampcord.window

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.material.Icon
import androidx.compose.material.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalFontFamilyResolver
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.platform.FontLoadResult
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.FrameWindowScope
import androidx.compose.ui.window.WindowPlacement
import androidx.compose.ui.window.WindowState
import androidx.compose.ui.zIndex
import com.sun.jna.platform.win32.User32
import com.sun.jna.platform.win32.WinDef.HWND
import com.sun.jna.platform.win32.WinUser
import kotlinx.coroutines.delay
import me.lampu.lampcord.window.jna.ComposeWindowProcedure
import me.lampu.lampcord.window.jna.structure.WinUserConst.HTCAPTION
import me.lampu.lampcord.window.jna.structure.WinUserConst.HTCLIENT
import me.lampu.lampcord.window.jna.structure.WinUserConst.HTCLOSE
import me.lampu.lampcord.window.jna.structure.WinUserConst.HTMAXBUTTON
import me.lampu.lampcord.window.jna.structure.WinUserConst.HTMINBUTTON
import java.awt.Window
import kotlin.time.Duration.Companion.milliseconds

val MinimizeIcon: ImageVector =
    ImageVector
        .Builder(
            name = "Minimize",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                fill = SolidColor(Color.Black),
                fillAlpha = 1f,
                stroke = null,
                strokeAlpha = 1f,
                strokeLineWidth = 1f,
                strokeLineCap = StrokeCap.Butt,
                strokeLineJoin = StrokeJoin.Bevel,
                strokeLineMiter = 1f,
                pathFillType = PathFillType.NonZero,
            ) {
                moveTo(3.75f, 12.5f)
                horizontalLineToRelative(16.5f)
                arcToRelative(0.75f, 0.75f, 0.0f,
                    isMoreThanHalf = false,
                    isPositiveArc = false,
                    dx1 = 0.0f,
                    dy1 = -1.5f
                )
                horizontalLineTo(3.75f)
                arcToRelative(0.75f, 0.75f, 0.0f, isMoreThanHalf = false, isPositiveArc = false, dx1 = 0.0f, dy1 = 1.5f)
                close()
            }
        }.build()
val MaximizeIcon: ImageVector =
    ImageVector
        .Builder(
            name = "Maximize",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                fill = SolidColor(Color.Black),
                fillAlpha = 1f,
                stroke = null,
                strokeAlpha = 1f,
                strokeLineWidth = 1f,
                strokeLineCap = StrokeCap.Butt,
                strokeLineJoin = StrokeJoin.Bevel,
                strokeLineMiter = 1f,
                pathFillType = PathFillType.NonZero,
            ) {
                moveTo(3.0f, 6.25f)
                curveTo(3.0f, 4.45f, 4.46f, 3.0f, 6.25f, 3.0f)
                horizontalLineToRelative(11.5f)
                curveTo(19.55f, 3.0f, 21.0f, 4.46f, 21.0f, 6.25f)
                verticalLineToRelative(11.5f)
                curveToRelative(0.0f, 1.8f, -1.46f, 3.25f, -3.25f, 3.25f)
                horizontalLineTo(6.25f)
                arcTo(3.25f, 3.25f, 0.0f, false, true, 3.0f, 17.75f)
                verticalLineTo(6.25f)
                close()
                moveTo(6.25f, 4.5f)
                curveToRelative(-0.97f, 0.0f, -1.75f, 0.78f, -1.75f, 1.75f)
                verticalLineToRelative(11.5f)
                curveToRelative(0.0f, 0.97f, 0.78f, 1.75f, 1.75f, 1.75f)
                horizontalLineToRelative(11.5f)
                curveToRelative(0.97f, 0.0f, 1.75f, -0.78f, 1.75f, -1.75f)
                verticalLineTo(6.25f)
                curveToRelative(0.0f, -0.97f, -0.78f, -1.75f, -1.75f, -1.75f)
                horizontalLineTo(6.25f)
                close()
            }
        }.build()
val RestoreIcon: ImageVector =
    ImageVector
        .Builder(
            name = "Restore",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                fill = SolidColor(Color.Black),
                fillAlpha = 1f,
                stroke = null,
                strokeAlpha = 1f,
                strokeLineWidth = 1f,
                strokeLineCap = StrokeCap.Butt,
                strokeLineJoin = StrokeJoin.Bevel,
                strokeLineMiter = 1f,
                pathFillType = PathFillType.NonZero,
            ) {
                moveTo(7.52f, 5.0f)
                horizontalLineTo(6.0f)
                curveToRelative(0.13f, -1.68f, 1.53f, -3.0f, 3.24f, -3.0f)
                horizontalLineToRelative(8.0f)
                arcTo(4.75f, 4.75f, 0.0f, false, true, 22.0f, 6.75f)
                verticalLineToRelative(8.0f)
                arcToRelative(3.25f, 3.25f, 0.0f, false, true, -3.0f, 3.24f)
                verticalLineToRelative(-1.5f)
                curveToRelative(0.85f, -0.13f, 1.5f, -0.86f, 1.5f, -1.74f)
                verticalLineToRelative(-8.0f)
                curveToRelative(0.0f, -1.8f, -1.46f, -3.25f, -3.25f, -3.25f)
                horizontalLineToRelative(-8.0f)
                curveToRelative(-0.88f, 0.0f, -1.61f, 0.65f, -1.73f, 1.5f)
                close()
                moveTo(5.25f, 6.0f)
                arcTo(3.25f, 3.25f, 0.0f, false, false, 2.0f, 9.25f)
                verticalLineToRelative(9.5f)
                curveTo(2.0f, 20.55f, 3.46f, 22.0f, 5.25f, 22.0f)
                horizontalLineToRelative(9.5f)
                curveToRelative(1.8f, 0.0f, 3.25f, -1.46f, 3.25f, -3.25f)
                verticalLineToRelative(-9.5f)
                curveTo(18.0f, 7.45f, 16.55f, 6.0f, 14.75f, 6.0f)
                horizontalLineToRelative(-9.5f)
                close()
                moveTo(3.5f, 9.25f)
                curveToRelative(0.0f, -0.97f, 0.78f, -1.75f, 1.75f, -1.75f)
                horizontalLineToRelative(9.5f)
                curveToRelative(0.97f, 0.0f, 1.75f, 0.78f, 1.75f, 1.75f)
                verticalLineToRelative(9.5f)
                curveToRelative(0.0f, 0.97f, -0.78f, 1.75f, -1.75f, 1.75f)
                horizontalLineToRelative(-9.5f)
                curveToRelative(-0.97f, 0.0f, -1.75f, -0.78f, -1.75f, -1.75f)
                verticalLineToRelative(-9.5f)
                close()
            }
        }.build()
val CloseIcon: ImageVector =
    ImageVector
        .Builder(
            name = "menu",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                fill = SolidColor(Color.Black),
                fillAlpha = 1f,
                stroke = null,
                strokeAlpha = 1f,
                strokeLineWidth = 1f,
                strokeLineCap = StrokeCap.Butt,
                strokeLineJoin = StrokeJoin.Bevel,
                strokeLineMiter = 1f,
                pathFillType = PathFillType.Companion.NonZero,
            ) {
                moveToRelative(4.4f, 4.55f)
                lineToRelative(0.07f, -0.08f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, 0.98f, -0.07f)
                lineToRelative(0.08f, 0.07f)
                lineTo(12.0f, 10.94f)
                lineToRelative(6.47f, -6.47f)
                arcToRelative(0.75f, 0.75f, 0.0f, true, true, 1.06f, 1.06f)
                lineTo(13.06f, 12.0f)
                lineToRelative(6.47f, 6.47f)
                curveToRelative(0.27f, 0.27f, 0.3f, 0.68f, 0.07f, 0.98f)
                lineToRelative(-0.07f, 0.08f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -0.98f, 0.07f)
                lineToRelative(-0.08f, -0.07f)
                lineTo(12.0f, 13.06f)
                lineToRelative(-6.47f, 6.47f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -1.06f, -1.06f)
                lineTo(10.94f, 12.0f)
                lineTo(4.47f, 5.53f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -0.07f, -0.98f)
                lineToRelative(0.07f, -0.08f)
                lineToRelative(-0.07f, 0.08f)
                close()
            }
        }.build()

val ArrowLeftIcon: ImageVector =
    ImageVector
        .Builder(
            name = "arrowleft",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                fill = SolidColor(Color.Black),
                fillAlpha = 1f,
                stroke = null,
                strokeAlpha = 1f,
                strokeLineWidth = 1f,
                strokeLineCap = StrokeCap.Butt,
                strokeLineJoin = StrokeJoin.Bevel,
                strokeLineMiter = 1f,
                pathFillType = PathFillType.Companion.NonZero,
            ) {
                moveTo(10.3f, 19.72f)
                arcToRelative(1.0f, 1.0f, 0.0f, false, false, 1.4f, -1.43f)
                lineTo(6.33f, 13.0f)
                horizontalLineTo(20.0f)
                arcToRelative(1.0f, 1.0f, 0.0f, false, false, 0.0f, -2.0f)
                horizontalLineTo(6.33f)
                lineToRelative(5.37f, -5.28f)
                arcToRelative(1.0f, 1.0f, 0.0f, false, false, -1.4f, -1.42f)
                lineToRelative(-6.93f, 6.82f)
                curveToRelative(-0.5f, 0.5f, -0.5f, 1.3f, 0.0f, 1.78f)
                lineToRelative(6.92f, 6.83f)
                close()
            }
        }.build()

val ExitFullscreenIcon: ImageVector =
    ImageVector
        .Builder(
            name = "exitfullscreenicon",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                fill = SolidColor(Color.Black),
                fillAlpha = 1f,
                stroke = null,
                strokeAlpha = 1f,
                strokeLineWidth = 1f,
                strokeLineCap = StrokeCap.Butt,
                strokeLineJoin = StrokeJoin.Bevel,
                strokeLineMiter = 1f,
                pathFillType = PathFillType.Companion.NonZero,
            ) {
                moveTo(21.78f, 2.22f)
                curveToRelative(0.26f, 0.27f, 0.29f, 0.68f, 0.07f, 0.98f)
                lineToRelative(-0.07f, 0.08f)
                lineToRelative(-6.23f, 6.23f)
                horizontalLineToRelative(5.7f)
                curveToRelative(0.38f, 0.0f, 0.7f, 0.28f, 0.74f, 0.65f)
                verticalLineToRelative(0.1f)
                curveToRelative(0.0f, 0.38f, -0.27f, 0.69f, -0.64f, 0.74f)
                horizontalLineTo(13.68f)
                arcToRelative(0.73f, 0.73f, 0.0f, false, true, -0.2f, -0.04f)
                lineToRelative(-0.1f, -0.04f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -0.38f, -0.56f)
                verticalLineToRelative(-7.6f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, 1.49f, -0.1f)
                verticalLineTo(8.43f)
                lineToRelative(6.23f, -6.22f)
                curveToRelative(0.29f, -0.3f, 0.76f, -0.3f, 1.06f, 0.0f)
                close()
                moveTo(11.0f, 13.75f)
                verticalLineToRelative(7.5f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -1.5f, 0.11f)
                verticalLineToRelative(-5.8f)
                lineToRelative(-6.22f, 6.22f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -1.13f, -0.98f)
                lineToRelative(0.07f, -0.08f)
                lineToRelative(6.22f, -6.22f)
                horizontalLineTo(2.75f)
                arcToRelative(0.75f, 0.75f, 0.0f, false, true, -0.74f, -0.64f)
                verticalLineToRelative(-0.1f)
                curveToRelative(0.0f, -0.42f, 0.33f, -0.76f, 0.74f, -0.76f)
                horizontalLineToRelative(7.56f)
                lineToRelative(0.07f, 0.01f)
                lineToRelative(0.1f, 0.03f)
                lineToRelative(0.05f, 0.02f)
                lineToRelative(0.09f, 0.04f)
                lineToRelative(0.08f, 0.06f)
                curveToRelative(0.06f, 0.04f, 0.11f, 0.09f, 0.15f, 0.14f)
                lineToRelative(0.07f, 0.1f)
                lineToRelative(0.04f, 0.1f)
                lineToRelative(0.02f, 0.07f)
                lineToRelative(0.01f, 0.06f)
                verticalLineToRelative(0.06f)
                verticalLineToRelative(-0.01f)
                lineToRelative(0.01f, 0.07f)
                close()
            }
        }.build()

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun FrameWindowScope.WindowsWindowFrame(
    onCloseRequest: () -> Unit,
    state: WindowState,
    captionBarHeight: Dp = 32.dp,
    darkTheme: Boolean,
    content: @Composable (
        windowInset: WindowInsets,
        captionBarInset: WindowInsets,
        onMaximized: () -> Unit,
        toggleFullscreen: () -> Unit,
    ) -> Unit,
) {
    LaunchedEffect(window) {
        window.findSkiaLayer()?.transparency = true
    }
    // Won't need mica, since Material

    /*WindowStyle(
        isDarkTheme = FluentTheme.colors.darkMode,
        backdropType = when {
            isWindows11OrLater() -> WindowBackdrop.Mica
            else -> WindowBackdrop.Solid(FluentTheme.colors.background.mica.baseAlt)
        }
    )*/

    val paddingInset = remember { MutableWindowInsets() }
    val maxButtonRect = remember { mutableStateOf(Rect.Zero) }
    val minButtonRect = remember { mutableStateOf(Rect.Zero) }
    val closeButtonRect = remember { mutableStateOf(Rect.Zero) }
    val captionBarRect = remember { mutableStateOf(Rect.Zero) }
    val layoutHitTestOwner = rememberLayoutHitTestOwner()
    val contentPaddingInset = remember { MutableWindowInsets() }

    //TODO: Use this somewhere.
    val isFullscreen = false

    val procedure =
        remember(window) {
            ComposeWindowProcedure(
                window = window,
                hitTest = { x, y ->
                    when {
                        maxButtonRect.value.contains(x, y) -> HTMAXBUTTON
                        minButtonRect.value.contains(x, y) -> HTMINBUTTON
                        closeButtonRect.value.contains(x, y) -> HTCLOSE
                        captionBarRect.value.contains(x, y) && !layoutHitTestOwner.hitTest(x, y) -> HTCAPTION
                        else -> HTCLIENT
                    }
                },
                onWindowInsetUpdate = { paddingInset.insets = it },
            )
        }

    val oldStyle = remember { mutableIntStateOf(0) }
    val oldPlacement = remember { mutableStateOf(WinUser.WINDOWPLACEMENT()) }

    fun windowsToggleFullscreen() {
        val user32 = User32.INSTANCE
        if (!isFullscreen) {

            procedure.isFullscreen = false

            user32.SetWindowLong(procedure.windowHandle, WinUser.GWL_STYLE, oldStyle.intValue)
            user32.SetWindowPlacement(procedure.windowHandle, oldPlacement.value)

            if (oldPlacement.value.showCmd == WinUser.SW_SHOWMAXIMIZED) {
                user32.ShowWindow(
                    procedure.windowHandle,
                    WinUser.SW_RESTORE,
                )
                user32.ShowWindow(
                    procedure.windowHandle,
                    WinUser.SW_MAXIMIZE,
                )
            }

            state.placement =
                if (oldPlacement.value.showCmd == WinUser.SW_SHOWMAXIMIZED) {
                    WindowPlacement.Maximized
                } else {
                    WindowPlacement.Floating
                }
        } else {
            procedure.isFullscreen = true
            oldStyle.value = user32.GetWindowLong(procedure.windowHandle, WinUser.GWL_STYLE)
            user32.GetWindowPlacement(procedure.windowHandle, oldPlacement.value)

            val newStyle = oldStyle.value and (WinUser.WS_CAPTION or WinUser.WS_THICKFRAME).inv()
            user32.SetWindowLong(procedure.windowHandle, WinUser.GWL_STYLE, newStyle)

            val monitor = user32.MonitorFromWindow(procedure.windowHandle, WinUser.MONITOR_DEFAULTTONEAREST)
            val monitorInfo = WinUser.MONITORINFO().apply { cbSize = size() }
            user32.GetMonitorInfo(monitor, monitorInfo)
            val rc = monitorInfo.rcMonitor

            user32.SetWindowPos(
                procedure.windowHandle,
                null,
                rc.left,
                rc.top,
                rc.right - rc.left,
                rc.bottom - rc.top,
                WinUser.SWP_NOZORDER or WinUser.SWP_FRAMECHANGED,
            )
        }
    }

    Box(
        // Weird ass padding bug
        modifier =
            Modifier
                .windowInsetsPadding(if (state.placement == WindowPlacement.Maximized) paddingInset else WindowInsets()),
    ) {
        val hoveringFullscreen = remember { mutableStateOf(false) }
        content(
            WindowInsets(top = captionBarHeight),
            contentPaddingInset,
            {},
            ::windowsToggleFullscreen,
        )
        if (isFullscreen) {
            CaptionButton(
                onClick = {
                    windowsToggleFullscreen()
                },
                slightlyBigger = true,
                modifier =
                    Modifier.pointerInput(Unit) {
                        awaitPointerEventScope {
                            while (true) {
                                val event = awaitPointerEvent()
                                when (event.type) {
                                    PointerEventType.Enter -> {
                                        hoveringFullscreen.value = true
                                    }

                                    PointerEventType.Exit -> {
                                        hoveringFullscreen.value = false
                                    }
                                }
                            }
                        }
                    },
                icon = CaptionButtonIcon.ArrowLeft,
                isActive = true,
                darkTheme = darkTheme,
                isCloseButton = false,
            )
        }
        val offsetCondition = remember { mutableStateOf(false) }
        val offset = animateDpAsState(if (offsetCondition.value) -captionBarHeight else 0.dp)
        LaunchedEffect(isFullscreen && !hoveringFullscreen.value) {
            if (isFullscreen && !hoveringFullscreen.value) {
                delay(1000.milliseconds)
                offsetCondition.value = true
            } else {
                offsetCondition.value = false
            }
        }

        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier =
                Modifier
                    .height(captionBarHeight)
                    .offset(x = 0.dp, y = offset.value)
                    .zIndex(99999f)
                    .onGloballyPositioned { captionBarRect.value = it.boundsInWindow() },
        ) {
            Spacer(modifier = Modifier.weight(1f))
            window.CaptionButtonRow(
                windowHandle = procedure.windowHandle,
                state = state,
                isMaximize = state.placement == WindowPlacement.Maximized,
                onCloseRequest = onCloseRequest,
                onMaximizeButtonRectUpdate = {
                    maxButtonRect.value = it
                },
                onMinimizeButtonRectUpdate = {
                    minButtonRect.value = it
                },
                onCloseButtonRectUpdate = {
                    closeButtonRect.value = it
                },
                toggleFullscreen = { windowsToggleFullscreen() },
                accentColor = procedure.windowFrameColor,
                frameColorEnabled = procedure.isWindowFrameAccentColorEnabled,
                isActive = procedure.isWindowActive,
                darkTheme = darkTheme,
                modifier =
                    Modifier
                        .align(Alignment.Top)
                        .onSizeChanged {
                            contentPaddingInset.insets = WindowInsets(right = it.width, top = it.height)
                        }.pointerInput(Unit) {
                            awaitPointerEventScope {
                                while (true) {
                                    val event = awaitPointerEvent()
                                    when (event.type) {
                                        PointerEventType.Enter -> {
                                            hoveringFullscreen.value = true
                                        }

                                        PointerEventType.Exit -> {
                                            hoveringFullscreen.value = false
                                        }
                                    }
                                }
                            }
                        },
                isFullscreen = isFullscreen
            )
        }
        if (isFullscreen) {
            Box(
                modifier =
                    Modifier
                        .height(4.dp)
                        .fillMaxWidth()
                        .pointerInput(Unit) {
                            awaitPointerEventScope {
                                while (true) {
                                    val event = awaitPointerEvent()
                                    when (event.type) {
                                        PointerEventType.Enter -> {
                                            hoveringFullscreen.value = true
                                        }

                                        PointerEventType.Exit -> {
                                            hoveringFullscreen.value = false
                                        }
                                    }
                                }
                            }
                        },
            )
        }
    }
}

@Composable
fun Window.CaptionButtonRow(
    windowHandle: HWND,
    state: WindowState,
    isMaximize: Boolean,
    isActive: Boolean,
    darkTheme: Boolean,
    accentColor: Color,
    frameColorEnabled: Boolean,
    onCloseRequest: () -> Unit,
    modifier: Modifier = Modifier,
    toggleFullscreen: () -> Unit,
    onMaximizeButtonRectUpdate: (Rect) -> Unit,
    onMinimizeButtonRectUpdate: (Rect) -> Unit = {},
    onCloseButtonRectUpdate: (Rect) -> Unit = {},
    isFullscreen: Boolean
) {
    // Draw the caption button
    Row(
        modifier =
            modifier
                .zIndex(1f),
    ) {
        CaptionButton(
            onClick = {
                User32.INSTANCE.ShowWindow(windowHandle, WinUser.SW_MINIMIZE)
            },
            icon = CaptionButtonIcon.Minimize,
            isActive = isActive,
            isCloseButton = false,
            darkTheme = darkTheme,
            modifier =
                Modifier.onGloballyPositioned {
                    onMinimizeButtonRectUpdate(it.boundsInWindow())
                },
        )
        CaptionButton(
            onClick = {
                if (isMaximize) {
                    User32.INSTANCE.ShowWindow(
                        windowHandle,
                        WinUser.SW_RESTORE,
                    )
                    state.placement = WindowPlacement.Floating
                } else {
                    User32.INSTANCE.ShowWindow(
                        windowHandle,
                        WinUser.SW_MAXIMIZE,
                    )
                    state.placement = WindowPlacement.Maximized
                }
            },
            slightlyBigger = isFullscreen,
            darkTheme = darkTheme,
            icon =
                if (isFullscreen) {
                    CaptionButtonIcon.ExitFullscreen
                } else {
                    if (isMaximize) {
                        CaptionButtonIcon.Restore
                    } else {
                        CaptionButtonIcon.Maximize
                    }
                },
            isActive = isActive,
            isCloseButton = false,
            modifier =
                Modifier.onGloballyPositioned {
                    onMaximizeButtonRectUpdate(it.boundsInWindow())
                },
        )
        CaptionButton(
            icon = CaptionButtonIcon.Close,
            onClick = onCloseRequest,
            isActive = isActive,
            darkTheme = darkTheme,
            isCloseButton = true,
            modifier =
                Modifier.onGloballyPositioned {
                    onCloseButtonRectUpdate(it.boundsInWindow())
                },
        )
    }
}

@Composable
fun CaptionButton(
    onClick: () -> Unit,
    icon: CaptionButtonIcon,
    isActive: Boolean,
    slightlyBigger: Boolean = false,
    modifier: Modifier = Modifier,
    isCloseButton: Boolean,
    interaction: MutableInteractionSource = remember { MutableInteractionSource() },
    darkTheme: Boolean = isSystemInDarkTheme(),
) {
    var hovering by remember { mutableStateOf(false) }
    var pressing by remember { mutableStateOf(false) }
    Box(
        modifier =
            modifier
                .size(46.dp, 32.dp)
                .clickable(
                    onClick = onClick,
                    interactionSource = interaction,
                    indication = null,
                ).pointerInput(Unit) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            when (event.type) {
                                PointerEventType.Enter -> {
                                    hovering = true
                                }

                                PointerEventType.Exit -> {
                                    hovering = false
                                }

                                PointerEventType.Press -> {
                                    pressing = true
                                }

                                PointerEventType.Release -> {
                                    pressing = false
                                }
                            }
                        }
                    }
                }.background(
                    if (hovering) {
                        if (isCloseButton) {
                            if (pressing) Color(0xFFB3271E) else Color(0xFFC42B1C)
                        } else {
                            if (darkTheme) {
                                Color.White.copy(if (pressing) 0.1f else 0.05f)
                            } else {
                                Color.Black.copy(if (pressing) 0.1f else 0.05f)
                            }
                        }
                    } else {
                        Color.Transparent
                    },
                ),
    ) {
        val fontFamily by rememberFontIconFamily()
        if (fontFamily != null) {
            Text(
                text = icon.glyph.toString(),
                color =
                    if (darkTheme) {
                        if (isActive) {
                            if (pressing) Color.White.copy(0.7f) else Color.White
                        } else {
                            Color.White.copy(0.5f)
                        }
                    } else {
                        if (isActive) {
                            if (pressing) Color.Black.copy(0.7f) else Color.Black
                        } else {
                            Color.Black.copy(0.5f)
                        }
                    },
                fontFamily = fontFamily,
                textAlign = TextAlign.Center,
                fontSize = if (slightlyBigger) 12.sp else 10.sp,
                modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center),
            )
        } else {
            Icon(
                imageVector = icon.imageVector,
                contentDescription = null,
                tint =
                    if (darkTheme) {
                        if (isActive) {
                            if (pressing) Color.White.copy(0.7f) else Color.White
                        } else {
                            Color.White.copy(0.5f)
                        }
                    } else {
                        if (isActive) {
                            if (pressing) Color.Black.copy(0.7f) else Color.Black
                        } else {
                            Color.Black.copy(0.5f)
                        }
                    },
                modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center).size(13.dp),
            )
        }
    }
}

@OptIn(ExperimentalTextApi::class)
@Composable
private fun rememberFontIconFamily(): State<FontFamily?> {
    val fontIconFamily = remember { mutableStateOf<FontFamily?>(null) }
    // Get windows system font icon, if get failed fall back to fluent svg icon.
    val fontFamilyResolver = LocalFontFamilyResolver.current
    LaunchedEffect(fontFamilyResolver) {
        fontIconFamily.value =
            sequenceOf("Segoe Fluent Icons", "Segoe MDL2 Assets").firstNotNullOfOrNull {
                val fontFamily = FontFamily(it)
                runCatching {
                    val result = fontFamilyResolver.resolve(fontFamily).value as FontLoadResult
                    if (result.typeface == null || result.typeface?.familyName != it) {
                        null
                    } else {
                        fontFamily
                    }
                }.getOrNull()
            }
    }
    return fontIconFamily
}

enum class CaptionButtonIcon(
    val glyph: Char,
    val imageVector: ImageVector,
) {
    Minimize(
        glyph = '\uE921',
        imageVector = MinimizeIcon,
    ),
    Maximize(
        glyph = '\uE922',
        imageVector = MaximizeIcon,
    ),
    Restore(
        glyph = '\uE923',
        imageVector = RestoreIcon,
    ),
    Close(
        glyph = '\uE8BB',
        imageVector = CloseIcon,
    ),
    ArrowLeft(
        glyph = '\uE72B',
        imageVector = ArrowLeftIcon,
    ),
    ExitFullscreen(
        glyph = '\uE73F',
        imageVector = ExitFullscreenIcon,
    ),
}

fun Rect.contains(
    x: Float,
    y: Float,
): Boolean = x in left..<right && y >= top && y < bottom
