# zk-image-provenance

## Problem Statement
Zero-knowledge proofs enable a prover to convince a verifier that a statement is true, without revealing the underlying witness data. This primitive naturally lends itself to privacy-preserving systems, where hiding the witness prevents the verifier from learning sensitive information. That said, zero-knowledge proofs can also be used in systems where the witness is not necessarily confidential but is not readily available to the verifier. One such use case is image provenance, where signed images are transformed before being distributed. Since the original image is not available to the user, the digital signature cannot be verified without a zero-knowledge proof. In this use case, zero-knowledge proofs enable verification of the authenticity of the image's source, the integrity of the image contents, and that only permitted transformations were applied. In this work we present an end-to-end prototype system that implements this provenance framework and several optimizations. One of our key optimizations is a packing scheme for reducing the number of Poseidon sponge absorb and permutation operations by ~32x. We also show that this packing scheme reduces median prover runtime by ~40x and verifier runtime by ~22x. We also introduce a chain of trust that removes digital signature verification from the circuit. Finally, we introduce custom PNG chunks that embed the required information in the captured images.

## Hardware and Software Specifications
### Signing Camera
The Raspberry Pi 4 used as the signing camera has the following specifications:
- CPU: Broadcom BCM2711, Quad core Cortex-A72 (ARM v8) 64-bit
- RAM: 8 GB LPDDR4-3200 SDRAM
- Disk: 128 GB MicroSD
- Raspberry Pi Camera Module v2

### Prover and Verifier Machine
The proofs are computed by a Lenovo Thinkpad X1 with the following specifications:
- CPU: 12th Gen Intel i9-12900H
- Memory: 16 GB
- OS: Ubuntu 22.04.4 (in WSL2)
- rustc: 1.87.0
- Kernel: x86\_64 Linux 6.6.87.2-microsoft-standard-WSL2

### Software Used
The primary libraries used by the implementation are listed below:
- halo2\_proofs v0.3.2 
- Python v3.14
- halo2curves v0.9.0
- image v0.25.9
- rand v0.8
- ff v0.13.1
- crc32fast v1.5.0
- secp256k1 v0.31.1

