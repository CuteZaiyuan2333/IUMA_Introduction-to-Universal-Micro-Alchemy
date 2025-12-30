# IUMA Architecture & Design Document

## 1. Core Philosophy (Universal Axioms)

### 1.1 The Void & Particles
- The universe consists of a continuous 2D space (The Void) and Fundamental Particles.
- **Immutability:** Particles do not change type or transmute. Complex behaviors emerge from motion and interaction, not state change.
- **Decoupling:** Particles do not directly interact with other particles. They interact exclusively with **Fields**.

### 1.2 The Speed of Light ($C$)
- There is no friction in the void.
- Instead, there is a universal speed limit ($C$).
- As $|v| \to C$, further acceleration diminishes (simulating relativistic mass increase or simple clamping, TBD).

## 2. Field Mechanics (The Alchemy)

### 2.1 Field Types
- A "Field Type" is a distinct layer of reality (e.g., "Heat", "Gravity", "Psionic").
- **Splatting Strategy:** 
    - Each Field Type corresponds to one global floating-point texture (Field Map).
    - Every frame, we clear these maps. 
    - Every particle "draws" (splats) its influence onto the map corresponding to the field it emits.
    - **Linearity:** Field strengths stack additively ($Total = A + B$). 

### 2.2 Particle-Field Interaction
- **Emission:** A particle type defines which field it emits and the shape of that emission (Bezier curve over distance).
- **Reception:** A particle type defines how it reacts to *other* fields.
    - `Weight`: Positive (attract), Negative (repel), Zero (ignore).
    - `Force` = $\nabla Field(pos) \times Weight$.

## 3. Technical Implementation (Bevy + WGPU)

### 3.1 Data Structure (ECS)
- **`Particle` Component:**
    - `pos`: Vec2
    - `vel`: Vec2
    - `mass`: f32
    - `type_id`: usize
- **`FieldDefinition` Resource:**
    - Stores the Bezier curve data and emission parameters.
- **`InteractionMatrix` Resource:**
    - A lookup table defining the `Weight` of `ParticleType` vs `FieldType`.

### 3.2 Render/Compute Graph
1.  **Clear Phase:** Zero out all Field Textures.
2.  **Splat Phase (Compute/Render):** Iterate all particles -> Write to Field Textures (Additive Blending).
3.  **Update Phase (Compute):** Iterate all particles -> Sample Field Textures (Gradient) -> Update Velocity -> Update Position.
