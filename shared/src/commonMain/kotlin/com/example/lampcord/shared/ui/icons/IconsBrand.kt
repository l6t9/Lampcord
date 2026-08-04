package com.example.lampcord.shared.ui.icons

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.dp

/**
 * Brand icons for integrations and special UI elements.
 * These are custom icons sourced from drawable XML resources and Simple Icons,
 * converted to ImageVector format.
 */
object IconsBrand {
    val Discord: ImageVector
        get() =
            _Discord ?: ImageVector
                .Builder(
                    name = "Discord",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M20.317 4.3698a19.7913 19.7913 0 0 0-4.8851-1.5152.0741.0741 0 0 0-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 0 0-.0785-.037 19.7363 19.7363 0 0 0-4.8852 1.515.0699.0699 0 0 0-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 0 0 .0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 0 0 .0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 0 0-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 0 1-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 0 1 .0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 0 1 .0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 0 1-.0066.1276 12.2986 12.2986 0 0 1-1.873.8914.0766.0766 0 0 0-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 0 0 .0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 0 0 .0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 0 0-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _Discord = it }
    private var _Discord: ImageVector? = null

    val LastFm: ImageVector
        get() =
            _LastFm ?: ImageVector
                .Builder(
                    name = "LastFm",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 512f,
                    viewportHeight = 512f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M308.214,337.861l-5.663-13.064L253.93,209.107c-16.056-40.931-56.085-68.601-101.198-68.601c-61.043,0-110.576,51.706-110.576,115.524c0,63.756,49.533,115.493,110.576,115.493c42.618,0,79.604-25.164,98.062-62.007l19.668,47.329c-27.876,35.526-70.298,58.155-117.729,58.155C68.645,415.002,0.5,343.886,0.5,256.031c0-87.834,68.145-159.033,152.231-159.033c63.446,0,114.696,35.361,140.741,98.093c1.946,4.865,27.516,67.255,49.834,120.369c13.788,32.856,25.537,54.678,63.776,56.023c37.441,1.325,63.249-22.484,63.249-52.648c0-29.45-19.7-36.542-52.825-48.042c-59.543-20.486-90.308-41.065-90.308-90.401c0-48.115,31.303-80.205,82.295-80.205c33.137,0,57.162,15.424,73.756,46.169l-32.618,17.37c-12.235-17.909-25.765-25-42.97-25c-23.934,0-40.94,17.381-40.94,40.465c0,32.805,28.095,37.742,67.348,51.179c52.866,17.981,77.431,38.529,77.431,89.801c0,53.86-44.232,93.093-102.006,93.01C356.256,412.942,327.861,385.769,308.214,337.861z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _LastFm = it }
    private var _LastFm: ImageVector? = null

    val Token: ImageVector
        get() =
            _Token ?: ImageVector
                .Builder(
                    name = "Token",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M21,7l-9-5L3,7v10l9,5l9-5L21,7z M12,4.29l5.91,3.28L14.9,9.24C14.17,8.48,13.14,8,12,8S9.83,8.48,9.1,9.24L6.09,7.57 L12,4.29z M11,19.16l-6-3.33V9.26l3.13,1.74C8.04,11.31,8,11.65,8,12c0,1.86,1.27,3.43,3,3.87V19.16z M10,12c0-1.1,0.9-2,2-2 s2,0.9,2,2s-0.9,2-2,2S10,13.1,10,12z M13,19.16v-3.28c1.73-0.44,3-2.01,3-3.87c0-0.35-0.04-0.69-0.13-1.01L19,9.26l0,6.57L13,19.16 z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _Token = it }
    private var _Token: ImageVector? = null

    val Github: ImageVector
        get() =
            _Github ?: ImageVector
                .Builder(
                    name = "Github",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61-.546-1.385-1.335-1.755-1.335-1.755-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _Github = it }
    private var _Github: ImageVector? = null

    val Telegram: ImageVector
        get() =
            _Telegram ?: ImageVector
                .Builder(
                    name = "Telegram",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _Telegram = it }
    private var _Telegram: ImageVector? = null

    val Instagram: ImageVector
        get() =
            _Instagram ?: ImageVector
                .Builder(
                    name = "Instagram",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zM12 0C8.741 0 8.333.014 7.053.072 2.695.272.273 2.69.073 7.052.014 8.333 0 8.741 0 12c0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98C15.668.014 15.259 0 12 0zm0 5.838a6.162 6.162 0 1 0 0 12.324 6.162 6.162 0 0 0 0-12.324zM12 16a4.162 4.162 0 1 1 0-8.324A4.162 4.162 0 0 1 12 16zm6.406-11.845a1.44 1.44 0 1 0 0 2.881 1.44 1.44 0 0 0 0-2.881z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _Instagram = it }
    private var _Instagram: ImageVector? = null

    val BuyMeACoffee: ImageVector
        get() =
            _BuyMeACoffee ?: ImageVector
                .Builder(
                    name = "BuyMeACoffee",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M20.2 7c-.4-1.2-1.3-2.1-2.5-2.5-1.1-.3-2.3-.1-3.2.5-.8.5-1.3 1.3-1.4 2.2H5c-1.1 0-2.1.4-2.8 1.2-.8.8-1.2 1.8-1.2 2.8 0 1.1.4 2.1 1.2 2.8.8.8 1.8 1.2 2.8 1.2h10c.8 0 1.5-.3 2.1-.8.6-.6.9-1.3.9-2.1V7.2c0-.1 0-.1-.1-.2zm-3.2 5c0 .6-.4 1-1 1H5c-.6 0-1-.4-1-1s.4-1 1-1h11c.6 0 1 .4 1 1zm2-5v3.1c0 .1-.1.2-.2.2h-1c-.1 0-.2-.1-.2-.2V7.1c0-.4.3-.8.7-.8s.7.4.7.7z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _BuyMeACoffee = it }
    private var _BuyMeACoffee: ImageVector? = null

    val Youtube: ImageVector
        get() =
            _Youtube ?: ImageVector
                .Builder(
                    name = "Youtube",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                ).apply {
                    addPath(
                        pathData =
                            PathParser()
                                .parsePathString(
                                    "M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.016 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z",
                                ).toNodes(),
                        fill = SolidColor(Color.Black),
                    )
                }.build()
                .also { _Youtube = it }
    private var _Youtube: ImageVector? = null
}
