#version 300 es
// One shape: the mesh vertex placed by the instance's three scaled axes and
// translation (iA, iB, iC; translation in .w), coloured by iD (rgb, and in
// .a the style, or for a see-through shape its opacity). Every value
// arrives as the core's Q16 integer; uQ turns it into a number.
in vec3 aPos;
in vec3 aNormal;
in vec4 iA;
in vec4 iB;
in vec4 iC;
in vec4 iD;
uniform mat4 uVP;
uniform float uPass;
uniform float uQ;
uniform float uEps;
// the ink line: least width, growth per metre of the shape, most width
uniform vec3 uInkSize;
out vec3 vWorld;
out vec3 vNormal;
out vec4 vColor;
out vec3 vLocal;
out vec3 vSize;
void main() {
  vec3 a = iA.xyz * uQ;
  vec3 b = iB.xyz * uQ;
  vec3 c = iC.xyz * uQ;
  vec3 t = vec3(iA.w, iB.w, iC.w) * uQ;
  vec3 p = aPos * uQ;
  vec3 n = aNormal * uQ;
  vec3 size = vec3(length(a), length(b), length(c));
  vLocal = p;
  if (uPass > 0.0 && uPass < 1.0 + 1.0) {
    if (uPass < 1.0 + uEps) {
      // the inverted hull: each axis grown by the line's width on both sides
      float big = max(size.x, max(size.y, size.z));
      float w = clamp(uInkSize.x + big * uInkSize.y, uInkSize.x, uInkSize.z);
      p *= (size + w + w) / max(size, vec3(uEps));
    }
  }
  vec3 world = a * p.x + b * p.y + c * p.z + t;
  vNormal = a * (n.x / max(dot(a, a), uEps)) + b * (n.y / max(dot(b, b), uEps)) + c * (n.z / max(dot(c, c), uEps));
  vSize = size;
  vColor = iD * uQ;
  vWorld = world;
  gl_Position = uVP * vec4(world, 1.0);
}
