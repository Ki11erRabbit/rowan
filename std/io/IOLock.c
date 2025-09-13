#include "IOLock.h"
#include <stddef.h>
#include <stdint.h>
#include <rowan.h>
#include <rowan_runtime.h>

#ifdef __linux__

#include <semaphore.h>

size_t rowan_sem_size() {
    return sizeof(sem_t);
}

typedef struct io_lock {
    object_t object;
    sem_t lock;
} io_lock_t;

void rowan_lock_init(io_lock_t* io_lock) {
    sem_init(&io_lock->lock, 0, 0);
    sem_post(&io_lock->lock);
}

void rowan_acquire_lock(io_lock_t* io_lock) {
    sem_wait(&io_lock->lock);
}

void rowan_release_lock(io_lock_t* io_lock) {
    sem_post(&io_lock->lock);
}

void rowan_lock_destroy(io_lock_t* io_lock) {
    sem_destroy(&io_lock->lock);
}

#endif

#ifdef __APPLE__

#include <dispatch/dispatch.h>

size_t rowan_sem_size() {
    return sizeof(dispatch_semaphore_t);
}

typedef struct io_lock {
    object_t object;
    dispatch_semaphore_t lock;
} io_lock_t;

void rowan_lock_init(io_lock_t* io_lock) {
    io_lock->lock = dispatch_semaphore_create(0);
    dispatch_semaphore_signal(io_lock->lock);
}

void rowan_acquire_lock(io_lock_t* io_lock) {
    dispatch_semaphore_wait(io_lock->lock, DISPATCH_TIME_FOREVER);
}

void rowan_release_lock(io_lock_t* io_lock) {
    dispatch_semaphore_signal(io_lock->lock);
}

void rowan_lock_destroy(io_lock_t* io_lock) {
    dispatch_release(io_lock->lock);
}

#endif

size_t lock__get_dash_size() {
    return rowan_sem_size();
}

void std__io__iolock__IOLock__create_dash_internal(rowan_context_t context, object_t *self) {
    io_lock_t *lock = (io_lock_t*)self;
    rowan_lock_init(lock);

}

void std__io__iolock__IOLock__lock(rowan_context_t context, object_t* self) {
    io_lock_t* io_lock = (io_lock_t*)self;
    rowan_acquire_lock(io_lock);
}

void std__io__iolock__IOLock__release(rowan_context_t context, object_t* self) {
    io_lock_t* io_lock = (io_lock_t*)self;
    rowan_release_lock(io_lock);
}

void custom_drop(object_t* self) {
    io_lock_t* io_lock = (io_lock_t*)self;
    rowan_lock_destroy(io_lock);
}