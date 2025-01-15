#ifndef LEARNVK_DEVICE_H
#define LEARNVK_DEVICE_H

#include <vulkan/vulkan_core.h>

struct queue_family_indices {
        uint32_t graphics;
};

struct queue_family_indices queue_family_find(VkPhysicalDevice device);
int32_t queue_family_is_complete(struct queue_family_indices family_indices);

VkPhysicalDevice physical_device_find_best(VkInstance instance);

VkDevice logical_device_create(VkPhysicalDevice device, const char **layers,
                const uint32_t layer_count);

#endif
