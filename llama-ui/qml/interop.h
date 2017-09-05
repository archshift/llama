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

typedef struct FrontendCallbacks {
    void(*set_running)(Backend*, bool);
    bool(*is_running)(Backend*);
    void(*reload_game)(Backend*);

    const uint8_t*(*top_screen)(Backend*, size_t*);
    const uint8_t*(*bot_screen)(Backend*, size_t*);
    void(*mod_button)(Backend*, Button, bool);

    void(*run_command)(Backend*, const char*, size_t);
    void(*use_trace_logs)(Backend*, bool);
    void(*log)(LogBufferView);
    LogBufferView(*buffer)(LogBufferMutView);
    size_t(*buffer_size)();
} FrontendCallbacks;

#ifdef __cplusplus
extern "C" {
#endif
    int llama_open_gui(Backend *backend, const FrontendCallbacks *callbacks);
#ifdef __cplusplus
}
#endif