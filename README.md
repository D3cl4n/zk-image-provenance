# zk-image-provenance

## Problem Statement
Cameras can use a signing key to digitially sign photos as soon as they are captured. The digital signature provides authenticity that the photo came from a real camera (as opposed to AI generation or vice versa) and integrity of the photo contents + metadata. That said, the original image is often not what is distributed to the public. For example, photos in newspapers are often edited before distributed (greyscaling, cropping, etc.). Recepients of the edited image are not able to verify the digital signature since they do not possess the original image. 

## Threat Analysis
1) Since the photo is signed automatically as soon as it is captured, and only the camera has $\text{SK}$, the editor cannot use $\text{SK}$ to edit the metadata or contents in $\text{I}$ and then re-sign it.
2) If the editor supplies $\text{I}'' \neq \text{I}'$ to the verifier, the proof will fail since the proof only works with $\text{Greyscale}(I) = \text{I}'$. A separate proof would need to be computed for $\text{I}''$
3) If the editor swaps out $\sigma = \text{Sign}_{\text{SK}}(\text{Poseidon}(I))$ for $\sigma' = \text{Sign}_{\text{SK}}(\text{Poseidon}(I'))$ but provides $\text{I}' = \text{Greyscale}(I)$ to the verifier, the proof will fail since there is no collision such that $\text{Poseidon}(I) = \text{Poseidon}(I')$
4) If the editor uses $\text{SK}' \neq \text{SK}$ to compute
 a new signature for $\text{I}$ the proof will fail since the verifier uses the corresponding $\text{PK}$ from the camera only. 

 ## TODO
- edit camera script to embed eXIF chunk in accordance with https://www.w3.org/TR/png-3/#eXIf (use fake data)
- edit camera script to embed ECDSA signature inside the png
- make sure exifdata parser for Rust and Python both get the same chunk and replace dummy exif data in code 
- Use real prover and verifier, calculate minimum number of rows needed. Verifier will need to use camera's public key
- demonstrate failed threats in the threat analysis section and document
- secure storate of the signing key on a chip in the pi somehow (buy a chip)
- write a paper

## idea for removing signature verification from the circuit
- camera computes: 
$$ \text{I}, \text{metadata} = \text{capture}()$$
$$\sigma = \text{sign}_{\text{SK}}(\text{POSEIDON}(\text{I} || \text{metadata}))$$
- editor computes (off-circuit):
$$ \text{I}' = \text{greyscale}(\text{I})$$
- editor computes a proof for the following statement (on-circuit) given $\text{H}, \text{metadata}, \text{I}'$:
$$ \text{I}' = \text{greyscale}(\text{I}) \wedge \text{POSEIDON}(\text{\text{I}}||\text{metadata}) = H $$
- the original hash $\text{H}$ is accessed using $\text{PK}$ off-circuit $\sigma^{\text{PK}} \equiv H \pmod n$ if using RSA signatures as a simple example.
- verifier (recepient) supplied with:
$$ \text{I}', \sigma, \pi $$

## Install Instructions
### On Pi 4
 - `sudo apt install pyenv`
 - `pyenv install 3.10.12` to align with supported versions for poseidon-hash sub-modules
 - `pyenv local 3.10.12` inside the project directory `~/Desktop/zk-image-provenance`
 - `pyenv install --list | grep "3.10.12"` for confirmation of install
 - `nano ~/.bashrc` -> ADD `export PATH="$HOME/.pyenv/bin:$PATH"` and `eval "$(pyenv init -)"` to the bottom
 - `python3.10 -m pip install poseidon-hash` from the project dir


`scp cdeclan@zk-camera-pi:/home/cdeclan/Desktop/zk-image-provenance/photo.jpeg .`

## PNG Specification Relevant Sections
- Structure of a PNG chunk: `https://www.w3.org/TR/png-3/#5Chunk-layout`
- Naming the custom signature chunk: `https://www.w3.org/TR/png-3/#5Chunk-naming-conventions`
- Embedded the exifdata in a chunk: `https://www.w3.org/TR/png-3/#eXIf`
- Color type 0 in IHDR for greyscale: `https://www.w3.org/TR/png-3/#3colourType`


## Prototype Summary
### Camera Computations
- $\text{SK}, \text{VK} = \text{ECDSA.Keygen}()$
- $\text{I} = \text{Capture}(\text{width, height, format})$
- $\text{H} = \text{Poseidon}(\text{I}_{R}||\text{I}_{G}||\text{I}_{B}||\text{I}_\text{exif})$
- $\sigma = \text{ECDSA.Sign}(\text{SK}, \text{H})$
- $\text{CustomChunk}(\text{data}, \text{type}) = \text{length}(\text{data}) || \text{type} || \text{data} || \text{CRC32}(\text{type} || \text{data})$
- $\text{ExifChunk} = \text{CustomChunk}(\text{length}(\text{I}_\text{exif}), \text{eXIf})$
- $\text{HashChunk} = \text{CustomChunk}(\text{length}(\text{H}), \text{hASh})$
- $\text{SignatureChunk} = \text{CustomChunk}(\text{length}(\sigma), \text{sIGn})$
- $\text{I} = \text{EmbedChunks}(\text{I}, \text{ExifChunk} || \text{HashChunk} || \text{SignatureChunk})$

### Editor Computations
- $\text{I}'_{\text{IDAT}} = \text{Greyscale}(\text{I}_{\text{IDAT}}) = \left\lfloor \frac{30 \times \text{r} + 58 \times \text{g} + 11 \times \text{b}}{100} \right\rfloor; \; \forall\: \text{r,g,b} \in \text{I}$
- $\text{I}'_{\text{IHDR}} = \text{SetToZero}(\text{I}_{\text{IHDR}}[9])$
- $\text{I}' = \text{I}_\text{IHDR}' || \text{I}_\text{IDAT}' || \text{ExifChunk} || \text{HashChunk} || \text{SignatureChunk} || \text{I}_\text{IEND}$

### ZKP Computed by Editor (Instance-Witness Relationship)
- $\mathcal{R} := \{(\text{I}', \text{H}) \; ; \; (\text{I}) \; :\\ \text{Greyscale}(\text{I}) = \text{I}' \wedge \text{Poseidon}(\text{I}_{R}||\text{I}_{G}||\text{I}_{B}||\text{I}_\text{exif}) = \text{H}\}$
- $\pi = \text{Halo2.Prove}(\text{I}, \text{I}', \text{H})$

### Verifier Computations
- $\text{H} = \text{Extract}(\text{I}', \text{hASh})$
- $\sigma = \text{Extract}(\text{I}', \text{sIGn})$
- $\text{ECDSA.Verify}(\text{PK}, \sigma, \text{H}) = \text{True}$
- $\text{Halo2.Verify}(\pi, \text{I}', \text{H}) = \text{True}$

## Threat Analysis Summary

### 1) Image Swapping Without Resigning
In this scenario there is a valid image from the camera, but the editor swaps it out for a different image. The alternate image could be generated by AI, or taken by an unauthorized camera. 
- $\text{I}'' = \text{AI.Generate}(\text{width, height, format})$

#### Remediation From Verifier
- $\text{Extract}(\text{I}'', \text{sIGn}) = \text{NULL}$
- Verifier rejects unsigned images

### 2) Image Swapping With Resigning
#### Valid Camera Computations 
- $\text{SK}, \text{VK} = \text{ECDSA.Keygen}()$
- $\text{I} = \text{Capture}(\text{width, height, format})$
- $\text{H} = \text{Poseidon}(\text{I}_{R}||\text{I}_{G}||\text{I}_{B}||\text{I}_\text{exif})$
- $\sigma = \text{ECDSA.Sign}(\text{SK}, \text{H})$

#### Unauthorized Third Party
- $\text{SK}'', \text{VK}'' = \text{ECDSA.Keygen}(); \; \text{SK}'' \neq \text{SK} \; \wedge \; \text{VK}'' \neq \text{VK}$
- $\text{I}'' = \text{Capture}(\text{width, height, format})$
- $\text{H}' = \text{Poseidon}(\text{I}''_{R}||\text{I}''_{G}||\text{I}''_{B}||\text{I}''_\text{exif})$
- $\sigma'' = \text{ECDSA.Sign}(\text{SK}'', \text{H})$

#### Remediation From Verifier
- $\text{ECDSA.Verify}(\text{VK}, \sigma'') = \text{False}$

### 3) Resigning Original Image
#### Valid Camera Computations 
- $\text{SK}, \text{VK} = \text{ECDSA.Keygen}()$
- $\text{I} = \text{Capture}(\text{width, height, format})$
- $\text{H} = \text{Poseidon}(\text{I}_{R}||\text{I}_{G}||\text{I}_{B}||\text{I}_\text{exif})$
- $\sigma = \text{ECDSA.Sign}(\text{SK}, \text{H})$

#### Unauthorized Third Party
- $\text{SK}'', \text{VK}'' = \text{ECDSA.Keygen}(); \; \text{SK}'' \neq \text{SK} \; \wedge \; \text{VK}'' \neq \text{VK}$
- $\sigma

### 4) Signature Swapping
-
- $\sigma = \text{ECDSA.Sign}(\text{SK}, \text{H})$
- $\sigma' = \text{ECDSA.Sign}(\text{SK}, \text{H}'); \; \text{H}' \neq \text{H}$
