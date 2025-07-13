#include "IOLock.h"
#include <stddef.h>
#include <stdint.h>
#include <rowan.h>
#include <stdio.h>

#ifdef linux

#include <semaphore.h>

size_t rowan_sem_size() {
    return sizeof(sem_t);
}

typedef struct io_lock {
    object_t object;
    sem_t lock;
    uint8_t set;
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


size_t lock_dash_initalized__get_dash_size() {
    return sizeof(uint8_t);
}

size_t lock__get_dash_size() {
    return rowan_sem_size();
}

void std__io__iolock__IOLock__lock(rowan_context_t context, object_t* self) {
    io_lock_t* io_lock = (io_lock_t*)self;
    if (!io_lock->set) {
        rowan_lock_init(io_lock);
        io_lock->set = 1;
    } else {
    }
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