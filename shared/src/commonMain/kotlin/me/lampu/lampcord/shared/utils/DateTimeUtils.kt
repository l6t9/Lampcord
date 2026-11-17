package me.lampu.lampcord.shared.utils

import kotlinx.datetime.*
import kotlinx.datetime.format.DateTimeComponents
import kotlinx.datetime.format.DateTimeFormat
import kotlinx.datetime.format.DayOfWeekNames
import kotlinx.datetime.format.FormatStringsInDatetimeFormats
import kotlinx.datetime.format.MonthNames
import kotlinx.datetime.format.Padding
import kotlinx.datetime.format.byUnicodePattern
import kotlinx.datetime.number
import me.lampu.lampcord.shared.settings.Settings
import kotlin.time.Clock
import kotlin.time.Instant

object DateTimeUtils {
//    fun formatTimestamp(isoString: String): String {
//        return try {
//            val instant = Instant.parse(isoString)
//            val now = Clock.System.now()
//            val timeZone = TimeZone.currentSystemDefault()
//            val localDateTime = instant.toLocalDateTime(timeZone)
//            val nowDate = now.toLocalDateTime(timeZone).date
//
//            val hour = localDateTime.hour
//            val minute = localDateTime.minute.toString().padStart(2, '0')
//            val amPm = if (hour >= 12) "PM" else "AM"
//            val displayHour = when {
//                hour == 0 -> 12
//                hour > 12 -> hour - 12
//                else -> hour
//            }
//            val timeString = "$displayHour:$minute $amPm"
//
//            val date = localDateTime.date
//            when {
//                date == nowDate -> timeString
//                date.dayOfYear == nowDate.dayOfYear - 1 && date.year == nowDate.year -> "Yesterday at $timeString"
//                else -> {
//                    val month = date.month.number
//                    val day = date.day
//                    val year = date.year.toString().takeLast(2)
//                    "$month/$day/$year, $timeString"
//                }
//            }
//        } catch (e: Exception) {
//            isoString.take(10)
//        }
//    }

    @OptIn(FormatStringsInDatetimeFormats::class)
    @Throws
    fun formatTimePattern(pattern: String): DateTimeFormat<LocalTime> {
        require(pattern.isNotBlank())

        val regex = Regex("('[^']*'|hh|h|a|[^'ha]+)")

        return LocalTime.Format {
            regex.findAll(pattern).forEach { match ->
                when (val token = match.value) {
                    "hh" -> amPmHour(Padding.ZERO)
                    "h"  -> amPmHour(Padding.NONE)
                    "a"  -> amPmMarker("AM", "PM")
                    else -> byUnicodePattern(token)
                }
            }
        }
    }

    @OptIn(FormatStringsInDatetimeFormats::class)
    @Throws
    fun formatDateTimePattern(
        pattern: String,
        monthNamesFull: MonthNames = MonthNames.ENGLISH_FULL,
        monthNamesAbbreviated: MonthNames = MonthNames.ENGLISH_ABBREVIATED,
        dayWeekFull: DayOfWeekNames = DayOfWeekNames.ENGLISH_FULL,
        dayWeekAbbreviated: DayOfWeekNames = DayOfWeekNames.ENGLISH_ABBREVIATED,
    ): DateTimeFormat<LocalDateTime> {
        require(pattern.isNotBlank())

        val regex = Regex("""'[^']*'|MMMM|MMM|EEEE|EEE|hh|h|a|(?:(?!'|MMMM|MMM|EEEE|EEE|hh|h|a).)+""")

        return LocalDateTime.Format {
            regex.findAll(pattern).forEach { match ->
                when (val token = match.value) {
                    "MMMM" -> monthName(monthNamesFull)
                    "MMM"  -> monthName(monthNamesAbbreviated)
                    "EEEE" -> dayOfWeek(dayWeekFull)
                    "EEE"  -> dayOfWeek(dayWeekAbbreviated)
                    "hh" -> amPmHour(Padding.ZERO)
                    "h"  -> amPmHour(Padding.NONE)
                    "a"  -> amPmMarker("AM", "PM")
                    else -> byUnicodePattern(token)
                }
            }
        }
    }

    fun formatTime(time: LocalTime, pattern: String): Result<String> {
        val format = runCatching {formatTimePattern(pattern)}

        return format.mapCatching {format -> time.format(format)}
    }

    fun formatDateTime(dateTime: LocalDateTime, pattern: String): Result<String> {
        val format = runCatching {formatDateTimePattern(pattern)}

        return format.mapCatching {f -> dateTime.format(f)}
    }

    fun formatTimestamp(isoString: String): String {
        val instant = Instant.parse(isoString)
        val rightNow = Clock.System.now()

        val timezone = TimeZone.currentSystemDefault()
        val localDateTime = instant.toLocalDateTime(timezone)

        val localRightNow = rightNow.toLocalDateTime(timezone)

        val timeFormat = Settings.shared.timeFormat

        return when {
            localDateTime.day == localRightNow.day -> {
                formatTime(localDateTime.time, timeFormat).fold(
                    onSuccess = { f -> f },
                    onFailure = { "Error when applying time" } // todo: logcat
                )
            }
            localDateTime.day == localRightNow.day - 1 -> {
                formatTime(localDateTime.time, timeFormat).fold(
                    onSuccess = { f -> "Yesterday at $f" },
                    onFailure = { "Error when applying time" } // todo: logcat
                )
            }
            else -> {
                formatDateTime(localDateTime, Settings.shared.dateTimeFormat).fold(
                    onSuccess = { f -> f },
                    onFailure = { "Error when applying date time" } // todo: logcat
                )
            }
        }
    }

    fun formatFullDate(isoString: String): String {
        return try {
            val instant = Instant.parse(isoString)
            val timeZone = TimeZone.currentSystemDefault()
            val localDateTime = instant.toLocalDateTime(timeZone)
            
            val dayOfWeek = localDateTime.dayOfWeek.name.lowercase().replaceFirstChar { it.uppercase() }
            val month = localDateTime.month.name.lowercase().replaceFirstChar { it.uppercase() }
            val day = localDateTime.day
            val year = localDateTime.year
            
            val hour = localDateTime.hour
            val minute = localDateTime.minute.toString().padStart(2, '0')
            val amPm = if (hour >= 12) "PM" else "AM"
            val displayHour = when {
                hour == 0 -> 12
                hour > 12 -> hour - 12
                else -> hour
            }
            
            "$dayOfWeek, $month $day, $year at $displayHour:$minute $amPm"
        } catch (e: Exception) {
            isoString
        }
    }

    fun formatDiscordTimestamp(epochSeconds: Long, format: String): String {
        val instant = Instant.fromEpochSeconds(epochSeconds)
        val timeZone = TimeZone.currentSystemDefault()
        val localDateTime = instant.toLocalDateTime(timeZone)
        val now = Clock.System.now()
        
        val hour = localDateTime.hour
        val minute = localDateTime.minute.toString().padStart(2, '0')
        val second = localDateTime.second.toString().padStart(2, '0')
        val amPm = if (hour >= 12) "PM" else "AM"
        val displayHour = when {
            hour == 0 -> 12
            hour > 12 -> hour - 12
            else -> hour
        }
        val timeShort = "$displayHour:$minute $amPm"
        val timeLong = "$displayHour:$minute:$second $amPm"
        
        val month = localDateTime.month.name.lowercase().replaceFirstChar { it.uppercase() }
        val monthNum = localDateTime.month.number
        val day = localDateTime.day
        val year = localDateTime.year
        val dayOfWeek = localDateTime.dayOfWeek.name.lowercase().replaceFirstChar { it.uppercase() }

        return when (format) {
            "t" -> timeShort
            "T" -> timeLong
            "d" -> "$monthNum/$day/$year"
            "D" -> "$month $day, $year"
            "f" -> "$month $day, $year $timeShort"
            "F" -> "$dayOfWeek, $month $day, $year $timeShort"
            "R" -> {
                val duration = now - instant
                val seconds = duration.inWholeSeconds
                when {
                    seconds < 5 && seconds > -5 -> "just now"
                    seconds >= 0 -> {
                        when {
                            seconds < 60 -> "$seconds seconds ago"
                            seconds < 3600 -> "${seconds / 60} minutes ago"
                            seconds < 86400 -> "${seconds / 3600} hours ago"
                            seconds < 2592000 -> "${seconds / 86400} days ago"
                            seconds < 31536000 -> "${seconds / 2592000} months ago"
                            else -> "${seconds / 31536000} years ago"
                        }
                    }
                    else -> {
                        val absSeconds = -seconds
                        when {
                            absSeconds < 60 -> "in $absSeconds seconds"
                            absSeconds < 3600 -> "in ${absSeconds / 60} minutes"
                            absSeconds < 86400 -> "in ${absSeconds / 3600} hours"
                            absSeconds < 2592000 -> "in ${absSeconds / 86400} days"
                            absSeconds < 31536000 -> "in ${absSeconds / 2592000} months"
                            else -> "in ${absSeconds / 31536000} years"
                        }
                    }
                }
            }
            else -> "$month $day, $year $timeShort"
        }
    }

    fun getCurrentTime(): String {
        val now = Clock.System.now()
        val timeZone = TimeZone.currentSystemDefault()
        val localDateTime = now.toLocalDateTime(timeZone)
        val nowDate = now.toLocalDateTime(timeZone).date

        val hour = localDateTime.hour
        val minute = localDateTime.minute.toString().padStart(2, '0')
        val amPm = if (hour >= 12) "PM" else "AM"
        val displayHour = when {
            hour == 0 -> 12
            hour > 12 -> hour - 12
            else -> hour
        }
        return ("$displayHour:$minute $amPm")
    }

    fun getCurrentTime24H(): String {
        val now = Clock.System.now()
        val timeZone = TimeZone.currentSystemDefault()
        val localDateTime = now.toLocalDateTime(timeZone)
        val nowDate = now.toLocalDateTime(timeZone).date

        val hour = localDateTime.hour
        val minute = localDateTime.minute.toString().padStart(2, '0')
        return ("$hour:$minute")
    }

    fun now(): Long = Clock.System.now().toEpochMilliseconds()

    fun formatLogTimestamp(timestamp: Long): String {
        val instant = Instant.fromEpochMilliseconds(timestamp)
        val timeZone = TimeZone.currentSystemDefault()
        val localDateTime = instant.toLocalDateTime(timeZone)
        return "${localDateTime.hour.toString().padStart(2, '0')}:${localDateTime.minute.toString().padStart(2, '0')}:${localDateTime.second.toString().padStart(2, '0')}.${(timestamp % 1000).toString().padStart(3, '0')}"
    }
}
