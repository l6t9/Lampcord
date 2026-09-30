package me.lampu.lampcord.shared.utils

import java.security.MessageDigest
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals

class Sha256Test {

    private fun hex(bytes: ByteArray) = bytes.joinToString("") { "%02x".format(it) }

    private fun reference(input: String): String =
        MessageDigest.getInstance("SHA-256").digest(input.toByteArray())
            .joinToString("") { "%02x".format(it) }

    @Test
    fun matchesTheJdkForKnownVectors() {
        val vectors = listOf(
            "" to "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "abc" to "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "The quick brown fox jumps over the lazy dog" to
                "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        )
        vectors.forEach { (input, expected) ->
            assertEquals(expected, hex(sha256(input.toByteArray())), "sha256(\"$input\")")
        }
    }

    @Test
    fun matchesTheJdkAcrossBlockBoundaries() {
        listOf(55, 56, 63, 64, 65, 119, 120, 1000).forEach { length ->
            val input = "x".repeat(length)
            assertContentEquals(
                MessageDigest.getInstance("SHA-256").digest(input.toByteArray()),
                sha256(input.toByteArray()),
                "length $length"
            )
        }
    }

    @Test
    fun matchesTheJdkForEveryShortLength() {
        for (length in 0..200) {
            val input = ByteArray(length) { (it * 31 % 251).toByte() }
            assertContentEquals(
                MessageDigest.getInstance("SHA-256").digest(input),
                sha256(input),
                "length $length"
            )
        }
    }
}
