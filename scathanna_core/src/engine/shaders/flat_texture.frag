#version 410 core

precision mediump float;

in highp vec2 f_tex;
out vec4 out_col;
uniform sampler2D usampler;

void main() { out_col = texture(usampler, f_tex); }
