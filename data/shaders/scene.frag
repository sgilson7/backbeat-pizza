#version 300 es
precision highp float;
// Passes (the core says which, crates/core/src/render.rs): 0 solid, lit in
// flat bands; 1 ink, the dark outline; 2 tint, see-through beams.
// Styles: 0 lit, 1 glow (unlit), 2 floor planks, 3 curtain folds.
// Every tuning number arrives as a uniform from the core.
in vec3 vWorld;
in vec3 vNormal;
in vec4 vColor;
in vec3 vLocal;
in vec3 vSize;
uniform vec3 uLightDir;
uniform vec3 uLight;
uniform vec3 uAmbient;
uniform vec3 uInk;
uniform float uPass;
uniform float uEps;
// bright threshold, dim threshold, the middle band's shade, the dark band's shade
uniform vec4 uBands;
// planks per metre, seam share; folds per metre, dark share
uniform vec4 uPattern;
// share of ambient light, share of direct light, the shade of a seam or fold
uniform vec3 uShade;
out vec4 outColor;

bool is(float style, float k) {
  return abs(style - k) < uEps + uEps;
}

void main() {
  if (uPass > 1.0 + uEps) {
    outColor = vColor;
    return;
  }
  if (uPass > uEps) {
    outColor = vec4(uInk, 1.0);
    return;
  }
  float style = floor(vColor.a + uEps);
  vec3 base = vColor.rgb;
  if (is(style, 1.0)) {
    outColor = vec4(base, 1.0);
    return;
  }
  if (is(style, 1.0 + 1.0)) {
    // floor planks running across the stage
    if (fract(vWorld.z * uPattern.x) < uPattern.y) base *= uShade.z;
  }
  if (is(style, 1.0 + 1.0 + 1.0)) {
    // curtain folds
    float f = fract(vWorld.x * uPattern.z);
    base *= mix(1.0, uShade.z, step(uPattern.w, f));
  }
  vec3 n = normalize(vNormal);
  float diffuse = dot(n, uLightDir);
  float band = diffuse > uBands.x ? 1.0 : (diffuse > uBands.y ? uBands.z : uBands.w);
  vec3 lit = base * (uAmbient * uShade.x + uLight * band * uShade.y);
  outColor = vec4(lit, 1.0);
}
