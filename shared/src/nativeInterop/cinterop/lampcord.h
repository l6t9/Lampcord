#include <stdbool.h>
#include <stdint.h>

typedef struct LampcordClient LampcordClient;

LampcordClient *lampcord_engine_new(const char *api_base, const char *storage_path);
void lampcord_engine_free(LampcordClient *client);
char *lampcord_engine_snapshot_json(const LampcordClient *client);
void lampcord_string_free(char *value);
bool lampcord_engine_connect_token(const LampcordClient *client, const char *token);
bool lampcord_engine_connect_stored_token(const LampcordClient *client);
bool lampcord_engine_select_guild(const LampcordClient *client, uint64_t guild_id);
bool lampcord_engine_select_channel(const LampcordClient *client, uint64_t channel_id);
