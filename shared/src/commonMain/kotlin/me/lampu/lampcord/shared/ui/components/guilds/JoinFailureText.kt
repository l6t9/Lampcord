package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.api.JoinFailure
import me.lampu.lampcord.shared.api.JoinGuildResult
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_banned
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_captcha_cancelled
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_captcha_rejected
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_forbidden
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_invalid_invite
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_join_restricted
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_no_access
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_no_permission
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_no_server
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_not_found
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_rate_limited
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_region
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_rejected_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_server_full
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_server_limit
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_unauthorized
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_unavailable_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_unreachable
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_verification_level
import me.lampu.lampcord.shared.generated.resources.lampcord_join_error_verify_account
import org.jetbrains.compose.resources.stringResource

@Composable
internal fun joinFailureText(failed: JoinGuildResult.Failed): String {
    val detail = failed.detail.orEmpty()
    return when (failed.failure) {
        JoinFailure.CaptchaCancelled -> stringResource(Res.string.lampcord_join_error_captcha_cancelled)
        JoinFailure.CaptchaRejected -> stringResource(Res.string.lampcord_join_error_captcha_rejected)
        JoinFailure.NoServerReturned -> stringResource(Res.string.lampcord_join_error_no_server)
        JoinFailure.Unreachable -> stringResource(Res.string.lampcord_join_error_unreachable)
        JoinFailure.ServerLimit -> stringResource(Res.string.lampcord_join_error_server_limit)
        JoinFailure.ServerFull -> stringResource(Res.string.lampcord_join_error_server_full)
        JoinFailure.JoinRestricted -> stringResource(Res.string.lampcord_join_error_join_restricted)
        JoinFailure.InvalidInvite -> stringResource(Res.string.lampcord_join_error_invalid_invite)
        JoinFailure.Banned -> stringResource(Res.string.lampcord_join_error_banned)
        JoinFailure.VerificationLevel -> stringResource(Res.string.lampcord_join_error_verification_level)
        JoinFailure.NoPermission -> stringResource(Res.string.lampcord_join_error_no_permission)
        JoinFailure.NoAccess -> stringResource(Res.string.lampcord_join_error_no_access)
        JoinFailure.RegionBlocked -> stringResource(Res.string.lampcord_join_error_region)
        JoinFailure.VerifyAccount -> stringResource(Res.string.lampcord_join_error_verify_account)
        JoinFailure.RateLimited -> stringResource(Res.string.lampcord_join_error_rate_limited)
        JoinFailure.DiscordUnavailable -> stringResource(Res.string.lampcord_join_error_unavailable_fmt, detail)
        JoinFailure.Unauthorized -> stringResource(Res.string.lampcord_join_error_unauthorized)
        JoinFailure.Forbidden -> stringResource(Res.string.lampcord_join_error_forbidden)
        JoinFailure.NotFound -> stringResource(Res.string.lampcord_join_error_not_found)
        JoinFailure.Rejected -> stringResource(Res.string.lampcord_join_error_rejected_fmt, detail)
        JoinFailure.ServerMessage -> detail
    }
}
