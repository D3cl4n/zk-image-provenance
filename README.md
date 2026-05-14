# zk-image-provenance

## Problem Statement
Cameras can use a signing key to digitially sign photos as soon as they are captured. The digital signature provides authenticity that the photo came from a real camera (as opposed to AI generation or vice versa) and integrity of the photo contents + metadata. That said, the original image is often not what is distributed to the public. For example, photos in newspapers are often edited before distributed (greyscaling, cropping, etc.). Recepients of the edited image are not able to verify the digital signature since they do not possess the original image. 

## Prototype Summary
### Camera Computations
- $\text{SK}, \text{VK} = \text{ECDSA.Keygen}()$
- $\text{I} = \text{Capture}(\text{width, height, format})$
- $\text{H} = \text{Poseidon}(\text{I}_{R}||\text{I}_{G}||\text{I}_{B}||\text{I}_\text{exif})$
- $\sigma = \text{ECDSA.Sign}(\text{SK}, \text{H})$
- $\text{CustomChunk}(\text{data}, \text{type}) = \text{length}(\text{data}) || \text{type} || \text{data} || \text{CRC32}(\text{type} || \text{data})$
- $\text{ExifChunk} = \text{CustomChunk}(\text{I}_\text{exif}, \text{eXIf})$
- $\text{HashChunk} = \text{CustomChunk}(\text{H}, \text{hASh})$
- $\text{SignatureChunk} = \text{CustomChunk}(\sigma, \text{sIGn})$
- $\text{I} = \text{EmbedChunks}(\text{I}, \text{ExifChunk} || \text{HashChunk} || \text{SignatureChunk})$

### Editor Computations
- $\text{I}'_{\text{IDAT}} = \text{Greyscale}(\text{I}_{\text{IDAT}}) = \left\lfloor \frac{30 \times \text{r} + 58 \times \text{g} + 11 \times \text{b}}{100} \right\rfloor; \; \forall\: \text{r,g,b} \in \text{I}$
- $\text{I}'_{\text{IHDR}} = \text{SetToZero}(\text{I}_{\text{IHDR.ColorType}})$
- $\text{I}' = \text{I}_\text{IHDR}' || \text{I}_\text{IDAT}' || \text{ExifChunk} || \text{HashChunk} || \text{SignatureChunk} || \text{I}_\text{IEND}$

#### ZKP Computed by Editor (Instance-Witness Relationship)
- $\mathcal{R} := \{(\text{H}, \text{I}') \; ; \; (\text{I}_{\text{R}}, \text{I}_{\text{G}}, \text{I}_{\text{B}}) \; :\\ \text{I}' = \text{Greyscale}(\text{I}_{\text{R}}, \text{I}_{\text{G}}, \text{I}_{\text{B}}) \wedge \text{Poseidon}(\text{I}_{R}||\text{I}_{G}||\text{I}_{B}||\text{I}_\text{exif}) = \text{H}\}$
- $\pi = \text{Halo2.Prove}(\text{I}, \text{I}', \text{H}_{0}, \text{H}_{1})$

### Verifier Computations
- $\text{H}_{0} = \text{Poseidon}(\text{I}'_{R}||\text{I}'_{G}||\text{I}'_{B}||\text{I}'_{\text{exif}})$
- $\text{H}_{1} = \text{Extract}(\text{I}', \text{hASh})$
- $\sigma = \text{Extract}(\text{I}', \text{sIGn})$
- $\text{ECDSA.Verify}(\text{PK}, \sigma, \text{H}) = \text{True}$ 
- $\text{Halo2.Verify}(\pi, \text{I}', \text{H}) = \text{True}$


## Threat Model
### Hash Swapping

### Signature Swapping

### Unauthorized Image Transformation After Signing

### Exifdata Tampering

### Alternate Greyscale Linear Combinations

### Original Image Dictionary Attack

