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
    struct object *parent;
    // A function for freeing up resources created for this object. Rowanc should have generated a header for you that provides the way of defining this.
    void (*custom_drop)(struct object*);
} object_t;


typedef void* rowan_context_t;

typedef void* context_t;

typedef struct value {
    uint64_t tag;
    union {
        uint8_t blank;
        uint8_t u8;
        uint16_t u16;
        uint32_t u32;
        uint64_t u64;
        float f32;
        double f64;
        object_t* object;
    }
} rowan_value_t;

void rowan_block_collection(object_t*);
void rowan_allow_collection(object_t*);
object_t* rowan_create_object(unsigned char* class_name);
object_t* rowan_create_empty_string_buffer();
object_t* rowan_create_string_buffer(const unsigned char* string_contents);
void rowan_get_string_buffer(object_t* string, uint8_t** buf, uint64_t* len);

object_t* rowan_create_array(context_t ctx, const unsigned char* type, uint64_t size);
void rowan_get_array_buffer(object_t* array, void** buf, uint64_t* len);

void rowan_set_call_argument(context_t ctx, uint8_t index, rowan_value_t value);

int32_t rowan_call_virtual_function(context_t ctx, object_t* object, unsigned char* class_name, unsigned char* method_name, rowan_value_t *return_slot);
int32_t rowan_call_static_function(context_t ctx, unsigned char* class_name, unsigned char* method_name, rowan_value_t *return_slot);
int32_t rowan_call_interface_function(context_t ctx, unsigned char* interface_name, unsigned char* method_name, rowan_value_t *return_slot);

void rowan_set_exception(context_t ctx, object_t* exception);

int32_t rowan_set_object_field(context_t ctx, object_* object, unsigned char* field, rowan_value_t return_slot);
int32_t rowan_get_object_field(context_t ctx, object_* object, unsigned char* field, rowan_value_t *return_slot);

#endif