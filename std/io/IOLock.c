#include "IOLock.h"
#include <stddef.h>
#include <rowan.h>

#ifdef linux

size_t rowan_sem_size() {
    return sizeof(sem_t);
}

typedef struct io_lock {
    object_t object;
    sem_t lock;
} io_lock_t;

#endif

size_t lock__get_dash_size() {
    return rowan_sem_size();
}

void std__io__iolock__IOLock__lock(rowan_context_t context, object_t*) {
}

void std__io__iolock__IOLock__release(rowan_context_t context, object_t*) {
}

void custom_drop(object_t*) {
}