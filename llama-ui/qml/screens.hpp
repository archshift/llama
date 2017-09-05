#pragma once

#include <QPixmap>
#include <QtQuick/QQuickPaintedItem>

#include "interop.h"

class QQuickItem;
class QPixmap;

class Screen: public QQuickPaintedItem {
    Q_OBJECT
    QPixmap pix_buffer;
public slots:
    void setImage(QImage &image);
    void paint(QPainter *painter);
public:
    Screen(int w, int h, QQuickItem *parent=Q_NULLPTR);
};

class TopScreen: public Screen {
    Q_OBJECT
public:
    TopScreen(QQuickItem *parent=Q_NULLPTR): Screen(400, 240, parent) { }
};

class BotScreen: public Screen {
    Q_OBJECT
public:
    BotScreen(QQuickItem *parent=Q_NULLPTR): Screen(320, 240, parent) { }
};

QTimer *createScreenRepainter(QObject *scrn_view, Backend *backend, const FrontendCallbacks *callbacks);
