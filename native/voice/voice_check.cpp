// A small executable check of the production DAVE state, with Discord's test external sender.
#include "external_sender.h"
#include "voice.cpp"
#include <cmath>
#include <iostream>

static void check(bool ok) { if (!ok) throw std::runtime_error("Voice protocol check failed"); }

int main() {
    daveSetLogSinkCallback([](DAVELoggingSeverity, const char*, int, const char*) {});
    try {
        constexpr uint64_t channel = 1234567890;
        Voice alice("1234123412341234", channel), bob("5678567856785678", channel), carol("9876987698769876", channel);
        dave::test::ExternalSender sender(1, channel);
        for (auto* v : {&alice, &bob, &carol}) {
            v->session->SetExternalSender(sender.GetMarshalledExternalSender());
            check(!v->initialize(1).empty());
            check(!v->ready && !v->encryptor->IsPassthroughMode());
        }
        const std::set<std::string> recognized{alice.user, bob.user, carol.user};
        auto add = [&](Voice& newcomer, uint32_t epoch, int transition) {
            auto proposal = sender.ProposeAdd(epoch, newcomer.session->GetMarshalledKeyPackage());
            auto result = alice.session->ProcessProposals(proposal, recognized);
            check(result.has_value());
            if (epoch > 0) check(bob.session->ProcessProposals(proposal, recognized).has_value());
            auto [commit, welcome] = sender.SplitCommitWelcome(*result);
            auto roster = alice.session->ProcessCommit(commit);
            check(std::holds_alternative<dave::RosterMap>(roster));
            alice.updateRoster(std::get<dave::RosterMap>(roster));
            alice.prepare(transition);
            if (epoch > 0) {
                auto existing = bob.session->ProcessCommit(commit);
                check(std::holds_alternative<dave::RosterMap>(existing));
                bob.updateRoster(std::get<dave::RosterMap>(existing));
                bob.prepare(transition);
            }
            auto joined = newcomer.session->ProcessWelcome(welcome, recognized);
            check(joined.has_value());
            newcomer.updateRoster(*joined);
            newcomer.prepare(transition);
        };
        add(bob, 0, 0);
        check(alice.ready && bob.ready);
        check(alice.session->GetLastEpochAuthenticator() == bob.session->GetLastEpochAuthenticator());
        std::array<opus_int16, 1920> pcm;
        for (size_t i = 0; i < pcm.size(); ++i) pcm[i] = static_cast<opus_int16>(std::sin(i / 2.0 * 440.0 * 6.283185 / 48000.0) * 4000);
        Bytes opus(1275);
        auto encoded = opus_encode(alice.encoder.get(), pcm.data(), 960, opus.data(), static_cast<opus_int32>(opus.size()));
        check(encoded > 0); opus.resize(encoded);
        auto exchange = [&](Voice& recipient) {
            alice.encryptor->AssignSsrcToCodec(42, dave::Opus);
            Bytes encrypted(alice.encryptor->GetMaxCiphertextByteSize(dave::Audio, opus.size()));
            size_t length;
            check(alice.encryptor->Encrypt(dave::Audio, 42, {opus.data(), opus.size()}, {encrypted.data(), encrypted.size()}, &length) == dave::IEncryptor::Success);
            encrypted.resize(length);
            Bytes plain(encrypted.size());
            auto& decryptor = recipient.peers.at(alice.user).decryptor;
            encrypted[0] ^= 1;
            check(decryptor->Decrypt(dave::Audio, {encrypted.data(), encrypted.size()}, {plain.data(), plain.size()}, &length) != dave::IDecryptor::Success);
            encrypted[0] ^= 1;
            check(decryptor->Decrypt(dave::Audio, {encrypted.data(), encrypted.size()}, {plain.data(), plain.size()}, &length) == dave::IDecryptor::Success);
            plain.resize(length); check(plain == opus);
            check(opus_decode(recipient.peers.at(alice.user).decoder.get(), plain.data(), static_cast<opus_int32>(plain.size()), pcm.data(), 960, 0) == 960);
            // Replays and cleartext audio must not be accepted as DAVE frames.
            check(decryptor->Decrypt(dave::Audio, {encrypted.data(), encrypted.size()}, {plain.data(), plain.size()}, &length) != dave::IDecryptor::Success);
            check(decryptor->Decrypt(dave::Audio, {opus.data(), opus.size()}, {plain.data(), plain.size()}, &length) != dave::IDecryptor::Success);
        };
        exchange(bob);
        add(carol, 1, 7);
        check(!carol.ready);
        exchange(bob); // Previous sender epoch remains usable until Execute Transition.
        alice.execute(7); bob.execute(7); carol.execute(7);
        exchange(bob); exchange(carol);
        check(alice.session->GetLastEpochAuthenticator() == carol.session->GetLastEpochAuthenticator());
        alice.updateRoster({{std::stoull(carol.user), {}}});
        check(alice.peers.count(carol.user) == 0 && alice.roster.count(carol.user) == 0);
        bool rejected = false;
        try { alice.execute(99); } catch (const std::exception&) { rejected = true; }
        check(rejected);
        check(!alice.initialize(1).empty());
        check(!alice.ready && alice.peers.empty() && alice.roster.empty() && alice.transitions.empty());
        std::cout << "DAVE join, rekey, Opus round-trip, tamper, replay and fail-closed checks passed\n";
        return 0;
    } catch (const std::exception& e) {
        std::cerr << e.what() << '\n';
        return 1;
    }
}
