#pragma once

struct Backend;
struct FrontendCallbacks {
    void(*set_running)(Backend*, bool);
    bool(*is_running)(Backend*);
    const uint8_t*(*top_screen)(Backend*, size_t*);
    const uint8_t*(*bot_screen)(Backend*, size_t*);
    void(*run_command)(Backend*, const char*, size_t);
    void(*use_trace_logs)(Backend*, bool);
};