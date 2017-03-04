import QtQuick 2.8
import QtQuick.Controls 2.1
import QtQuick.Layouts 1.3

ColumnLayout {
    id: dbg_console

    spacing: 4

    signal commandRun(string cmd)
    property TextEdit text: txtTxt

    Rectangle {
        id: txtRect

        radius: 3
        border.width: 1
        border.color: "#AAAAAA"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true

        TextEdit {
            id: txtTxt

            height: parent.height
            width: parent.width
            padding: 4

            font.family: "Source Code Pro"
            font.pointSize: 12
            wrapMode: Text.WrapAnywhere
            verticalAlignment: Text.AlignBottom

            readOnly: true
            selectByMouse: true
        }
    }

    Rectangle {
        radius: 3
        border.width: 1
        border.color: "#AAAAAA"
        color: "#EEEEEE"

        Layout.preferredHeight: 20
        Layout.fillWidth: true

        TextInput {
            id: input
            anchors.fill: parent
            font.family: "Source Code Pro"
            font.pointSize: 12
            padding: 4

            onAccepted: {
                dbg_console.commandRun(text)
                clear()
            }
        }
    }
}