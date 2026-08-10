@file:Suppress("ktlint:standard:max-line-length")

package me.lampu.lampcord.shared.ui.icons

import androidx.compose.ui.graphics.vector.ImageVector

/** Material Symbols used throughout the Android, shared, and desktop UI. */
object Icons {
    val Filled: IconsRoundedFilled = IconsRoundedFilled
    val Default: IconsRounded = IconsRounded
    val Outlined: IconsRounded = IconsRounded
    val Rounded: IconsRounded = IconsRounded
    val Brand: IconsBrand = IconsBrand

    object AutoMirrored {
        object Rounded {
            val ArrowBack: ImageVector by lazy { materialSymbol("Rounded.ArrowBack", IconsRounded.ArrowBack_Path, autoMirror = true) }
            val ArrowForward: ImageVector by lazy { materialSymbol("Rounded.ArrowForward", IconsRounded.ArrowForward_Path, autoMirror = true) }
            val KeyboardArrowLeft: ImageVector by lazy { materialSymbol("Rounded.ChevronLeft", IconsRounded.ChevronLeft_Path, autoMirror = true) }
            val KeyboardArrowRight: ImageVector by lazy { materialSymbol("Rounded.ChevronRight", IconsRounded.ChevronRight_Path, autoMirror = true) }
            val List: ImageVector by lazy { materialSymbol("Rounded.List", IconsRounded.List_Path, autoMirror = true) }
            val Login: ImageVector by lazy { materialSymbol("Rounded.Login", IconsRounded.Login_Path, autoMirror = true) }
            val Logout: ImageVector by lazy { materialSymbol("Rounded.Logout", IconsRounded.Logout_Path, autoMirror = true) }
            val MenuOpen: ImageVector by lazy { materialSymbol("Rounded.MenuOpen", IconsRounded.MenuOpen_Path, autoMirror = true) }
            val OpenInNew: ImageVector by lazy { materialSymbol("Rounded.OpenInNew", IconsRounded.OpenInNew_Path, autoMirror = true) }
            val PlaylistAdd: ImageVector by lazy { materialSymbol("Rounded.PlaylistAdd", IconsRounded.PlaylistAdd_Path, autoMirror = true) }
            val Send: ImageVector by lazy { materialSymbol("Rounded.Send", IconsRounded.Send_Path, autoMirror = true) }
            val VolumeDown: ImageVector by lazy { materialSymbol("Rounded.VolumeDown", IconsRounded.VolumeDown_Path, autoMirror = true) }
            val VolumeMute: ImageVector by lazy { materialSymbol("Rounded.VolumeMute", IconsRounded.VolumeMute_Path, autoMirror = true) }
            val VolumeOff: ImageVector by lazy { materialSymbol("Rounded.VolumeOff", IconsRounded.VolumeOff_Path, autoMirror = true) }
            val VolumeUp: ImageVector by lazy { materialSymbol("Rounded.VolumeUp", IconsRounded.VolumeUp_Path, autoMirror = true) }
        }

        object Filled {
            val ArrowBack: ImageVector by lazy { materialSymbol("Rounded_filled.ArrowBack", IconsRoundedFilled.ArrowBack_Path, autoMirror = true) }
            val ArrowForward: ImageVector by lazy { materialSymbol("Rounded_filled.ArrowForward", IconsRoundedFilled.ArrowForward_Path, autoMirror = true) }
            val KeyboardArrowLeft: ImageVector by lazy { materialSymbol("Rounded_filled.ChevronLeft", IconsRoundedFilled.ChevronLeft_Path, autoMirror = true) }
            val KeyboardArrowRight: ImageVector by lazy { materialSymbol("Rounded_filled.ChevronRight", IconsRoundedFilled.ChevronRight_Path, autoMirror = true) }
            val List: ImageVector by lazy { materialSymbol("Rounded_filled.List", IconsRoundedFilled.List_Path, autoMirror = true) }
            val Login: ImageVector by lazy { materialSymbol("Rounded_filled.Login", IconsRoundedFilled.Login_Path, autoMirror = true) }
            val Logout: ImageVector by lazy { materialSymbol("Rounded_filled.Logout", IconsRoundedFilled.Logout_Path, autoMirror = true) }
            val MenuOpen: ImageVector by lazy { materialSymbol("Rounded_filled.MenuOpen", IconsRoundedFilled.MenuOpen_Path, autoMirror = true) }
            val OpenInNew: ImageVector by lazy { materialSymbol("Rounded_filled.OpenInNew", IconsRoundedFilled.OpenInNew_Path, autoMirror = true) }
            val PlaylistAdd: ImageVector by lazy { materialSymbol("Rounded_filled.PlaylistAdd", IconsRoundedFilled.PlaylistAdd_Path, autoMirror = true) }
            val Send: ImageVector by lazy { materialSymbol("Rounded_filled.Send", IconsRoundedFilled.Send_Path, autoMirror = true) }
            val VolumeDown: ImageVector by lazy { materialSymbol("Rounded_filled.VolumeDown", IconsRoundedFilled.VolumeDown_Path, autoMirror = true) }
            val VolumeMute: ImageVector by lazy { materialSymbol("Rounded_filled.VolumeMute", IconsRoundedFilled.VolumeMute_Path, autoMirror = true) }
            val VolumeOff: ImageVector by lazy { materialSymbol("Rounded_filled.VolumeOff", IconsRoundedFilled.VolumeOff_Path, autoMirror = true) }
            val VolumeUp: ImageVector by lazy { materialSymbol("Rounded_filled.VolumeUp", IconsRoundedFilled.VolumeUp_Path, autoMirror = true) }
        }
        
        val Outlined = Rounded
        val Default = Rounded
    }
}
