import QtQuick 2.8
import QtQuick.Controls 2.1

Rectangle {
    radius: 3
    border.width: 1
    border.color: "#AAAAAA"
    color: "#EEEEEE"

    signal commandRun(string cmd)

    TextInput {
        id: input
        anchors.fill: parent
        font.family: "Source Code Pro"
        font.pixelSize: 12
        padding: 4

        property var prevCommands: []
        onAccepted: {
            var ranCommand = text
            // Implicitly run the last command
            if (ranCommand === "") {
                ranCommand = prevCommands[0]
            }

            // Actually run command
            parent.commandRun(ranCommand)
            upCounter = -1
            clear()

            // Push onto the queue, which fits up to 20 commands
            if (prevCommands[0] != ranCommand) {
                prevCommands.unshift(ranCommand)
                if (prevCommands.length > 20) {
                    prevCommands.pop()
                }
            }
        }

        property int upCounter: -1
        Keys.onPressed: {
            // Handle moving between previously run commands with arrow keys

            if (event.key == Qt.Key_Up) {
                upCounter = Math.min(prevCommands.length - 1, upCounter + 1)
                text = prevCommands[upCounter]
                event.accepted = true
            } else if (event.key == Qt.Key_Down) {
                upCounter = Math.max(-1, upCounter - 1)
                if (upCounter >= 0) {
                    text = prevCommands[upCounter]
                }
                event.accepted = true
            }
        }
    }
}
