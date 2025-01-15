#ifndef LEARNVK_STATE_H
#define LEARNVK_STATE_H

#include <stdint.h>

#include <GLFW/glfw3.h>
#include <vulkan/vulkan.h>

struct app_state {
        GLFWwindow *window;

        VkInstance vk_instance;
        
        VkPhysicalDevice phyisical_device;
        VkDevice logical_device;

        VkQueue graphics_queue;

        VkDebugUtilsMessengerEXT vk_debug_messenger;
};

struct app_state app_init(void);
void app_cleanup(struct app_state state);

#endif

