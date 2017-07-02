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

QTimer *createScreenRepainter(QObject *scrn_view, Backend *backend, FrontendCallbacks *callbacks) {
    QObject *top_screen = qvariant_cast<QObject*>(QQmlProperty::read(scrn_view, "topScreen"));
    QObject *bot_screen = qvariant_cast<QObject*>(QQmlProperty::read(scrn_view, "botScreen"));
    QTimer *scrn_update_timer = new QTimer(scrn_view);

    QObject::connect(scrn_update_timer, &QTimer::timeout, [=] {
        size_t buf_size = 0;

        const uint8_t *top_buf = callbacks->top_screen(backend, &buf_size);
        QImage top(top_buf, 240, 400, 240*3, QImage::Format_RGB888);
        assert(buf_size = 240*3*400);

        const uint8_t *bot_buf = callbacks->bot_screen(backend, &buf_size);
        QImage bot(bot_buf, 240, 320, 240*3, QImage::Format_RGB888);
        assert(buf_size = 240*3*320);

        QTransform transform;
        transform.rotate(-90);
        QImage new_top = top.transformed(transform);
        QImage new_bot = bot.transformed(transform);

        QMetaObject::invokeMethod(top_screen, "setImage", Q_ARG(QImage&, new_top));
        QMetaObject::invokeMethod(bot_screen, "setImage", Q_ARG(QImage&, new_bot));
    });

    return scrn_update_timer;
}