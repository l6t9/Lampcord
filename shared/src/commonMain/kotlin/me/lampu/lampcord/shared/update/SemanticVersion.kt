package me.lampu.lampcord.shared.update

data class SemanticVersion(
    val major: Int,
    val minor: Int,
    val patch: Int,
    val prerelease: List<String>,
    val build: String? = null,
) : Comparable<SemanticVersion> {

    val isPrerelease: Boolean get() = prerelease.isNotEmpty()

    override fun compareTo(other: SemanticVersion): Int {
        major.compareTo(other.major).takeIf { it != 0 }?.let { return it }
        minor.compareTo(other.minor).takeIf { it != 0 }?.let { return it }
        patch.compareTo(other.patch).takeIf { it != 0 }?.let { return it }

        if (prerelease.isEmpty() && other.prerelease.isNotEmpty()) return 1
        if (prerelease.isNotEmpty() && other.prerelease.isEmpty()) return -1

        for (index in 0 until maxOf(prerelease.size, other.prerelease.size)) {
            val left = prerelease.getOrNull(index) ?: return -1
            val right = other.prerelease.getOrNull(index) ?: return 1
            val comparison = compareIdentifier(left, right)
            if (comparison != 0) return comparison
        }
        return 0
    }

    override fun toString(): String = buildString {
        append(major).append('.').append(minor).append('.').append(patch)
        if (prerelease.isNotEmpty()) {
            append('-').append(prerelease.joinToString("."))
        }
        if (!build.isNullOrBlank()) {
            append('+').append(build)
        }
    }

    companion object {
        private val pattern = Regex(
            "^v?(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)" +
                "(?:-([0-9A-Za-z.-]+))?" +
                "(?:\\+([0-9A-Za-z.-]+))?$"
        )

        private val trailingDigits = Regex("^(.*?)(\\d+)$")

        private fun compareIdentifier(left: String, right: String): Int {
            val leftMatch = trailingDigits.matchEntire(left)
            val rightMatch = trailingDigits.matchEntire(right)
            if (leftMatch == null || rightMatch == null) {
                return left.compareTo(right)
            }
            val leftPrefix = leftMatch.groupValues[1]
            val rightPrefix = rightMatch.groupValues[1]
            leftPrefix.compareTo(rightPrefix).takeIf { it != 0 }?.let { return it }
            return leftMatch.groupValues[2].toBigInteger()
                .compareTo(rightMatch.groupValues[2].toBigInteger())
        }

        fun parse(value: String): SemanticVersion? {
            val match = pattern.matchEntire(value.trim()) ?: return null
            return SemanticVersion(
                major = match.groupValues[1].toIntOrNull() ?: return null,
                minor = match.groupValues[2].toIntOrNull() ?: return null,
                patch = match.groupValues[3].toIntOrNull() ?: return null,
                prerelease = match.groupValues[4].takeIf(String::isNotEmpty)?.split('.').orEmpty(),
                build = match.groupValues[5].takeIf(String::isNotEmpty),
            )
        }

        fun isNewer(candidate: String, current: String): Boolean {
            val candidateVersion = parse(candidate) ?: return false
            val currentVersion = parse(current) ?: return false
            return candidateVersion > currentVersion
        }
    }
}
