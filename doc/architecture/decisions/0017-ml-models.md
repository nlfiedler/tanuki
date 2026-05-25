# ML Image Classification Models

- Status: accepted
- Deciders: Nathan Fiedler
- Date: 2026-05-23

## Context

This is not the most comprehensive comparison possibly, it is simply the most
appropriate models available at this time that are general enough to be
appropriate for this application, and do not rely on the availability of a GPU.

### Image classification

Google AI overview of NASNet-Mobile vs MobileNetV2:

> NASNet-Mobile and MobileNetV2 are both highly efficient Convolutional Neural
> Networks (CNNs) designed for image classification on mobile and edge devices.
> While MobileNetV2 features a hand-crafted architecture focused on raw speed,
> NASNet-Mobile utilizes a structure discovered via automated Neural
> Architecture Search, frequently yielding slightly higher classification
> accuracy at the cost of marginally higher complexity.

#### Key Architectural Differences

* **MobileNetV2**: Uses a hand-crafted structure built on inverted residuals and
  linear bottlenecks. It expands the image data to higher dimensions for feature
  extraction, then compresses it in thin bottleneck layers to maintain low
  memory usage and high processing speeds. 
* **NASNet-Mobile**: Designed using reinforcement learning by searching for the most
  effective modular building blocks (cells) on a smaller dataset (like CIFAR-10)
  before transferring it to broader tasks. It heavily leverages parallelized
  operations, like custom reduction and normal cells. 

### Comparative Metrics

| Feature | MobileNetV2 | NASNet-Mobile |
| ------- | ----------- | ------------- |
| Model Size | ~3.4 to 3.5 MB | ~ 5.3 MB |
| Number of Parameters | ~3.4 million | ~5.3 million |
| Typical ImageNet Accuracy	| ~72% to 74% Top-1 Accuracy | ~74% to 75% Top-1 Accuracy |
| Design Approach | Hand-crafted (Human intuition) | Neural Architecture Search (AI) |
| Primary Advantage | Outstanding speed, highly optimized for edge devices. | Often yields marginally higher accuracy and feature fidelity. |

### Face Recognition

Google AI overview of ResNet-100 (R100) vs MobileFaceNet:

> Comparing ResNet-100 (R100) and MobileFaceNet comes down to accuracy versus
> efficiency. R100 is a massive, highly accurate model best suited for heavy
> server workloads, whereas MobileFaceNet is a lightweight, optimized
> architecture built explicitly to run fast on mobile and edge devices.

#### Architecture & Design

* **R100** (ResNet-100): A deep 100-layer residual network. It relies on standard
  convolutional layers and is widely considered the gold standard for high-end
  facial recognition (frequently paired with margin-based loss functions like
  ArcFace). 
* **MobileFaceNet**: Based on MobileNetV2 architecture principles, it scales down
  model size using depthwise separable convolutions and replaces standard global
  average pooling with global depthwise convolution (GDC) to heavily optimize
  specifically for facial geometry. 

#### Performance Comparison

| Feature | ResNet-100 (R100) | MobileFaceNet |
| ------- | ----------------- | ------------- |
| Model Size | ~200 MB to 250 MB | ~4 MB to 10 MB |
| Parameters | ~65 million | ~1 million to 4 million |
| Accuracy (LFW Benchmark) | Extremely High (~99.80% +) | High (~99.00% to 99.3%) |
| Hardware Suitability | Desktop, Server, Cloud (GPU-dependent) | Mobile, Embedded, Edge |
| Speed/Latency | Slower (requires dedicated GPUs for real-time) | Very Fast (runs well on basic mobile CPUs) |

#### Key Trade-offs

* **When to use R100**: Choose R100 for large databases (like 1:N recognition,
  massive surveillance, or secure gate access) where you need to minimize false
  positives and have dedicated hardware (like an NVIDIA GPU) to process the
  heavy matrix calculations. 
* **When to use MobileFaceNet**: Choose MobileFaceNet if you are building an app
  for iOS/Android or running models on IoT devices. It takes up a fraction of
  the memory while still maintaining competitive accuracy for general 1:1 face
  verification. 

## Decision

* **MobileNetV2** for image classification
* **MobileFaceNet** for face recognition

## Consequences

TBD

## Links

- [0004-synthetic-data](../../specs/0004-synthetic-data.md)
