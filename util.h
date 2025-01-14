#ifndef LEARNVK_UTIL_H
#define LEARNVK_UTIL_H

#include <stddef.h>

#define ARRAY_FOR_EACH(item, array) \
        for (size_t keep = 1, count = 0, size = sizeof(array) /\
                        sizeof(array[0]); keep && count != size; \
                keep = !keep, count++) \
                for(item = *(array + count); keep; keep = !keep)

#define ARRAY_FOR_EACH_DYN(item, array, size) \
        for (size_t keep = 1, count = 0; keep && count != size; \
                keep = !keep, count++) \
                for (item = *(array + count); keep; keep = !keep)

#endif
