#ifndef LEARNVK_STATE_H
#define LEARNVK_STATE_H

#include <stdint.h>

#include <GLFW/glfw3.h>
#include <vulkan/vulkan.h>

struct app_state {
        VkInstance vk_instance;
        VkDebugUtilsMessengerEXT vk_debug_messenger;
        GLFWwindow *window;
};

struct app_state app_init(void);
void app_cleanup(struct app_state state);

#endif

