compile_shaders:
    mkdir -p slangc_output
    slangc src/shaders/shader.slang -target spirv -entry vertexMain -stage vertex -o slangc_output/shader.vert.spv -reflection-json slangc_output/vertex_reflection.json
    slangc src/shaders/shader.slang -target spirv -entry fragmentMain -stage fragment -o slangc_output/shader.frag.spv -reflection-json slangc_output/fragment_reflection.json

show_bindings:
    @echo "Vertex:"
    @jq -r '.parameters[] | select(.type.kind == "parameterBlock") | "\(.binding.index): \(.name)"' slangc_output/vertex_reflection.json
    @echo "Fragment:"
    @jq -r '.parameters[] | select(.type.kind == "parameterBlock") | "\(.binding.index): \(.name)"' slangc_output/fragment_reflection.json