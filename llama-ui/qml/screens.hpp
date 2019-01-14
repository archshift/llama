#pragma once

#include <QtQuick/QQuickPaintedItem>

#include "interop.h"

class QQuickItem;


enum class WhichScreen {
    Top,
    Bottom
};

class Screen: public QQuickPaintedItem {
    Q_OBJECT

    Backend *backend;
    const FrontendCallbacks *callbacks;

    int real_w, real_h;
    WhichScreen screen;

public slots:
    void setCallbacks(Backend *backend, const FrontendCallbacks *callbacks);
    void paint(QPainter *painter);
public:
    Screen(WhichScreen screen, QQuickItem *parent=Q_NULLPTR);
};

class TopScreen: public Screen {
    Q_OBJECT
public:
    TopScreen(QQuickItem *parent=Q_NULLPTR): Screen(WhichScreen::Top, parent) { }
};

class BotScreen: public Screen {
    Q_OBJECT
public:
    BotScreen(QQuickItem *parent=Q_NULLPTR): Screen(WhichScreen::Bottom, parent) { }
};

void initScreenRepainter(QObject *scrn_view, Backend *backend, const FrontendCallbacks *callbacks);
