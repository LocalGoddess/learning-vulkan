#include <GLFW/glfw3.h>
#include <vulkan/vulkan_core.h>

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
        if (!glfwInit()) {
                printf("error: failed to inititalize glfw\n");
                exit(1);
        }
        
        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        GLFWwindow *window = glfwCreateWindow(800, 600, "Vulkan Window", NULL,
                        NULL);
        if (window == NULL) {
                printf("error: failed to create window\n");
                exit(1);
        }

        uint32_t extension_count = 0;
        vkEnumerateInstanceExtensionProperties(NULL, &extension_count, NULL);
        
        glfwMakeContextCurrent(window);
        while (!glfwWindowShouldClose(window)) {
                glfwPollEvents();
        }

        glfwDestroyWindow(window);
        glfwTerminate();

        return 0;
}
