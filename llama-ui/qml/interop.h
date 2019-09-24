#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>

#pragma once

typedef struct Backend Backend;

typedef struct LogBufferView {
    const char* buf_ptr;
    size_t buf_size;
} LogBufferView;

typedef struct LogBufferMutView {
    char* buf_ptr;
    size_t buf_size;
} LogBufferMutView;

enum Button {
    BUTTON_A,
    BUTTON_B,
    BUTTON_X,
    BUTTON_Y,
    BUTTON_L,
    BUTTON_R,
    BUTTON_UP,
    BUTTON_DOWN,
    BUTTON_LEFT,
    BUTTON_RIGHT,
    BUTTON_SELECT,
    BUTTON_START
};

enum ColorFormat {
    COLOR_RGBA8,
    COLOR_RGB8,
    COLOR_RGB565,
    COLOR_RGB5A1,
    COLOR_RGBA4
};

typedef struct FrontendCallbacks {
    void(*set_running)(Backend*, bool);
    bool(*is_running)(Backend*);
    void(*reload_game)(Backend*);

    const uint8_t*(*top_screen)(Backend*, size_t*, enum ColorFormat*);
    const uint8_t*(*bot_screen)(Backend*, size_t*, enum ColorFormat*);
    void(*mod_button)(Backend*, Button, bool);

    void(*run_command)(Backend*, const char*, size_t);
    void(*use_trace_logs)(Backend*, bool);
    void(*log)(LogBufferView);
    LogBufferView(*buffer)(LogBufferMutView);
    size_t(*buffer_size)();
} FrontendCallbacks;

#ifdef _MSC_VER
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

#ifdef __cplusplus
extern "C" {
#endif
    int EXPORT llama_open_gui(int argc, char **argv, Backend *backend, const FrontendCallbacks *callbacks);
#ifdef __cplusplus
}
#endif
