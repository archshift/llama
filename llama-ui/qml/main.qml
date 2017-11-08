import QtQuick 2.6
import QtQuick.Controls 2.1
import QtQuick.Layouts 1.3

Item {
    width: 880
    height: 480

    property alias scrnView: scrnView
    property alias dbgConsole: dbgConsole

    RowLayout {
        anchors.fill: parent
        anchors.margins: 4

        ScreenView {
            id: scrnView
            Layout.fillWidth: true
            Layout.fillHeight: true

            onDbgViewToggled: {
                dbgConsole.visible = !dbgConsole.visible
            }
        }

        DbgConsole {
            id: dbgConsole
            Layout.preferredWidth: 400
            Layout.fillHeight: true
        }
    }
}