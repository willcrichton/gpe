#include <cuda.h>
#include <stdio.h>

typedef unsigned int uint;

typedef struct {
  char r;
  char g;
  char b;
} Color;

typedef struct {
  float x;
  float y;
} Point;

typedef struct {
  Point* points;
  uint num_points;
  char r; char g; char b; char a;
} Polygon;

typedef struct {
  Polygon* polygons;
  uint num_polygons;
  uint width;
  uint height;
} Encoding;

__device__ __inline__ char add(uint older, uint newer, uint alpha) {
  uint addend = newer * alpha / 255;
  if (addend + older > 255) { return 255; }
  else { return addend + older; }
}

__device__ __inline__ Color blend(Color old_color, Polygon poly) {
  Color color;
  color.r = add(old_color.r, poly.r, poly.a);
  color.g = add(old_color.g, poly.g, poly.a);
  color.b = add(old_color.b, poly.b, poly.a);
  return color;
}

__global__ void render_kernel(Encoding* img, Color* output, bool antialias) {
  int pixel = blockDim.x * blockIdx.x + threadIdx.x;
  if (pixel >= img->width * img->height) return;

  for (int i = 0; i < img->num_polygons; i++) {
    Polygon polygon = img->polygons[i];

    // TODO: polygon containment test
    output[pixel] = blend(output[pixel], polygon);
  }
}

Point* points_to_cuda(Polygon polygon) {
  Point* cuda_points;
  cudaMalloc(&cuda_points, sizeof(Point) * polygon.num_points);
  cudaMemcpy(cuda_points, polygon.points, sizeof(Point) * polygon.num_points, cudaMemcpyHostToDevice);

  return cuda_points;
}

Polygon* polygons_to_cuda(Encoding* img) {
  Polygon* cuda_polygons;
  cudaMalloc(&cuda_polygons, sizeof(Polygon) * img->num_polygons);

  for (int i = 0; i < img->num_polygons; i++) {
    img->polygons[i].points = points_to_cuda(img->polygons[i]);
  }

  cudaMemcpy(cuda_polygons, img->polygons, sizeof(Polygon) * img->num_polygons, cudaMemcpyHostToDevice);

  return cuda_polygons;
}

Encoding* encoding_to_cuda(Encoding* img) {
  Encoding* cuda_img;
  cudaMalloc(&cuda_img, sizeof(Encoding));

  img->polygons = polygons_to_cuda(img);
  cudaMemcpy(cuda_img, img, sizeof(Encoding), cudaMemcpyHostToDevice);

  return cuda_img;
}

extern "C" void cuda_render(Encoding img, Color* output, bool antialias) {
  uint N = img.width * img.height;
  size_t size = N * sizeof(Color);

  Color* cuda_output;
  cudaMalloc(&cuda_output, size);
  cudaMemcpy(cuda_output, output, size, cudaMemcpyHostToDevice);

  Encoding* cuda_img = encoding_to_cuda(&img);

  dim3 threadsPerBlock(256, 1);
  dim3 blocksPerGrid((N + threadsPerBlock.x - 1) / threadsPerBlock.x);
  render_kernel<<<blocksPerGrid, threadsPerBlock>>>(cuda_img, cuda_output, antialias);

  cudaMemcpy(output, cuda_output, size, cudaMemcpyDeviceToHost);

  cudaFree(cuda_output);
  cudaFree(cuda_img);
}