#ifndef ROWAN_H
#define ROWAN_H


#include <stdint.h>
#include <stddef.h>

typedef size_t symbol_t;

// This is just a way of defining what a Rowan object looks like in C.
// All fields should not be manipulated by your code.
// However, This struct can be embedded in your structs to allow for struct punning
struct object {
    // The symbol to a class object. This should not be manipulated directly ever
    symbol_t class;
    // This is to represent the space taken up by a Rust Box<[Reference]>. Therefore this should never be manipulated ever
    uint8_t parent_objects[16];
    // A function for freeing up resources created for this object. Rowanc should have generated a header for you that provides the way of defining this.
    void (*custom_drop)(struct object*);
};

typedef struct object object_t;

extern struct context_t;

typedef struct context* rowan_context_t;

#endif
