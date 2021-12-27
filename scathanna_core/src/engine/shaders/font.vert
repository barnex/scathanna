#version 410 core

layout(location = 0) in vec3 vertex_pos;
layout(location = 2) in vec2 vertex_tex_coord;
uniform mat4 proj;
uniform vec2 tex_offset;
uniform vec2 pos_offset;

out vec2 frag_tex_coord;

void main() {
  frag_tex_coord = vertex_tex_coord + tex_offset;
  vec3 pos_offset3 = vec3(pos_offset, 0.0);
  gl_Position = proj * vec4(vertex_pos + pos_offset3, 1.0);
}