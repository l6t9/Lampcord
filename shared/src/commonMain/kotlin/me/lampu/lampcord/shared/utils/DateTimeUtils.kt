package me.lampu.lampcord.shared.utils

import kotlinx.datetime.*
import kotlinx.datetime.number
import kotlin.time.Clock
import kotlin.time.Instant

object DateTimeUtils {
    fun formatTimestamp(isoString: String): String {
        return try {
            val instant = Instant.parse(isoString)
            val now = Clock.System.now()
            val timeZone = TimeZone.currentSystemDefault()
            val localDateTime = instant.toLocalDateTime(timeZone)
            val nowDate = now.toLocalDateTime(timeZone).date
            
            val hour = localDateTime.hour
            val minute = localDateTime.minute.toString().padStart(2, '0')
            val amPm = if (hour >= 12) "PM" else "AM"
            val displayHour = when {
                hour == 0 -> 12
                hour > 12 -> hour - 12
                else -> hour
            }
            val timeString = "$displayHour:$minute $amPm"
            
            val date = localDateTime.date
            when {
                date == nowDate -> timeString
                date.dayOfYear == nowDate.dayOfYear - 1 && date.year == nowDate.year -> "Yesterday at $timeString"
                else -> {
                    val month = date.month.number
                    val day = date.day
                    val year = date.year.toString().takeLast(2)
                    "$month/$day/$year, $timeString"
                }
            }
        } catch (e: Exception) {
            isoString.take(10)
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
}
