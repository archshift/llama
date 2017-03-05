TEMPLATE = lib
SOURCES += screens.cpp main.cpp
HEADERS += screens.hpp interop.hpp
RESOURCES = qml.qrc
QT += qml quick

INCLUDEPATH += $$(LGL_INC_DIR)
LIBS += -L$$(LGL_LIB_DIR) -l$$(LGL_LIB)