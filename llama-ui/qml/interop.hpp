#pragma once

struct Backend;

struct LogBufferView {
    const char* buf_ptr;
    size_t buf_size;
};

struct LogBufferMutView {
    char* buf_ptr;
    size_t buf_size;
};

struct FrontendCallbacks {
    void(*set_running)(Backend*, bool);
    bool(*is_running)(Backend*);
    const uint8_t*(*top_screen)(Backend*, size_t*);
    const uint8_t*(*bot_screen)(Backend*, size_t*);
    void(*run_command)(Backend*, const char*, size_t);
    void(*use_trace_logs)(Backend*, bool);

    void(*reload_game)(Backend*);

    void(*log)(LogBufferView);
    LogBufferView(*buffer)(LogBufferMutView);
    size_t(*buffer_size)();
};