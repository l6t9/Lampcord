package me.lampu.lampcord.shared.di

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.api.ApplicationApi
import me.lampu.lampcord.shared.api.AuthApi
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.api.RemoteAuthClient
import me.lampu.lampcord.shared.api.RestClient
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.api.createHttpClient
import me.lampu.lampcord.shared.database.AppDatabase
import me.lampu.lampcord.shared.database.MessageDao
import me.lampu.lampcord.shared.database.getRoomDatabase
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.notifications.PushTokenRegistrar
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.state.handlers.*
import me.lampu.lampcord.shared.utils.getDatabaseBuilder
import org.koin.dsl.module

val coreModule = module {
    single {
        Json {
            ignoreUnknownKeys = true
            coerceInputValues = true
            isLenient = true
            explicitNulls = false
        }
    }
    single { createHttpClient() }
    single { RestClient(get(), get()) }
    single { CoroutineScope(Dispatchers.Main) }
}

val apiModule = module {
    single { AuthApi(get()) }
    single { ChannelApi(get()) }
    single { MessageApi(get()) }
    single { GuildApi(get()) }
    single { UserApi(get()) }
    single { MediaApi(get()) }
    single { ApplicationApi(get()) }
}

val databaseModule = module {
    single { getRoomDatabase(getDatabaseBuilder()) }
    single<MessageDao> { get<AppDatabase>().messageDao() }
    single { MessageLogger(get<Json>(), get<MessageDao>(), get<UserStore>(), get<CoroutineScope>()) }
}

val networkModule = module {
    single { RemoteAuthClient(get(), get()) }
    single { GatewayManager(get(), get(), get(), getOrNull<ReadStateStore>()) }
    single { VoiceGatewayManager(get(), get()) }
}

val storeModule = module {
    single { UserStore() }
    single { EntityStore(get()) }
    single { SelectionStore() }
    single { ReadStateStore(get()) }
    single { UserGuildSettingsStore() }
    single { AppErrorStore() }
    single { PresenceStore(userApi = get()) }
    single { ClientProfileStore(httpClient = get(), json = get(), scope = get()) }
    single { RelationshipStore(userApi = get(), userStore = get(), scope = get()) }
    single {
        GuildStore(
            guildApi = get(), errorStore = get(), selectionStore = get(),
            entityStore = get(), userGuildSettingsStore = get(), readStateStore = get(), userStore = get(), scope = get()
        )
    }
    single {
        MemberListStore(
            gatewayManager = get(), selectionStore = get(), userStore = get(), presenceStore = get()
        )
    }
    single {
        MessageStore(
            messageApi = get(), channelApi = get(), userStore = get(), errorStore = get(),
            selectionStore = get(), messageLogger = get(), settingsStore = get(), scope = get()
        )
    }
    single { TypingStore(scope = get()) }
    single {
        VoiceStore(
            gatewayManager = get(), voiceGatewayManager = get(), channelApi = get(), userStore = get(),
            guildStore = get(), settingsStore = get(), json = get(), scope = get()
        )
    }
    single {
        SearchStore(
            messageApi = get(), memberListStore = get(), guildStore = get(), userStore = get(), json = get(), scope = get()
        )
    }
    single { ProfileStore(userApi = get(), scope = get()) }
    single {
        AutocompleteStore(
            memberListStore = get(), relationshipStore = get(), guildStore = get(),
            userStore = get(), commandStore = get(), guildApi = get()
        )
    }
    single { CommandStore(guildApi = get(), gatewayManager = get(), scope = get()) }
    single { ExperimentStore() }
    single { BadgeStore(httpClient = get(), json = get(), scope = get()) }
    single { FinderStore(guildStore = get(), scope = get()) }
    single { MentionsStore(messageApi = get(), scope = get()) }
    single { EmojiStore() }
    single { ApplicationStore(get(), get()) }
    single { TokenStore(json = get()) }
    single { SettingsStore(userApi = get()) }
    single { ThemeStore(scope = get()) }
    single {
        NavigationStore(
            channelApi = get(), guildApi = get(), messageApi = get(), userApi = get(),
            guildStore = get(), memberListStore = get(), messageStore = get(), userStore = get(),
            readStateStore = get(), profileStore = get(), commandStore = get(), selectionStore = get(),
            finderStore = get(), notifier = getOrNull<MessageNotifier>(), scope = get()
        )
    }
    single { NotificationStore(scope = get()) }
    single {
        ChannelNavigator(
            navigationStore = get(), entityStore = get(), gatewayManager = get(), scope = get()
        )
    }
    single(createdAtStart = true) {
        SessionManager(
            gatewayManager = get(), voiceGatewayManager = get(), authApi = get(), navigationStore = get(),
            tokenStore = get(), userStore = get(), gatewayHandler = get(), entityStore = get(),
            readStateStore = get(), userGuildSettingsStore = get(), presenceStore = get(),
            relationshipStore = get(), guildStore = get(), memberListStore = get(), messageStore = get(),
            typingStore = get(), commandStore = get(), voiceStore = get(), pushTokenRegistrar = getOrNull<PushTokenRegistrar>()
        )
    }

    single {
        MessageEventHandler(
            json = get(), userStore = get(), messageStore = get(), messageLogger = get(),
            readStateStore = get(), entityStore = get(), guildStore = get(),
            navigationStore = get(), finderStore = get(), scope = get()
        )
    }
    single {
        GuildEventHandler(
            json = get(), entityStore = get(), guildStore = get(), navigationStore = get(),
            settingsStore = get(), userStore = get(), presenceStore = get()
        )
    }
    single {
        PresenceEventHandler(json = get(), presenceStore = get(), userStore = get())
    }
    single {
        RelationshipEventHandler(json = get(), relationshipStore = get())
    }
    single {
        UserEventHandler(json = get(), userStore = get(), settingsStore = get(), guildStore = get())
    }
    single {
        TypingEventHandler(json = get(), userStore = get(), typingStore = get()) {
            get<UserStore>().currentUser.value?.id
        }
    }
    single {
        MemberListEventHandler(json = get(), memberListStore = get())
    }
    single {
        NotificationEventHandler(
            json = get(), userStore = get(), messageStore = get(), navigationStore = get(),
            userGuildSettingsStore = get(), relationshipStore = get(), entityStore = get(),
            notifier = getOrNull<MessageNotifier>()
        )
    }

    single {
        GatewayEventDispatcher(
            listOf(
                get<MessageEventHandler>(),
                get<GuildEventHandler>(),
                get<PresenceEventHandler>(),
                get<RelationshipEventHandler>(),
                get<UserEventHandler>(),
                get<TypingEventHandler>(),
                get<MemberListEventHandler>(),
                get<NotificationEventHandler>(),
                get<VoiceStore>()
            )
        )
    }
}

val gatewayModule = module {
    single {
        GatewayHandler(
            json = get(), dispatcher = get(), scope = get(),
            userStore = get(), settingsStore = get(), userGuildSettingsStore = get(),
            guildStore = get(), messageStore = get(), readStateStore = get(),
            experimentStore = get(), relationshipStore = get(), presenceStore = get(),
            navigationStore = get(), gatewayManager = get(), tokenStore = get(),
            memberListStore = get(), voiceStore = get()
        )
    }
}

val appModule = module {
    includes(
        coreModule, apiModule, databaseModule, networkModule,
        storeModule, gatewayModule
    )
}
