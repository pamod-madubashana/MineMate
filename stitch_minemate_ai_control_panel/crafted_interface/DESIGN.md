---
name: Crafted Interface
colors:
  surface: '#131313'
  surface-dim: '#131313'
  surface-bright: '#393939'
  surface-container-lowest: '#0e0e0e'
  surface-container-low: '#1b1b1b'
  surface-container: '#202020'
  surface-container-high: '#2a2a2a'
  surface-container-highest: '#353535'
  on-surface: '#e5e2e1'
  on-surface-variant: '#bbcbb3'
  inverse-surface: '#e5e2e1'
  inverse-on-surface: '#303030'
  outline: '#86957f'
  outline-variant: '#3c4b38'
  surface-tint: '#33e43d'
  primary: '#f6ffef'
  on-primary: '#003a04'
  primary-container: '#55ff55'
  on-primary-container: '#007311'
  inverse-primary: '#006e10'
  secondary: '#c7c6c6'
  on-secondary: '#2f3031'
  secondary-container: '#464747'
  on-secondary-container: '#b5b5b5'
  tertiary: '#fffbff'
  on-tertiary: '#68000c'
  tertiary-container: '#ffd6d3'
  on-tertiary-container: '#bd242c'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#74ff6b'
  primary-fixed-dim: '#33e43d'
  on-primary-fixed: '#002202'
  on-primary-fixed-variant: '#005309'
  secondary-fixed: '#e3e2e2'
  secondary-fixed-dim: '#c7c6c6'
  on-secondary-fixed: '#1b1c1c'
  on-secondary-fixed-variant: '#464747'
  tertiary-fixed: '#ffdad7'
  tertiary-fixed-dim: '#ffb3ae'
  on-tertiary-fixed: '#410005'
  on-tertiary-fixed-variant: '#930016'
  background: '#131313'
  on-background: '#e5e2e1'
  surface-variant: '#353535'
typography:
  headline-xl:
    fontFamily: Space Mono
    fontSize: 48px
    fontWeight: '700'
    lineHeight: '1.1'
    letterSpacing: -0.05em
  headline-lg:
    fontFamily: Space Mono
    fontSize: 32px
    fontWeight: '700'
    lineHeight: '1.2'
  headline-md:
    fontFamily: Space Mono
    fontSize: 24px
    fontWeight: '700'
    lineHeight: '1.2'
  body-lg:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '400'
    lineHeight: '1.6'
  body-md:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.5'
  label-md:
    fontFamily: Space Mono
    fontSize: 14px
    fontWeight: '700'
    lineHeight: '1.0'
  headline-lg-mobile:
    fontFamily: Space Mono
    fontSize: 28px
    fontWeight: '700'
    lineHeight: '1.2'
spacing:
  pixel-unit: 4px
  block-sm: 8px
  block-md: 16px
  block-lg: 32px
  gutter: 16px
  margin-mobile: 16px
  margin-desktop: 40px
---

## Brand & Style
This design system captures the nostalgic, low-fidelity essence of sandbox voxel gaming, translated into a functional web and mobile interface. The personality is rugged, adventurous, and tactile, aiming to evoke the "clicky" satisfaction of inventory management and crafting.

The style is **Pixel-Brutalism**. It utilizes heavy, multi-layered borders to simulate 3D depth within a 2D plane, high-contrast beveling, and a rigid adherence to a block-based grid. Every element should feel like a physical "item" or "block" that has weight and occupies a specific slot in the world.

## Colors
The palette is rooted in the "Stone and Charcoal" aesthetic of classic game menus. 

- **Primary (Emerald):** Used for "Success" states, experience bars, and active toggles.
- **Secondary (Stone):** The structural foundation. Used for bevels, borders, and button faces.
- **Tertiary (Redstone):** Reserved for "Destructive" actions, errors, and low-health indicators.
- **Neutral (Charcoal):** The background for item slots and deep container wells.

**Surface Treatment:** Use a tiling noise or pixelated texture (simulating dirt or stone) at low opacity over the `#1D1D1D` background to provide grit and character.

## Typography
The system uses a dual-font approach to balance theme and readability. 

**Space Mono** serves as the thematic anchor. It is used for all headings, buttons, and "stat" labels. To mimic a pixel font, it should always be rendered with high weight (`700`) and, where possible, with text-shadows that match the blocky UI borders.

**Inter** provides the necessary legibility for long-form text, descriptions, and settings. It acts as the "Tool-tip" font, staying neutral and out of the way of the more expressive display type.

## Layout & Spacing
The layout follows a **Rigid Grid** philosophy. Everything is built on a 4px "pixel unit." 

- **The Container:** Main UI panels are centered with fixed widths (e.g., 800px for desktop) to mimic a game "window" popping up.
- **Item Slots:** A core layout component is the "Inventory Grid," consisting of 64x64px squares with 4px internal gutters.
- **Responsibility:** On mobile, panels scale to fill the width, but margins remain thick (16px) to maintain the "windowed" feel. Elements do not fluidly stretch; they "snap" to the nearest 4px increment.

## Elevation & Depth
Depth is achieved through **Simulated Extrusion** rather than soft shadows. 

- **Raised Elements (Buttons):** A 4px top/left border of light gray (`#BDBDBD`) and a 4px bottom/right border of dark gray (`#404040`). 
- **Sunken Elements (Input Fields/Slots):** The inverse of buttons. A 4px top/left border of black (`#000000`) and a 4px bottom/right border of light gray (`#8B8B8B`).
- **Overlay Panels:** Use a high-density backdrop blur (20px) with a semi-transparent charcoal tint (`#1D1D1DCC`) to isolate the active "GUI" from the background world.

## Shapes
The design system is strictly **Sharp (0px roundedness)**. 

To maintain the pixelated aesthetic, avoid all CSS border-radius. Any "rounding" should be simulated using stepped pixel patterns in the border art itself if necessary, but for this system, perfectly square corners are the standard. This reinforces the "Block" nature of the brand.

## Components

### Buttons
Buttons are the most tactile element. 
- **Normal:** Stone gray background with beveled edges. Text is white with a dark drop shadow.
- **Hover:** Secondary color (`#8B8B8B`) face with a thick yellow/green border (Experience Green).
- **Pressed:** Face shifts downward 2px; shadow orientation reverses to look "sunken."

### Item Slots (Cards)
Used for displaying content or products.
- Square containers with a `#1D1D1D` background.
- High-contrast inset border to create a "well."
- Hovering over a slot triggers a "Tooltip" – a dark purple box with a light purple border (reminiscent of enchanted item hovers).

### Input Fields
- Deeply recessed boxes. 
- Font: Space Mono.
- Cursor: A solid block `_` that blinks, rather than a thin line.

### Progress Bars (Experience Bar)
- Background: Black.
- Fill: Vibrant Emerald Green (`#55FF55`).
- Segmented every 10% with a 2px black line to mimic discrete "levels."

### Checkboxes & Radios
- Styled as small "Slots."
- Checked state: A pixelated "X" or "Check" mark in Emerald Green.
- Unchecked: Empty recessed well.