# cfi_asm_mod
Assembly-based modification tool for my CFI implementation

## What exactly for?

In my bachelor thesis I developed and implemented a coarse-grained CFI check for ARMv7-M based embedded systems. The implementation provided two ways for integrating the changes into an application by instrumentalizing the application code. This is the actual tool implementing one of these ways.

## How it works

The tool expects that the build system / toolchain does not compile + assemble + link in one step. Instead, the build script should be adjusted so that the compiler compiles the source code and just emits the generated assembly code. Then, cfi_asm_mod is called to perform the required changes. Afterwards, the assembler can be used to assemble the modified code. When all desired source units were modified, the linker can finally link the application.
