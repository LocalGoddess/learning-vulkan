#include "device.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <vulkan/vulkan_core.h>

int32_t physical_device_suitable(VkPhysicalDevice device);
int32_t physical_device_rate(VkPhysicalDeviceProperties props,
                VkPhysicalDeviceFeatures features);

struct queue_family_indices queue_family_find(VkPhysicalDevice device)
{
        struct queue_family_indices family_indices = { UINT32_MAX };

        uint32_t count;
        vkGetPhysicalDeviceQueueFamilyProperties(device, &count, NULL);

        VkQueueFamilyProperties queue_families[count];
        vkGetPhysicalDeviceQueueFamilyProperties(device, &count, queue_families);
        
        for (int32_t i = 0; i < count; i++) {
                VkQueueFamilyProperties family = queue_families[i];
                if (family.queueFlags & VK_QUEUE_GRAPHICS_BIT) {
                        family_indices.graphics = i;
                }
                
                // Early exit to stop looping :P
                if (queue_family_is_complete(family_indices)) {
                        break;
                }
        }

        return family_indices;
}

int32_t queue_family_is_complete(struct queue_family_indices family_indices)
{
        return family_indices.graphics != UINT32_MAX;
}

VkPhysicalDevice physical_device_find_best(VkInstance instance)
{
        int32_t top_score = -1;
        VkPhysicalDevice top_scorer;

        uint32_t device_count;
        vkEnumeratePhysicalDevices(instance, &device_count, NULL);

        if (device_count == 0) {
                printf("error: failed to find physical devices with vulkan \
                                support\n");
                exit(1);
        }

        VkPhysicalDevice devices[device_count];
        if (vkEnumeratePhysicalDevices(instance, &device_count, devices)
                        != VK_SUCCESS) {
                printf("error: failed to get physical devices\n");
                exit(1);
        }

        for (uint32_t i = 0; i < device_count; i++) {
                VkPhysicalDevice device = devices[i];
                
                VkPhysicalDeviceProperties props;
                VkPhysicalDeviceFeatures features;
                vkGetPhysicalDeviceProperties(device, &props);
                vkGetPhysicalDeviceFeatures(device, &features);
                
                if (!physical_device_suitable(device)) {
                        continue;
                }

                int32_t score = physical_device_rate(props, features);
                if (score > top_score) {
                        top_score = score;
                        top_scorer = device;
                }
        }

        if (top_score == -1) {
                printf("error: failed to find any suitable devices\n");
                exit(1);
        }
        return top_scorer;
}

int32_t physical_device_suitable(VkPhysicalDevice device)
{
        struct queue_family_indices family_indices = queue_family_find(device);

        return queue_family_is_complete(family_indices);
}

int32_t physical_device_rate(VkPhysicalDeviceProperties props,
                VkPhysicalDeviceFeatures features) 
{
        int32_t score = 0;
        if (props.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU) {
                score += 1000;
        }

        score += props.limits.maxImageDimension2D; // The more texture slots the
                                                   // better
        return score;
}
