import QtQuick 2.8
import QtQuick.Controls 2.1
import QtQuick.Layouts 1.3

import Screens 1.0

Item {
    implicitWidth: 480
    implicitHeight: 480

    signal pauseToggled()
    signal stopped()
    signal reloaded()
    signal fullscreenActivated()
    signal configOpened()

    property alias topScreen: topScreen
    property alias botScreen: botScreen

    Rectangle {
        id: topScreenDecor
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 256.0/480.0 * contents.height + 0.5 * (parent.height - contents.height)
        color: "#0072BC"
    }

    Item {
        id: contents

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        width: Math.min(parent.width, parent.height)
        height: Math.min(parent.width, parent.height)

        Rectangle {
            id: topScreenDecorBounds
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 256.0/480.0 * contents.height
            color: "#00000000"
        }
        TopScreen {
            id: topScreen
            anchors.horizontalCenter: parent.horizontalCenter
            width: 400.0/480.0 * parent.width
            height: 240.0/480.0 * parent.height
        }
        BotScreen {
            id: botScreen
            anchors.top: topScreen.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            width: 320.0/480.0 * parent.width
            height: 240.0/480.0 * parent.height
        }

        ColumnLayout {
            anchors.left: parent.left
            anchors.top: topScreenDecorBounds.bottom
            anchors.bottom: parent.bottom
            anchors.right: botScreen.left

            anchors.margins: parent.width * 0.02
            spacing: parent.width * 0.02

            Button {
                text: "P/P"
                onClicked: pauseToggled()
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
            Button {
                text: "Stop"
                onClicked: stopped()
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
            Button {
                text: "Rload"
                onClicked: reloaded()
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }

        ColumnLayout {
            anchors.left: botScreen.right
            anchors.top: topScreenDecorBounds.bottom
            anchors.bottom: parent.bottom
            anchors.right: parent.right

            anchors.margins: parent.width * 0.02
            spacing: parent.width * 0.02

            Button {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
            Button {
                text: "Fscr"
                onClicked: fullscreenActivated()
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
            Button {
                text: "Cfg"
                onClicked: configOpened()
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }
}