import QtQuick 2.6
import QtQuick.Controls 2.1
import QtGraphicalEffects 1.0

Button {
    id: control
    property url imgSrc
    
    property color colorUp: "#111"
    property color colorDown: "#444"
    property color colorBgUp: "#FFF"
    property color colorBgDown: "#F8F8F8"

    contentItem: Item {
        anchors.fill: parent;

        Image {
            id: img
            source: control.imgSrc
            anchors.fill: parent
            fillMode: Image.Pad
            visible: false

            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.verticalCenter
            sourceSize.width: Math.floor(parent.width / 25) * 12
            sourceSize.height: Math.floor(parent.height / 25) * 12
        }

        ColorOverlay {
            anchors.fill: img
            opacity: enabled ? 1.0 : 0.3
            source: img
            cached: true
            color: control.down ? control.colorDown : control.colorUp
        }
    }

    background: Rectangle {
        implicitWidth: 100
        implicitHeight: 100
        opacity: enabled ? 1 : 0.3
        border.color: control.down ? control.colorDown : control.colorUp
        border.width: 1
        radius: 2
        color: control.down ? control.colorBgDown : control.colorBgUp
    }
}
