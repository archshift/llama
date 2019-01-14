#include <QGuiApplication>
#include <QQmlProperty>
#include <QKeyEvent>
#include <QString>
#include <QTimer>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickView>

#include <cassert>
#include <cstdio>
#include <vector>

#include "interop.h"
#include "screens.hpp"

class ConsoleManager: public QObject
{
    Q_OBJECT

    QObject *dbg_console;
    Backend *backend;
    const FrontendCallbacks *callbacks;

    QTimer *text_poll_timer;
    std::vector<char> text_buf;
public slots:
    void fillLog() {
        LogBufferView view = callbacks->buffer({ &text_buf[0], text_buf.size() });
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
    ConsoleManager(QObject *dbg_console, Backend *backend, const FrontendCallbacks *callbacks):
            dbg_console(dbg_console),
            backend(backend),
            callbacks(callbacks),
            text_poll_timer(new QTimer(this)),
            text_buf(callbacks->buffer_size())
    {
        QObject::connect(text_poll_timer, &QTimer::timeout, this, &ConsoleManager::fillLog);
        text_poll_timer->start(100);
    }
};

class ScreenManager: public QObject
{
    Q_OBJECT

    QObject *screen_view;
    Backend *backend;
    const FrontendCallbacks *callbacks;
public slots:
    void togglePaused() {
        bool val = !callbacks->is_running(backend);
        callbacks->set_running(backend, val);
    }

    void reloadGame() {
        callbacks->reload_game(backend);
    }

protected:
    bool handleKey(QKeyEvent* event, bool pressed) {
        if (event->isAutoRepeat()) return false;

        Button button;
        switch(event->key()) {
            case Qt::Key::Key_A: button = BUTTON_A; break;
            case Qt::Key::Key_S: button = BUTTON_B; break;
            case Qt::Key::Key_Z: button = BUTTON_X; break;
            case Qt::Key::Key_X: button = BUTTON_Y; break;
            case Qt::Key::Key_Q: button = BUTTON_L; break;
            case Qt::Key::Key_W: button = BUTTON_R; break;

            case Qt::Key::Key_Up: button = BUTTON_UP; break;
            case Qt::Key::Key_Down: button = BUTTON_DOWN; break;
            case Qt::Key::Key_Left: button = BUTTON_LEFT; break;
            case Qt::Key::Key_Right: button = BUTTON_RIGHT; break;

            case Qt::Key::Key_M: button = BUTTON_START; break;
            case Qt::Key::Key_N: button = BUTTON_SELECT; break;
            default: return false;
        }
        event->accept();
        callbacks->mod_button(backend, button, pressed);
        return true;
    }

    bool eventFilter(QObject *obj, QEvent *event)
    {
        if (event->type() == QEvent::KeyPress) {
            return handleKey(static_cast<QKeyEvent*>(event), true);
        } else if (event->type() == QEvent::KeyRelease) {
            return handleKey(static_cast<QKeyEvent*>(event), false);
        } else {
            // standard event processing
            return QObject::eventFilter(obj, event);
        }
    }
public:
    ScreenManager(QObject *screen_view, Backend *backend, const FrontendCallbacks *callbacks):
            screen_view(screen_view), backend(backend), callbacks(callbacks) { }
};


extern "C" int llama_open_gui(int argc, char **argv, Backend *backend, const FrontendCallbacks *callbacks) {
    QGuiApplication app(argc, argv);

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
    QObject::connect(scrn_view, SIGNAL(reloaded()), &scrnmgr, SLOT(reloadGame()));
    scrn_view->installEventFilter(&scrnmgr);

    initScreenRepainter(scrn_view, backend, callbacks);

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
