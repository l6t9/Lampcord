/*
 * Lampcord Project (C) 2026
 * Licensed under GPL-3.0 | See git history for contributors
 */

import AVFoundation
import Combine
import Shared
import SwiftUI
import UIKit

@main
struct LampcordApp: App {
    init() {
        configureAppAudioSession()
        LampcordPlatform.shared.initialize()
    }

    var body: some Scene {
        WindowGroup {
            AppHostView()
        }
    }

    private func configureAppAudioSession() {
        do {
            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.playback, mode: .default)
            try session.setActive(true)
        } catch {
            print("Audio session configuration error: \(error.localizedDescription)")
        }
    }
}

private struct AppHostView: View {
    var body: some View {
        ComposeView()
            .ignoresSafeArea()
            .onOpenURL { url in
                MainViewControllerKt.handleOpenUrl(url: url.absoluteString)
            }
    }
}

struct ComposeView: UIViewControllerRepresentable {
    func makeUIViewController(context: Context) -> UIViewController {
        MainViewControllerKt.MainViewController()
    }

    func updateUIViewController(_ uiViewController: UIViewController, context: Context) {}
}