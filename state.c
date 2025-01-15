#include "state.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <GLFW/glfw3.h>

#include "util.h"
#include "vulkan_extensions.h"

#ifndef DEBUG
static const char *vk_extension_layers[0];
static const uint32_t vk_extension_layer_count = 0; 
#else
static const char *vk_extension_layers[1] = { "VK_LAYER_KHRONOS_validation" };
static const uint32_t vk_extension_layer_count = 1;
#endif

VkInstance create_vk_instance(void);
uint32_t check_layer_support(const char **layers, uint32_t layer_count);
const char **get_required_extensions(uint32_t *count);

VkDebugUtilsMessengerEXT create_debug_extension(VkInstance vk_instance);
VKAPI_ATTR VkBool32 VKAPI_CALL debug_callback(
                VkDebugUtilsMessageSeverityFlagBitsEXT,
                VkDebugUtilsMessageTypeFlagsEXT,
                const VkDebugUtilsMessengerCallbackDataEXT*, void*);


struct app_state app_init(void) {
        struct app_state state = { 0 };

        if (!glfwInit()) {
                printf("error: failed to initialize glfw.\n");
                exit(1);
        }

        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);

        GLFWwindow *window = glfwCreateWindow(800, 700, "Vulkan Window", NULL,
                        NULL);

        if (window == NULL) {
                printf("error: failed to initialize the window.\n");
                exit(1);
        }
        
        VkInstance vk_instance = create_vk_instance();
        
#ifdef DEBUG
        VkDebugUtilsMessengerEXT extension = create_debug_extension(vk_instance);
        state.vk_debug_messenger = extension;
#endif

        state.window = window;
        state.vk_instance = vk_instance;
        return state;
}

void app_cleanup(struct app_state state) {
#ifdef DEBUG
        vkDestroyDebugUtilsMessengerEXT(state.vk_instance,
               state.vk_debug_messenger, NULL);
#endif

        vkDestroyInstance(state.vk_instance, NULL);

        glfwDestroyWindow(state.window);
        glfwTerminate();
}

VkInstance create_vk_instance(void) {
        VkApplicationInfo app_info = { 0 };
        app_info.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
        app_info.pApplicationName = "Learning Vulkan (With C)";
        app_info.applicationVersion = VK_MAKE_VERSION(1, 0, 0);
        app_info.pEngineName = "No Engine";
        app_info.engineVersion = VK_MAKE_VERSION(1, 0, 0);
        app_info.apiVersion = VK_API_VERSION_1_0;
        
        VkInstanceCreateInfo create_info = { 0 };
        create_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
        create_info.pApplicationInfo = &app_info;
        
        uint32_t extension_count;
        const char **extensions;
        extensions = get_required_extensions(&extension_count);
        
        if (!check_layer_support(vk_extension_layers, vk_extension_layer_count)) {
                printf("error: requested layers not available\n");
                exit(1);
        }


        create_info.ppEnabledExtensionNames = extensions;
        create_info.enabledExtensionCount = extension_count;
        create_info.ppEnabledLayerNames = vk_extension_layers;
        create_info.enabledLayerCount = vk_extension_layer_count;
        
        VkInstance vk_instance;
        if (vkCreateInstance(&create_info, NULL, &vk_instance) != VK_SUCCESS) {
                printf("error: failed to create a vulkan instance.\n");
                exit(1);
        }
        
        free(extensions);
        return vk_instance;
}

uint32_t check_layer_support(const char **layers, uint32_t layer_count) {
        uint32_t av_layer_count;
        vkEnumerateInstanceLayerProperties(&av_layer_count, NULL);

        VkLayerProperties available_layers[av_layer_count];
        vkEnumerateInstanceLayerProperties(&av_layer_count, available_layers);

        ARRAY_FOR_EACH_DYN(const char *layer, layers, layer_count) {
                int32_t found = 0;
                ARRAY_FOR_EACH(const VkLayerProperties prop, available_layers) {
                        if (strcmp(layer, prop.layerName) == 0) {
                                found = 1;
                                break;
                        }
                }

                if (!found) {
                        return 0;
                }
        }

        return 1;
}

const char **get_required_extensions(uint32_t *count) {
        const char **glfw_extensions;
        uint32_t glfw_ext_count;

        glfw_extensions = glfwGetRequiredInstanceExtensions(&glfw_ext_count);
        const char **extensions = malloc(sizeof(const char *) *
                        (glfw_ext_count + 1));
        if (!extensions) {
                printf("error: failed to allocate enough memory for vk \
                                extension names\n");
                exit(1);
        }

        memcpy(extensions, glfw_extensions, sizeof(const char *) *
                        glfw_ext_count);
        
#ifdef DEBUG
        extensions[glfw_ext_count] = VK_EXT_DEBUG_UTILS_EXTENSION_NAME;
        *count = glfw_ext_count + 1;
#else
        *count = glfw_ext_count;
#endif
        return extensions;
}

VkDebugUtilsMessengerEXT create_debug_extension(VkInstance vk_instance) {
        VkDebugUtilsMessengerEXT messenger_ext = { 0 };

        VkDebugUtilsMessengerCreateInfoEXT create_info = { 0 };
        create_info.sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
        create_info.messageSeverity = 
                VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT |
                VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
                VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;

        create_info.messageType =
                VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT    |
                VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;

        create_info.pfnUserCallback = debug_callback;

        if (vkCreateDebugUtilsMessengerEXT(vk_instance, &create_info, NULL,
                                &messenger_ext) != VK_SUCCESS) {
                printf("error: failed to create debug utils messenger\n");
                exit(1);
        }
        
        return messenger_ext;
}

VKAPI_ATTR VkBool32 VKAPI_CALL debug_callback(
                VkDebugUtilsMessageSeverityFlagBitsEXT severity,
                VkDebugUtilsMessageTypeFlagsEXT type,
                const VkDebugUtilsMessengerCallbackDataEXT *callback_data,
                void *user_data) {
        if (severity >= VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT) {
                printf("Validation Layer: %s\n", callback_data->pMessage);
        }

        return VK_FALSE;
}


