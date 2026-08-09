package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class SearchResponse(
    val total_results: Int,
    val messages: List<List<Message>>,
    val threads: List<Channel> = emptyList(),
    val members: List<Member> = emptyList()
)
