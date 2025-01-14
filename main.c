#include <stdint.h>

#include <GLFW/glfw3.h>

#include "state.h"

int main(void) {
        struct app_state state = app_init();

        while (!glfwWindowShouldClose(state.window)) {
                if (glfwGetTime() >= 3.0) {
                        break;
                }
                glfwPollEvents();
        }
        app_cleanup(state);

        return 0;
}
