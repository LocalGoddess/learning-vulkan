if [[ -z "${VULKAN_SDK}" ]]; then
    echo "You need the Vulkan SDK in order to run this script"
    return
fi

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "yay"
else
    echo "This  script may not work on your system"
fi

echo "Compiling shaders using glslc"
$VULKAN_SDK/bin/glslc -fshader-stage=vert basic_shader.vert.glsl -o vert.spv
$VULKAN_SDK/bin/glslc -fshader-stage=frag basic_shader.frag.glsl -o frag.spv
