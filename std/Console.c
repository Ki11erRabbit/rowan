
#include "Console.h"
#include <rowan.h>
#include <rowan_runtime.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#ifdef linux
#include <unistd.h>

void rowan_print_internal(int fd, uint8_t* buff, uint64_t length) {
    write(1, buff, length);
}

void rowan_flush(int fd) {
    fsync(fd);
}

#endif



void std__console__Console__print_dash_internal(context_t context, object_t* text) {
    string_t* str = (string_t*)text;
    rowan_print_internal(1, str->buffer, str->length);
    rowan_normal_return(context);
}

void std__console__Console__println_dash_internal(context_t context, object_t* text) {
    string_t* str = (string_t*) text;
    uint8_t* buff = NULL;
    ssize_t length = str->length;
    buff = (uint8_t*)malloc(length + 1);
    memcpy(buff, str->buffer, str->length);
    buff[str->length] = '\n';
    rowan_print_internal(1, buff, str->length + 1);
    rowan_flush(1);
    free(buff);
    buff = NULL;
    rowan_normal_return(context);
}

void std__console__Console__eprint_dash_internal(context_t context, object_t* text) {
    string_t* str = (string_t*)text;
    rowan_print_internal(2, str->buffer, str->length);
    rowan_normal_return(context);
}

void std__console__Console__eprintln_dash_internal(context_t context, object_t* text) {
    string_t* str = (string_t*) text;
    uint8_t* buff = (uint8_t*)malloc(str->length + 1);
    memcpy(buff, str->buffer, str->length);
    buff[str->length] = '\n';
    rowan_print_internal(2, buff, str->length + 1);
    free(buff);
    buff = NULL;
    rowan_normal_return(context);
}