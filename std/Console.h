#ifndef STD__CONSOLE__CONSOLE_H
#define STD__CONSOLE__CONSOLE_H

#include <rowan.h>
#include <stdint.h>

void std__console__Console__print_dash_internal(rowan_context_t context, object_t*);
void std__console__Console__println_dash_internal(rowan_context_t context, object_t*);
void std__console__Console__eprint_dash_internal(rowan_context_t context, object_t*);
void std__console__Console__eprintln_dash_internal(rowan_context_t context, object_t*);

#endif
