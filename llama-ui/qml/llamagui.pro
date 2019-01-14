TEMPLATE = lib
SOURCES += screens.cpp main.cpp
HEADERS += screens.hpp interop.h
RESOURCES = qml.qrc
CONFIG += c++11 qtquickcompiler release force_debug_info
QT += qml quick
QMAKE_LN_SHLIB = :
CONFIG += unversioned_libname unversioned_soname
