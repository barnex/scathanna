#version 410 core

precision highp float;

in vec3 f_col;
out vec4 out_col;

void main() { out_col = vec4(f_col, 1.0); }