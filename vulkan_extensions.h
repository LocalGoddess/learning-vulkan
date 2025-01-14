#ifndef LEARNVK_VULKAN_EXTENSIONS_H
#define LEARNVK_VULKAN_EXTENSIONS_H

#include <vulkan/vulkan.h>
#include <vulkan/vulkan_core.h>

//////////////////////
/// Debug Messenger
/////////////////////

VKAPI_ATTR VkResult VKAPI_CALL vkCreateDebugUtilsMessengerEXT(
                VkInstance instance,
                const VkDebugUtilsMessengerCreateInfoEXT *create_info,
                const VkAllocationCallbacks *allocator,
                VkDebugUtilsMessengerEXT *messenger) {
        PFN_vkCreateDebugUtilsMessengerEXT func = (PFN_vkCreateDebugUtilsMessengerEXT) vkGetInstanceProcAddr(instance, "vkCreateDebugUtilsMessengerEXT");
        if (func != NULL) {
                return func(instance, create_info, allocator, messenger);
        } else {
                return VK_ERROR_EXTENSION_NOT_PRESENT;
        }
}

VKAPI_ATTR void VKAPI_CALL vkDestroyDebugUtilsMessengerEXT(VkInstance instance,
                VkDebugUtilsMessengerEXT messenger, 
                const VkAllocationCallbacks *allocator) {
        PFN_vkDestroyDebugUtilsMessengerEXT func = (PFN_vkDestroyDebugUtilsMessengerEXT) vkGetInstanceProcAddr(instance, "vkDestroyDebugUtilsMessengerEXT");
        if (func != NULL) {
                func(instance, messenger, allocator);
        } 
}

#endif
