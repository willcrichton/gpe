#include <cuda.h>
#include <stdio.h>

typedef struct {
  float x;
  float y;
} Point;

typedef struct {
  Point points[];
  int num_points;
} Polygon;

typedef struct {
  Polygon polygons[];
  int num_polygons;
} Encoding;

__global__ void render_kernel() {

}

extern "C" void cuda_render(Encoding img) {
  printf("%d\n", img.num_polygons);
  //printf("%f, %f", p.x, p.y);
}