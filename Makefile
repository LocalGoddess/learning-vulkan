BUILD_DIR	= build
TARGET 		= learning-vulkan


OBJECTS 	= $(patsubst %.c, $(BUILD_DIR)/%.o, $(wildcard *.c))
HEADERS		= $(wildcard *.h)

CC 		= gcc
CFLAGS		= -Wall -Werror -DGLFW_INCLUDE_VULKAN
LDFLAGS		= -lglfw -lvulkan

.PRECIOUS: $(BUILD_DIR)/$(TARGET) $(OBJECTS)
.PHONY: release debug executable clean

debug: CFLAGS 	+= -DDEBUG -g
debug: executable

release: CFLAGS += -O2
release: executable

executable: $(BUILD_DIR)/$(TARGET)



$(BUILD_DIR)/%.o: %.c $(HEADERS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/$(TARGET): $(OBJECTS)
	$(CC) $(OBJECTS) -o $@ $(LDFLAGS)


clean:
	-rm -r $(BUILD_DIR)/*.o
	-rm -r $(BUILD_DIR)/$(TARGET)
