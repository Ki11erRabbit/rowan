#ifndef ROWAN_RUNTIME_H
#define ROWAN_RUNTIME_H
#include <stdint.h>
#include <stddef.h>

typedef size_t symbol_t;

// This is just a way of defining what a Rowan object looks like in C.
// All fields should not be manipulated by your code.
// However, This struct can be embedded in your structs to allow for struct punning
typedef struct object {
    // The symbol to a class object. This should not be manipulated directly ever
    symbol_t class;
    // This is to represent the space taken up by a Rust Box<[Reference]>. Therefore this should never be manipulated ever
    uint8_t parent_objects[16];
    // A function for freeing up resources created for this object. Rowanc should have generated a header for you that provides the way of defining this.
    void (*custom_drop)(struct object*);
} object_t;


typedef void* rowan_context_t;

typedef struct string_object {
    object_t object;
    uint64_t length;
    uint64_t capacity;
    uint8_t* buffer;
} string_t;

typedef void* context_t;

object_t* rowan_create_object(unsigned char* class_name);
string_t* rowan_create_string(unsigned char* string_contents);
string_t* rowan_create_empty_string();

void* rowan_get_virtual_function(context_t ctx, object_t* object, unsigned char* class_name, unsigned char* source_class_name, unsigned char* method_name);
void* rowan_get_static_function(context_t ctx, unsigned char* class_name, unsigned char* method_name);

void rowan_set_exception(context_t ctx, object_t* exception);
void rowan_normal_return(context_t ctx);
#endif