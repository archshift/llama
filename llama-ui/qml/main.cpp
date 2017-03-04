#include <QGuiApplication>
#include <QQmlProperty>
#include <QString>
#include <QTimer>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickView>

#include <cassert>
#include <cstdio>
#include <mutex>
#include <memory>
#include <vector>

#include <lgl.h>

struct Backend;
struct FrontendCallbacks {
    void(*set_running)(Backend*, bool);
    bool(*is_running)(Backend*);
};

class ConsoleManager: public QObject
{
    Q_OBJECT

    QTimer *text_poll_timer;
    QObject *dbg_console_text;
    std::vector<char> text_buf;
public slots:
    void fillLog() {
        LogBufferView view = lgl_buffer({ &text_buf[0], text_buf.size() });
        QString txt = QString::fromUtf8(view.buf_ptr, view.buf_size);
        dbg_console_text->setProperty("text", txt);
    }

    void runCommand(const QString &msg) {
        qDebug() << "Ran command:" << msg;
    }

public:
    ConsoleManager(QObject *textedit):
            dbg_console_text(textedit),
            text_buf(lgl_buffer_size()) {
        text_poll_timer = new QTimer(this);
        QObject::connect(text_poll_timer, SIGNAL(timeout()), this, SLOT(fillLog()));
        text_poll_timer->start(10);
    }
};

class ScreenManager: public QObject
{
    Q_OBJECT

    QObject *screen_view;
    Backend *backend;
    FrontendCallbacks *callbacks;
public slots:
    void togglePaused() {
        bool val = !callbacks->is_running(backend);
        callbacks->set_running(backend, val);
    }

public:
    ScreenManager(QObject *screen_view, Backend *backend, FrontendCallbacks *callbacks):
            screen_view(screen_view), backend(backend), callbacks(callbacks) { }
};


extern "C" int llama_open_gui(Backend *backend, FrontendCallbacks *callbacks) {
    int argc = 0;
    QGuiApplication app(argc, nullptr);

    QQuickView view(QUrl("qrc:/main.qml"));
    view.setResizeMode(QQuickView::SizeRootObjectToView);
    QObject *item = view.rootObject();

    assert(item != nullptr);

    QObject *dbg_console = qvariant_cast<QObject*>(QQmlProperty::read(item, "dbgConsole"));
    QObject *scrn_view = qvariant_cast<QObject*>(QQmlProperty::read(item, "scrnView"));

    ScreenManager scrnmgr(scrn_view, backend, callbacks);
    QObject::connect(scrn_view, SIGNAL(pauseToggled()), &scrnmgr, SLOT(togglePaused()));

    QObject *dbg_console_text = qvariant_cast<QObject*>(QQmlProperty::read(dbg_console, "text"));
    ConsoleManager consmgr(dbg_console_text);
    QObject::connect(dbg_console, SIGNAL(commandRun(QString)),
                     &consmgr, SLOT(runCommand(QString)));

    view.show();
    int result = app.exec();
    return result;
}

#include "main.moc"