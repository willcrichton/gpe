#include <cuda.h>
#include <stdio.h>

#define THREADS_PER_BLOCK 128

typedef unsigned int u32;
typedef unsigned char u8;

typedef struct {
  u8 r;
  u8 g;
  u8 b;
} Color;

typedef struct {
  float x;
  float y;
} Point;

typedef struct {
  Point* points;
  u32 num_points;
  u8 r; u8 g; u8 b; u8 a;
  Point center;
  float max_dist;
} Polygon;

typedef struct {
  Polygon* polygons;
  u32 num_polygons;
  u32 width;
  u32 height;
  Polygon* dev_ptr;
} Encoding;

__device__ __inline__ u8 add(u32 older, u32 newer, u32 alpha) {
  u32 addend = newer * alpha / 255;
  if (addend + older > 255) { /*printf("%d, %d, %d\n", older, newer, alpha);*/ return 255; }
  else { return addend + older; }
}

__device__ __inline__ void blend(Color* old_color, Color* new_color, u32 alpha) {
  old_color->r = add(old_color->r, new_color->r, alpha);
  old_color->g = add(old_color->g, new_color->g, alpha);
  old_color->b = add(old_color->b, new_color->b, alpha);
}

__device__ __inline__ Color polycolor(Polygon poly, Point pt) {
  Color color;
  float x = pt.x - poly.center.x;
  float y = pt.y - poly.center.y;
  float scale = 1.0 - (x * x + y * y) / poly.max_dist;

  /*if (scale < 0.0 || scale >= 1.0) {
    printf("scale %f, max_dist %f, point (%f, %f), center (%f, %f)\n", scale, poly.max_dist, pt.x, pt.y, poly.center.x, poly.center.y);
    }*/

  color.r = poly.r * scale;
  color.g = poly.g * scale;
  color.b = poly.b * scale;
  return color;
}

__device__ bool query(Point pt, Polygon poly, bool antialias) {
  bool inside = false;
  for (int i = 0; i < poly.num_points; i++) {
    Point a = poly.points[i], b = poly.points[(i + 1) % poly.num_points];
    if ((a.y > pt.y) != (b.y > pt.y) &&
        (pt.x < (b.x - a.x) * (pt.y - a.y) / (b.y - a.y) + a.x)) {
      inside = !inside;
    }
  }

  return inside;
}

__global__ void render_kernel(Encoding* img, Color* output, bool antialias) {
  int pixel = blockDim.x * blockIdx.x + threadIdx.x;
  if (pixel >= img->width * img->height) return;

  Point pt = { pixel % img->width, pixel / img->width };

  for (int i = 0; i < img->num_polygons; i++) {
    Polygon polygon = img->polygons[i];

    if (query(pt, polygon, antialias)) {
      blend(&output[pixel], &polycolor(polygon, pt), polygon.a);
    }
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
    float max_dist = 0.0;
    for (int j = 0; j < img->polygons[i].num_points; j++) {
      float x = img->polygons[i].points[j].x - img->polygons[i].center.x;
      float y = img->polygons[i].points[j].y - img->polygons[i].center.y;
      float dist = x * x + y * y;
      if (dist > max_dist) {
        max_dist = dist;
      }
    }

    img->polygons[i].max_dist = max_dist;
    img->polygons[i].points = points_to_cuda(img->polygons[i]);
  }

  cudaMemcpy(cuda_polygons, img->polygons, sizeof(Polygon) * img->num_polygons, cudaMemcpyHostToDevice);

  return cuda_polygons;
}

Encoding* encoding_to_cuda(Encoding* img) {
  Encoding* cuda_img;
  cudaMalloc(&cuda_img, sizeof(Encoding));

  Polygon* tmp = img->polygons;
  img->polygons = polygons_to_cuda(img);
  cudaMemcpy(cuda_img, img, sizeof(Encoding), cudaMemcpyHostToDevice);

  img->dev_ptr = img->polygons;
  img->polygons = tmp;
  return cuda_img;
}

void encoding_free(Encoding* img) {
  for (int i = 0; i < img->num_polygons; i++) {
    cudaFree(img->polygons[i].points);
  }

  cudaFree(img->dev_ptr);
}

extern "C" void cuda_render(Encoding img, Color* output, bool antialias) {
  u32 N = img.width * img.height;
  size_t size = N * sizeof(Color);

  Color* cuda_output;
  cudaMalloc(&cuda_output, size);
  cudaMemcpy(cuda_output, output, size, cudaMemcpyHostToDevice);

  Encoding* cuda_img = encoding_to_cuda(&img);

  dim3 threadsPerBlock(THREADS_PER_BLOCK, 1);
  dim3 blocksPerGrid((N + threadsPerBlock.x - 1) / threadsPerBlock.x);
  render_kernel<<<blocksPerGrid, threadsPerBlock>>>(cuda_img, cuda_output, antialias);

  cudaMemcpy(output, cuda_output, size, cudaMemcpyDeviceToHost);

  encoding_free(&img);
  cudaFree(cuda_img);
  cudaFree(cuda_output);
}