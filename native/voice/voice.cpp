#include <jni.h>
#include <dave/dave_interfaces.h>
#include <openssl/aead.h>
#include <opus.h>
#include <algorithm>
#include <array>
#include <stdexcept>

namespace dave = discord::dave;
using Bytes = std::vector<uint8_t>;

namespace {
struct Peer {
    std::unique_ptr<dave::IDecryptor> decryptor = dave::CreateDecryptor();
    std::unique_ptr<OpusDecoder, decltype(&opus_decoder_destroy)> decoder{nullptr, opus_decoder_destroy};
    Peer() {
        int error;
        decoder.reset(opus_decoder_create(48000, 2, &error));
        if (error != OPUS_OK) throw std::runtime_error("Could not create Opus decoder");
        decryptor->TransitionToPassthroughMode(false, std::chrono::seconds(0));
    }
};

struct Voice {
    std::string user;
    uint64_t channel;
    std::string mlsError;
    std::unique_ptr<dave::mls::ISession> session;
    std::shared_ptr<mlspp::SignaturePrivateKey> signingKey;
    std::unique_ptr<dave::IEncryptor> encryptor = dave::CreateEncryptor();
    std::unique_ptr<OpusEncoder, decltype(&opus_encoder_destroy)> encoder{nullptr, opus_encoder_destroy};
    std::map<std::string, Peer> peers;
    std::set<std::string> roster;
    std::map<int, std::unique_ptr<dave::IKeyRatchet>> transitions;
    bool ready = false;

    Voice(std::string userId, uint64_t channelId) : user(std::move(userId)), channel(channelId) {
        session = dave::mls::CreateSession(nullptr, "", [this](const auto&, const auto&) {
            // Never forward MLS material or protocol payloads to application logs.
            mlsError = "DAVE could not process the MLS message";
        });
        if (!session || !encryptor) throw std::runtime_error("Could not create DAVE session");
        encryptor->SetPassthroughMode(false);
        int error;
        encoder.reset(opus_encoder_create(48000, 2, OPUS_APPLICATION_VOIP, &error));
        if (error != OPUS_OK) throw std::runtime_error("Could not create Opus encoder");
        opus_encoder_ctl(encoder.get(), OPUS_SET_BITRATE(64000));
        opus_encoder_ctl(encoder.get(), OPUS_SET_COMPLEXITY(5));
        opus_encoder_ctl(encoder.get(), OPUS_SET_INBAND_FEC(1));
        opus_encoder_ctl(encoder.get(), OPUS_SET_PACKET_LOSS_PERC(10));
    }

    Bytes initialize(int version) {
        ready = false;
        transitions.clear(); roster.clear(); peers.clear();
        encryptor->SetKeyRatchet(nullptr);
        if (version != 1 || version > daveMaxSupportedProtocolVersion()) throw std::runtime_error("Unsupported DAVE version");
        // ponytail: a full MLS reset drops in-flight audio; retain expiring old decryptors if seamless resets are needed.
        session->Init(static_cast<uint16_t>(version), channel, user, signingKey);
        auto package = session->GetMarshalledKeyPackage();
        if (package.empty()) throw std::runtime_error("Missing MLS key package");
        return package;
    }

    void updateRoster(const dave::RosterMap& changes) {
        for (const auto& [id, key] : changes) {
            auto uid = std::to_string(id);
            if (key.empty()) {
                roster.erase(uid);
                peers.erase(uid);
            } else {
                roster.insert(uid);
            }
        }
    }

    void prepare(int transition) {
        auto sender = session->GetKeyRatchet(user);
        if (!sender) throw std::runtime_error("DAVE group has no sender key");
        for (const auto& uid : roster) {
            auto key = session->GetKeyRatchet(uid);
            if (!key) throw std::runtime_error("DAVE group has no receiver key");
            peers[uid].decryptor->TransitionToKeyRatchet(std::move(key));
        }
        if (transitions.size() >= 32) throw std::runtime_error("Too many pending DAVE transitions");
        transitions[transition] = std::move(sender);
        if (transition == 0) execute(0);
    }

    void execute(int transition) {
        auto it = transitions.find(transition);
        if (it == transitions.end()) throw std::runtime_error("Unknown DAVE transition");
        encryptor->SetKeyRatchet(std::move(it->second));
        transitions.erase(it);
        ready = true;
    }
};

Voice& voice(jlong handle) {
    if (!handle) throw std::runtime_error("Voice session is closed");
    return *reinterpret_cast<Voice*>(handle);
}

std::string string(JNIEnv* env, jstring value) {
    if (!value) throw std::runtime_error("Missing string");
    const char* chars = env->GetStringUTFChars(value, nullptr);
    if (!chars) throw std::bad_alloc();
    std::string result(chars);
    env->ReleaseStringUTFChars(value, chars);
    return result;
}

Bytes fromJavaBytes(JNIEnv* env, jbyteArray value) {
    if (!value) throw std::runtime_error("Missing bytes");
    auto length = env->GetArrayLength(value);
    if (length > 1024 * 1024) throw std::runtime_error("Oversized voice payload");
    Bytes result(length);
    env->GetByteArrayRegion(value, 0, length, reinterpret_cast<jbyte*>(result.data()));
    return result;
}

jbyteArray array(JNIEnv* env, const Bytes& value) {
    auto result = env->NewByteArray(static_cast<jsize>(value.size()));
    if (result) env->SetByteArrayRegion(result, 0, static_cast<jsize>(value.size()), reinterpret_cast<const jbyte*>(value.data()));
    return result;
}

std::set<std::string> users(JNIEnv* env, jobjectArray ids) {
    if (!ids || env->GetArrayLength(ids) > 10000) throw std::runtime_error("Invalid participant list");
    std::set<std::string> result;
    for (int i = 0; i < env->GetArrayLength(ids); ++i) {
        auto id = static_cast<jstring>(env->GetObjectArrayElement(ids, i));
        result.insert(string(env, id));
        env->DeleteLocalRef(id);
    }
    return result;
}

// No C++ exception may cross JNI, including allocation failures.
template <typename T, typename F> T checked(JNIEnv* env, T fallback, F fn) {
    try { return fn(); }
    catch (const std::exception&) {
        if (!env->ExceptionCheck()) env->ThrowNew(env->FindClass("java/lang/IllegalStateException"), "Native voice operation failed");
        return fallback;
    }
    catch (...) {
        if (!env->ExceptionCheck()) env->ThrowNew(env->FindClass("java/lang/IllegalStateException"), "Native voice operation failed");
        return fallback;
    }
}
}

#define JNI(name) Java_me_lampu_lampcord_shared_voice_NativeVoice_##name

extern "C" JNIEXPORT jlong JNICALL JNI(create)(JNIEnv* env, jobject, jstring user, jstring channel) {
    return checked<jlong>(env, 0, [&] {
        // libdave's default verbose sink can log key packages. Disable it completely.
        daveSetLogSinkCallback([](DAVELoggingSeverity, const char*, int, const char*) {});
        return reinterpret_cast<jlong>(new Voice(string(env, user), std::stoull(string(env, channel))));
    });
}

extern "C" JNIEXPORT void JNICALL JNI(destroy)(JNIEnv*, jobject, jlong handle) {
    delete reinterpret_cast<Voice*>(handle);
}

extern "C" JNIEXPORT jbyteArray JNICALL JNI(init)(JNIEnv* env, jobject, jlong handle, jint version) {
    return checked<jbyteArray>(env, nullptr, [&] {
        return array(env, voice(handle).initialize(version));
    });
}

extern "C" JNIEXPORT void JNICALL JNI(externalSender)(JNIEnv* env, jobject, jlong handle, jbyteArray payload) {
    checked<int>(env, 0, [&] {
        auto& v = voice(handle);
        v.mlsError.clear();
        v.session->SetExternalSender(fromJavaBytes(env, payload));
        if (!v.mlsError.empty()) throw std::runtime_error(v.mlsError);
        return 0;
    });
}

extern "C" JNIEXPORT jbyteArray JNICALL JNI(proposals)(JNIEnv* env, jobject, jlong handle, jbyteArray payload, jobjectArray ids) {
    return checked<jbyteArray>(env, nullptr, [&] {
        auto& v = voice(handle);
        v.mlsError.clear();
        auto result = v.session->ProcessProposals(fromJavaBytes(env, payload), users(env, ids));
        if (!v.mlsError.empty()) throw std::runtime_error(v.mlsError);
        return result ? array(env, *result) : nullptr;
    });
}

// 0: invalid (reset/rejoin), 1: ignored, 2: prepared successfully.
extern "C" JNIEXPORT jint JNICALL JNI(commit)(JNIEnv* env, jobject, jlong handle, jbyteArray payload, jobjectArray ids, jint transition, jboolean welcome) {
    return checked<jint>(env, 0, [&] {
        auto& v = voice(handle);
        if (welcome) {
            auto result = v.session->ProcessWelcome(fromJavaBytes(env, payload), users(env, ids));
            if (!result) return 0;
            v.roster.clear(); v.peers.clear();
            v.updateRoster(*result);
        } else {
            auto result = v.session->ProcessCommit(fromJavaBytes(env, payload));
            if (std::holds_alternative<dave::ignored_t>(result)) return 1;
            if (std::holds_alternative<dave::failed_t>(result)) return 0;
            v.updateRoster(std::get<dave::RosterMap>(result));
        }
        v.prepare(transition);
        return 2;
    });
}

extern "C" JNIEXPORT void JNICALL JNI(transition)(JNIEnv* env, jobject, jlong handle, jint id, jboolean prepare) {
    checked<int>(env, 0, [&] {
        auto& v = voice(handle);
        if (prepare) v.prepare(id); else v.execute(id);
        return 0;
    });
}

extern "C" JNIEXPORT void JNICALL JNI(removeUser)(JNIEnv* env, jobject, jlong handle, jstring user) {
    checked<int>(env, 0, [&] {
        auto& v = voice(handle);
        auto uid = string(env, user);
        v.peers.erase(uid);
        v.roster.erase(uid);
        return 0;
    });
}

extern "C" JNIEXPORT jbyteArray JNICALL JNI(authenticator)(JNIEnv* env, jobject, jlong handle) {
    return checked<jbyteArray>(env, nullptr, [&] { return array(env, voice(handle).session->GetLastEpochAuthenticator()); });
}

extern "C" JNIEXPORT jbyteArray JNICALL JNI(encode)(JNIEnv* env, jobject, jlong handle, jshortArray pcm) {
    return checked<jbyteArray>(env, nullptr, [&] {
        if (!pcm || env->GetArrayLength(pcm) != 1920) throw std::runtime_error("Expected 20ms stereo PCM");
        std::array<jshort, 1920> samples;
        env->GetShortArrayRegion(pcm, 0, 1920, samples.data());
        Bytes out(1275);
        int length = opus_encode(voice(handle).encoder.get(), samples.data(), 960, out.data(), static_cast<opus_int32>(out.size()));
        if (length < 0) throw std::runtime_error("Opus encoding failed");
        out.resize(length);
        return array(env, out);
    });
}

extern "C" JNIEXPORT jbyteArray JNICALL JNI(encrypt)(JNIEnv* env, jobject, jlong handle, jint ssrc, jbyteArray opus) {
    return checked<jbyteArray>(env, nullptr, [&]() -> jbyteArray {
        auto& v = voice(handle);
        if (!v.ready) return nullptr; // Never send plaintext while joining/rekeying.
        auto in = fromJavaBytes(env, opus);
        if (in.size() > 1275) throw std::runtime_error("Oversized Opus frame");
        v.encryptor->AssignSsrcToCodec(ssrc, dave::Opus);
        Bytes out(v.encryptor->GetMaxCiphertextByteSize(dave::Audio, in.size()));
        size_t length = 0;
        auto result = v.encryptor->Encrypt(dave::Audio, ssrc, {in.data(), in.size()}, {out.data(), out.size()}, &length);
        if (result != dave::IEncryptor::Success) throw std::runtime_error("DAVE encryption failed");
        out.resize(length);
        return array(env, out);
    });
}

extern "C" JNIEXPORT jbyteArray JNICALL JNI(decrypt)(JNIEnv* env, jobject, jlong handle, jstring user, jbyteArray frame) {
    return checked<jbyteArray>(env, nullptr, [&]() -> jbyteArray {
        auto& v = voice(handle);
        auto peer = v.peers.find(string(env, user));
        if (peer == v.peers.end()) return nullptr;
        auto in = fromJavaBytes(env, frame);
        if (in.size() > 8192) return nullptr;
        Bytes out(in.size());
        size_t length = 0;
        auto result = peer->second.decryptor->Decrypt(dave::Audio, {in.data(), in.size()}, {out.data(), out.size()}, &length);
        if (result != dave::IDecryptor::Success) return nullptr;
        out.resize(length);
        return array(env, out);
    });
}

extern "C" JNIEXPORT jshortArray JNICALL JNI(decode)(JNIEnv* env, jobject, jlong handle, jstring user, jbyteArray opus) {
    return checked<jshortArray>(env, nullptr, [&]() -> jshortArray {
        auto& v = voice(handle);
        auto peer = v.peers.find(string(env, user));
        if (peer == v.peers.end()) return nullptr;
        auto in = opus ? fromJavaBytes(env, opus) : Bytes{};
        if (in.size() > 8192) return nullptr;
        std::array<jshort, 5760 * 2> pcm;
        int count = opus_decode(peer->second.decoder.get(), in.empty() ? nullptr : in.data(), static_cast<opus_int32>(in.size()), pcm.data(), opus ? 5760 : 960, 0);
        if (count < 0) return nullptr;
        auto result = env->NewShortArray(count * 2);
        if (result) env->SetShortArrayRegion(result, 0, count * 2, pcm.data());
        return result;
    });
}

// Discord's RTP-size transport AEAD. The header is authenticated; the counter is a suffix.
extern "C" JNIEXPORT jbyteArray JNICALL JNI(aead)(JNIEnv* env, jobject, jboolean encrypt, jboolean aes, jbyteArray key, jbyteArray nonce, jbyteArray header, jbyteArray payload) {
    return checked<jbyteArray>(env, nullptr, [&]() -> jbyteArray {
        auto k = fromJavaBytes(env, key), n = fromJavaBytes(env, nonce), h = fromJavaBytes(env, header), in = fromJavaBytes(env, payload);
        const auto* algorithm = aes ? EVP_aead_aes_256_gcm() : EVP_aead_xchacha20_poly1305();
        if (k.size() != 32 || n.size() != EVP_AEAD_nonce_length(algorithm) || h.size() > 256 || in.size() > 65536)
            throw std::runtime_error("Invalid AEAD parameters");
        bssl::UniquePtr<EVP_AEAD_CTX> ctx(EVP_AEAD_CTX_new(algorithm, k.data(), k.size(), 16));
        if (!ctx) throw std::runtime_error("AEAD initialization failed");
        Bytes out(in.size() + 16);
        size_t length = 0;
        int ok = encrypt
            ? EVP_AEAD_CTX_seal(ctx.get(), out.data(), &length, out.size(), n.data(), n.size(), in.data(), in.size(), h.data(), h.size())
            : EVP_AEAD_CTX_open(ctx.get(), out.data(), &length, out.size(), n.data(), n.size(), in.data(), in.size(), h.data(), h.size());
        if (!ok) {
            if (encrypt) throw std::runtime_error("Transport encryption failed");
            return nullptr;
        }
        out.resize(length);
        return array(env, out);
    });
}
