import QtQuick 2.8
import QtQuick.Controls 2.1
import QtQuick.Layouts 1.3

ColumnLayout {
    id: dbg_console

    spacing: 4

    signal commandRun(string cmd)
    signal traceToggled(bool trace)
    property alias text: txtTxt

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
            font.pixelSize: 12
            wrapMode: Text.WrapAnywhere
            verticalAlignment: Text.AlignBottom

            readOnly: true
            selectByMouse: true
        }
    }

    Item {
        Layout.fillWidth: true
        Layout.preferredHeight: 20

        RowLayout {
            anchors.fill: parent
            spacing: 4

            CommandLine {
                Layout.fillWidth: true
                Layout.fillHeight: true

                onCommandRun: dbg_console.commandRun(cmd)
            }

            Button {
                text: "T"

                property bool active: false

                Layout.preferredWidth: 20
                Layout.fillHeight: true

                onClicked: {
                    active ^= true
                    down = active
                    dbg_console.traceToggled(active)
                }
            }
        }
    }
}
