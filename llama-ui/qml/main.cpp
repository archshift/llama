#include <QGuiApplication>
#include <QQmlProperty>
#include <QString>
#include <QTimer>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickView>

#include <cassert>
#include <cstdio>
#include <vector>

#include <lgl.h>

#include "interop.hpp"
#include "screens.hpp"

class ConsoleManager: public QObject
{
    Q_OBJECT

    QObject *dbg_console;
    Backend *backend;
    FrontendCallbacks *callbacks;

    QTimer *text_poll_timer;
    std::vector<char> text_buf;
public slots:
    void fillLog() {
        LogBufferView view = lgl_buffer({ &text_buf[0], text_buf.size() });
        QString txt = QString::fromUtf8(view.buf_ptr, view.buf_size);
        QObject *dbg_console_text = qvariant_cast<QObject*>(QQmlProperty::read(dbg_console, "text"));
        dbg_console_text->setProperty("text", txt);
    }

    void runCommand(const QString &msg) {
        QByteArray utf8 = msg.toUtf8();
        callbacks->run_command(backend, utf8.constData(), utf8.size());
    }

    void useTraceLogs(bool trace) {
        callbacks->use_trace_logs(backend, trace);
    }

public:
    ConsoleManager(QObject *dbg_console, Backend *backend, FrontendCallbacks *callbacks):
            dbg_console(dbg_console),
            backend(backend),
            callbacks(callbacks),
            text_poll_timer(new QTimer(this)),
            text_buf(lgl_buffer_size())
    {
        QObject::connect(text_poll_timer, &QTimer::timeout, this, &ConsoleManager::fillLog);
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

    qmlRegisterType<TopScreen>("Screens", 1, 0, "TopScreen");
    qmlRegisterType<BotScreen>("Screens", 1, 0, "BotScreen");

    QQuickView view(QUrl("qrc:/main.qml"));
    view.setResizeMode(QQuickView::SizeRootObjectToView);
    QObject *item = view.rootObject();

    assert(item != nullptr);

    QObject *dbg_console = qvariant_cast<QObject*>(QQmlProperty::read(item, "dbgConsole"));
    QObject *scrn_view = qvariant_cast<QObject*>(QQmlProperty::read(item, "scrnView"));

    ScreenManager scrnmgr(scrn_view, backend, callbacks);
    QObject::connect(scrn_view, SIGNAL(pauseToggled()), &scrnmgr, SLOT(togglePaused()));

    QTimer *scrn_update_timer = createScreenRepainter(scrn_view, backend, callbacks);
    scrn_update_timer->start(16); // TODO: not ideal

    ConsoleManager consmgr(dbg_console, backend, callbacks);
    QObject::connect(dbg_console, SIGNAL(commandRun(QString)),
                     &consmgr, SLOT(runCommand(QString)));
    QObject::connect(dbg_console, SIGNAL(traceToggled(bool)),
                     &consmgr, SLOT(useTraceLogs(bool)));

    view.show();
    int result = app.exec();
    return result;
}

#include "main.moc"