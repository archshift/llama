#include "screens.hpp"

#include <QImage>
#include <QPainter>
#include <QQmlProperty>
#include <QTimer>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickPaintedItem>

#include <cassert>

Screen::Screen(int w, int h, QQuickItem *parent):
        QQuickPaintedItem(parent), pix_buffer(w, h) {
    pix_buffer.fill(Qt::black);
    setOpaquePainting(true);
}

void Screen::setImage(QImage &image) {
    pix_buffer.convertFromImage(image);
    update();
}

void Screen::paint(QPainter *painter) {
    painter->setRenderHint(QPainter::Antialiasing);
    QRectF src { 0, 0, (qreal)pix_buffer.width(), (qreal)pix_buffer.height() };
    QRectF dst { 0, 0, (qreal)width(), (qreal)height() };
    painter->drawPixmap(dst, pix_buffer, src);
}

static const QMatrix SCREEN_ROTATE(0, -1,
                                   1,  0,
                                   0,  0);

QTimer *createScreenRepainter(QObject *scrn_view, Backend *backend, const FrontendCallbacks *callbacks) {
    QObject *top_screen = qvariant_cast<QObject*>(QQmlProperty::read(scrn_view, "topScreen"));
    QObject *bot_screen = qvariant_cast<QObject*>(QQmlProperty::read(scrn_view, "botScreen"));
    QTimer *scrn_update_timer = new QTimer(scrn_view);

    QObject::connect(scrn_update_timer, &QTimer::timeout, [=] {
        size_t top_size = 0;
        size_t bot_size = 0;

        const uint8_t *top_buf = callbacks->top_screen(backend, &top_size);
        const uint8_t *bot_buf = callbacks->bot_screen(backend, &bot_size);

        if (!top_buf || !bot_buf) return;

        QImage top(top_buf, 240, 400, 240*3, QImage::Format_RGB888);
        assert(top_size == 240*3*400);
        QImage bot(bot_buf, 240, 320, 240*3, QImage::Format_RGB888);
        assert(bot_size == 240*3*320);

        QImage new_top = top.transformed(SCREEN_ROTATE);
        QImage new_bot = bot.transformed(SCREEN_ROTATE);

        QMetaObject::invokeMethod(top_screen, "setImage", Q_ARG(QImage&, new_top));
        QMetaObject::invokeMethod(bot_screen, "setImage", Q_ARG(QImage&, new_bot));
    });

    return scrn_update_timer;
}
