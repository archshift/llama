cmake_minimum_required(VERSION 3.1.0)

project(llamagui)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt5 COMPONENTS Quick Core)
find_package(Qt5QuickCompiler)

set(qtui_SRCS
    main.cpp
	 screens.cpp
	 qml.qrc
)

qtquick_compiler_add_resources(RESOURCES qml.qrc)
add_library(llamagui SHARED ${qtui_SRCS})
qt5_use_modules(llamagui Quick Core)
