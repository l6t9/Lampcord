package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.autofill.ContentType
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.semantics.contentType
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp

@Composable
fun OtpInput(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier.fillMaxWidth(),
    length: Int = 6,
    enabled: Boolean = true,
    isError: Boolean = false,
    autoFocus: Boolean = false,
    keyboardActions: KeyboardActions = KeyboardActions.Default
) {
    val focusRequester = remember { FocusRequester() }
    LaunchedEffect(autoFocus, enabled) {
        if (autoFocus && enabled) runCatching { focusRequester.requestFocus() }
    }
    BasicTextField(
        value = value,
        onValueChange = { next -> onValueChange(next.filter { it.isDigit() }.take(length)) },
        modifier = modifier
            .focusRequester(focusRequester)
            .semantics { contentType = ContentType.SmsOtpCode },
        enabled = enabled,
        singleLine = true,
        keyboardOptions = KeyboardOptions(
            keyboardType = KeyboardType.Number,
            imeAction = ImeAction.Done
        ),
        keyboardActions = keyboardActions,
        decorationBox = { innerTextField ->
            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    repeat(length) { index ->
                        val digit = value.getOrNull(index)?.toString().orEmpty()
                        val isCurrent = enabled && index == value.length
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .height(56.dp)
                                .border(
                                    width = if (isCurrent) 2.dp else 1.dp,
                                    color = when {
                                        isError -> MaterialTheme.colorScheme.error
                                        isCurrent -> MaterialTheme.colorScheme.primary
                                        else -> MaterialTheme.colorScheme.outline
                                    },
                                    shape = MaterialTheme.shapes.medium
                                ),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(
                                text = digit,
                                style = MaterialTheme.typography.titleLarge,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                        }
                    }
                }
                Box(modifier = Modifier.matchParentSize().alpha(0f)) {
                    innerTextField()
                }
            }
        }
    )
}
