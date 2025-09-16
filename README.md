# Stunts Native

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)]()

A high-performance native desktop application for creating animations, motion graphics, and video content using natural language descriptions. Built with Rust and a custom reactive GUI framework.

## Features

- **Natural Language Animation**: Describe animations in plain English - no manual keyframes needed
- **Visual Path Direction**: Place arrows to guide object movement through your scene
- **Screen Capture and Zoom**: Capture screen content with beautiful mouse zooms included
- **High-Performance Rendering**: Built on CommonUI and Vello for GPU-accelerated 2D graphics
- **Video Export**: Export videos to HD MP4
- **Object Management**: Work with polygons, images, videos, and text elements

## How It Works

Stunts revolutionizes animation creation through natural language and visual direction:

1. **Place Visual Guides**: Drop arrows in your scene to show where objects should move
2. **Describe in Natural Language**: Use simple descriptions for properties:
   - **Position**: "Move along the path, bouncing slightly"
   - **Rotation**: "Spin clockwise while moving" 
   - **Opacity**: "Fade in at the start, fade out at the end"
   - **Scale**: "Start small, grow larger over time"
   - **Object Description**: "A red bouncing ball" or "Corporate logo"

3. **Generate & Iterate**: Hit "Generate" to create your animation, or "Regenerate" to refine it

### Example Workflow

Want a bouncing ball that grows as it moves? Simply:
- Place arrow guides showing the bounce path
- Set position to: "Follow the path with realistic bouncing physics"
- Set scale to: "Start at 50% size, grow to 150% by the end"
- Set object to: "A bright red rubber ball"
- Hit Generate!

No timeline scrubbing, no manual keyframes - just describe what you want to see.

## Prerequisites

- **Rust**: Latest stable version (1.75+)
- **GPU**: DirectX 11/12, Vulkan, or Metal compatible graphics card
- **OS**: Windows, macOS, or Linux

## Project Structure

This project is part of a multi-repository ecosystem:

```
projects/common/
├── stunts-native/     # Main native application (this repo)
├── stunts-engine/     # Core animation and rendering engine
├── commonui/          # Custom reactive GUI framework
└── vello/            # 2D graphics rendering library
```

## Setup

### 1. Clone the Repository Ecosystem

To get started, you'll need to clone all the sister repositories alongside each other:

```bash
# Create a common directory for all repositories
mkdir projects/common
cd projects/common

# Clone all repositories
git clone <stunts-native-repo-url> stunts-native
git clone <stunts-engine-repo-url> stunts-engine  
git clone <commonui-repo-url> commonui
git clone <vello-repo-url> vello
```

The directory structure should look like:
```
projects/common/
├── stunts-native/
├── stunts-engine/
├── commonui/
└── vello/
```

### 2. Build and Run Stunts Native

```bash
cd stunts-native
cargo build --release
cargo run
```

## Development

### Quick Start

```bash
# For development builds (faster compilation)
cargo run

# For optimized builds
cargo run --release
```

### Project Dependencies

- **stunts-engine**: Core animation engine with timeline, objects, and export functionality
- **commonui**: GPU-accelerated 2D graphics rendering
- **wgpu**: Cross-platform graphics API
- **winit**: Cross-platform windowing
- **tokio**: Async runtime for background tasks

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Vello](https://github.com/linebender/vello) - 2D graphics rendering
- [Taffy](https://github.com/DioxusLabs/taffy) - Layout engine used in CommonUI  
- [wgpu](https://github.com/gfx-rs/wgpu) - Cross-platform graphics API
- [winit](https://github.com/rust-windowing/winit) - Cross-platform windowing

## Support

- Create an issue for bug reports or feature requests
- Check existing issues before creating new ones
- Provide detailed information including OS, GPU, and reproduction steps