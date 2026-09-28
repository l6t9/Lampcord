@file:Suppress("ktlint:standard:max-line-length")

package me.lampu.lampcord.shared.ui.icons

import androidx.compose.ui.graphics.vector.ImageVector

object Icons {
    val Filled: IconsRoundedFilled = IconsRoundedFilled
    val Default: IconsRounded = IconsRounded
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
            val Keyboard: ImageVector by lazy {
                materialSymbol(
                    "Rounded_filled.Keyboard",
                    "M800-200H160C116-200 80-164 80-120V-680C80-724 116-760 160-760H800C844-760 880-724 880-680V-280C880-236 844-200 800-200ZM440-320H520V-400H440V-320ZM440-440H520V-520H440V-440ZM320-320H400V-400H320V-320ZM320-440H400V-520H320V-440ZM280-440H200V-520H280V-440ZM280-320H200V-400H280V-320ZM600-560H360C338-560 320-578 320-600C320-622 338-640 360-640H600C622-640 640-622 640-600C640-578 622-560 600-560ZM640-320H560V-400H640V-320ZM640-440H560V-520H640V-440ZM760-320H680V-400H760V-320ZM760-440H680V-520H760V-440Z"
                )
            }
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
        
        val Default = Rounded
    }
}
