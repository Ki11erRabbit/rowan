#ifndef STD__IO__IOLOCK__IOLOCK_H
#define STD__IO__IOLOCK__IOLOCK_H

#include <rowan_runtime.h>
#include <stdint.h>
#include <stddef.h>

size_t lock__get_dash_size();
void std__io__iolock__IOLock__create_dash_internal(rowan_context_t context, object_t*);
void std__io__iolock__IOLock__lock(rowan_context_t context, object_t*);
void std__io__iolock__IOLock__release(rowan_context_t context, object_t*);
void custom_drop(object_t*);

#endif
