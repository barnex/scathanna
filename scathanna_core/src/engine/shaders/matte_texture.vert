#version 410 core

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_tex;
uniform mat4 model;
uniform mat4 proj;

out vec3 f_normal;
out vec2 f_tex;

void main() {
  gl_Position = proj * (model * vec4(v_pos, 1.0));
  f_normal =
      (model * vec4(v_normal, 0.0)).xyz; // assumes orthonormal transform.
  f_tex = v_tex;
}