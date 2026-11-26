import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Lampcord

// Credentials, token and 2FA steps of the Compose LoginScreen. QR login and
// captcha solving are not wired to the core yet.
Rectangle {
    id: root

    color: Theme.background

    // Stay on the MFA step after a rejected code, like the Compose login does.
    property bool mfaActive: false
    property string step: mfaActive ? "mfa" : manualStep
    property string manualStep: "credentials"
    readonly property bool busy: backend.authState === "pending" || backend.authState === "success"
    readonly property string errorText: backend.authState === "failed" ? backend.authMessage : ""

    function go(next) {
        backend.resetAuth()
        mfaActive = false
        manualStep = next
    }

    function submitMfa() {
        if (!root.busy && root.mfaActive && code.text.length === 6)
            backend.submitMfa(code.text)
    }

    Connections {
        target: backend
        function onAuthStateChanged() {
            if (backend.authState === "mfa")
                mfaActive = true
            if (backend.authState === "failed")
                code.text = ""
        }
    }

    // The QR session only lives while the login screen is up.
    Component.onCompleted: backend.startQrLogin()
    Component.onDestruction: backend.cancelQrLogin()

    IconButton {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.margins: 12
        visible: root.step !== "credentials"
        glyph: "Rounded.ArrowBack"
        iconColor: Theme.surfaceText
        tip: "Back to login"
        onClicked: root.go("credentials")
    }

    Flickable {
        anchors.top: parent.top
        anchors.topMargin: window.statusBarInset
        anchors.bottom: parent.bottom
        anchors.bottomMargin: window.navBarInset
        anchors.left: parent.left
        anchors.right: parent.right
        contentHeight: Math.max(height, column.implicitHeight + 48)
        boundsBehavior: Flickable.StopAtBounds

        ColumnLayout {
            id: column
            width: Math.min(480, root.width) - 48
            anchors.horizontalCenter: parent.horizontalCenter
            y: Math.max(24, (parent.height - implicitHeight) / 2)
            spacing: 0

            AsciiCat {
                Layout.alignment: Qt.AlignHCenter
            }

            Item { implicitHeight: 16 }

            StackLayout {
                Layout.fillWidth: true
                currentIndex: ["credentials", "token", "mfa"].indexOf(root.step)

                // Credentials
                ColumnLayout {
                    spacing: 0

                    OutlinedField {
                        id: login
                        Layout.fillWidth: true
                        label: "Enter phone or email"
                        focus: true
                        enabled: !root.busy
                        onTextEdited: if (backend.authState === "failed") backend.resetAuth()
                        onAccepted: password.forceActiveFocus()
                    }
                    Item { implicitHeight: 18 }
                    OutlinedField {
                        id: password
                        property bool shown: false
                        Layout.fillWidth: true
                        label: "Password"
                        echoMode: shown ? TextInput.Normal : TextInput.Password
                        enabled: !root.busy
                        error: root.errorText !== ""
                        trailingIcon: shown ? "Rounded.VisibilityOff" : "Rounded.Visibility"
                        onTrailingClicked: shown = !shown
                        onTextEdited: if (backend.authState === "failed") backend.resetAuth()
                        onAccepted: submit.clicked()
                    }
                    ErrorBanner { text: root.errorText }
                    Item { implicitHeight: 8 }
                    RowLayout {
                        Layout.fillWidth: true
                        TextButton {
                            Layout.alignment: Qt.AlignLeft
                            text: "Login with token"
                            onClicked: root.go("token")
                        }
                        Item { Layout.fillWidth: true }
                        TextButton {
                            Layout.alignment: Qt.AlignRight
                            text: "Forgot password?"
                            enabled: login.text.trim().length > 0
                            onClicked: backend.forgotPassword(login.text)
                        }
                    }
                    Item { implicitHeight: 36 }
                    FilledButton {
                        id: submit
                        Layout.fillWidth: true
                        text: "Login"
                        loading: root.busy
                        enabled: !root.busy
                        onClicked: backend.loginPassword(login.text, password.text)
                    }
                    Item { implicitHeight: 32 }
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 1
                        color: Theme.outlineVariant
                    }
                    Item { implicitHeight: 32 }
                    Text {
                        Layout.fillWidth: true
                        text: "Or scan this QR code to log in instantly"
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.letterSpacing: 0.25
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.Wrap
                    }
                    Item { implicitHeight: 16 }
                    QrPanel {
                        Layout.alignment: Qt.AlignHCenter
                    }
                }

                // Token
                ColumnLayout {
                    spacing: 0

                    StepHeading {
                        title: "Login with token"
                        subtitle: "Paste your Discord token. It is stored on this device only."
                    }
                    Item { implicitHeight: 20 }
                    OutlinedField {
                        id: token
                        Layout.fillWidth: true
                        label: "Token"
                        echoMode: TextInput.Password
                        enabled: !root.busy
                        error: root.errorText !== ""
                        onTextEdited: if (backend.authState === "failed") backend.resetAuth()
                        onAccepted: tokenSubmit.clicked()
                    }
                    ErrorBanner { text: root.errorText }
                    Item { implicitHeight: 20 }
                    FilledButton {
                        id: tokenSubmit
                        Layout.fillWidth: true
                        text: "Login"
                        loading: root.busy
                        enabled: !root.busy && token.text.trim().length > 0
                        onClicked: backend.loginToken(token.text)
                    }
                }

                // Two-factor
                ColumnLayout {
                    spacing: 0

                    StepHeading {
                        title: "Two-factor authentication"
                        subtitle: backend.authMessage !== "" ? backend.authMessage
                                                              : "Enter the 6-digit code from your authenticator app."
                    }
                    Item { implicitHeight: 20 }
                    OtpInput {
                        id: code
                        Layout.fillWidth: true
                        enabled: !root.busy
                        error: root.errorText !== ""
                        onAccepted: root.submitMfa()
                        onTextChanged: root.submitMfa()
                    }
                    ErrorBanner { text: root.errorText }
                    Item { implicitHeight: 8 }
                    TextButton {
                        Layout.alignment: Qt.AlignHCenter
                        text: "Send code by SMS instead"
                        onClicked: backend.sendMfaSms()
                    }
                    Item { implicitHeight: 12 }
                    FilledButton {
                        id: mfaSubmit
                        Layout.fillWidth: true
                        text: "Verify"
                        loading: root.busy
                        enabled: !root.busy && code.text.length === 6
                        onClicked: root.submitMfa()
                    }
                }
            }
        }
    }

    component StepHeading: ColumnLayout {
        property string title
        property string subtitle
        Layout.fillWidth: true
        spacing: 6

        Text {
            Layout.fillWidth: true
            text: parent.title
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 24
            font.weight: Font.Bold
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
        }
        Text {
            Layout.fillWidth: true
            visible: text !== ""
            text: parent.subtitle
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 14
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
        }
    }

    component ErrorBanner: Text {
        Layout.fillWidth: true
        Layout.topMargin: text !== "" ? 12 : 0
        visible: text !== ""
        color: Theme.error
        font.family: Theme.font
        font.pixelSize: 14
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.Wrap
    }
}
