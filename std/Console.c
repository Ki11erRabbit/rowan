
#include "Console.h"
#include <rowan.h>
#include <rowan_runtime.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#ifdef __linux__
#include <unistd.h>

void rowan_print_internal(int fd, uint8_t* buff, uint64_t length) {
    write(1, buff, length);
}

void rowan_flush(int fd) {
    fsync(fd);
}

#endif

#ifdef __APPLE__
#include <unistd.h>

void rowan_print_internal(int fd, uint8_t* buff, uint64_t length) {
    write(1, buff, length);
}

void rowan_flush(int fd) {
    fsync(fd);
}

#endif



void std__console__Console__print_dash_internal(context_t context, object_t* text) {
    uint64_t length = 0;
    uint8_t* buf = NULL;

    rowan_get_string_buffer(text, &buf, &length);

    rowan_print_internal(1, buf, length);
}

void std__console__Console__println_dash_internal(context_t context, object_t* text) {
    uint64_t length = 0;
    uint8_t* buf = NULL;

    rowan_get_string_buffer(text, &buf, &length);

    rowan_print_internal(1, buf, length);
    rowan_print_internal(1, "\n", 1);
    rowan_flush(1);
}

void std__console__Console__eprint_dash_internal(context_t context, object_t* text) {
    uint64_t length = 0;
    uint8_t* buf = NULL;

    rowan_get_string_buffer(text, &buf, &length);

    rowan_print_internal(2, buf, length);
    rowan_print_internal(2, "\n", 1);
}

void std__console__Console__eprintln_dash_internal(context_t context, object_t* text) {
    uint64_t length = 0;
    uint8_t* buf = NULL;

    rowan_get_string_buffer(text, &buf, &length);

    rowan_print_internal(2, buf, length);
    rowan_print_internal(2, "\n", 1);
}