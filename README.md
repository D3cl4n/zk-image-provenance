# zk-image-provenance

## Problem Statement
Cameras can use a signing key to digitially sign photos as soon as they are captured. The digital signature provides authenticity that the photo came from a real camera (as opposed to AI generation or vice versa) and integrity of the photo contents + metadata. That said, the original image is often not what is distributed to the public. For example, photos in newspapers are often edited before distributed (greyscaling, cropping, etc.). Recepients of the edited image are not able to verify the digital signature since they do not possess the original image. 

## Setup on Raspberry Pi
1) $\text{SK}, \text{PK} \leftarrow \text{keygen}()$ 
2) $\text{PK}$ is stored on the laptop for the prover and verifier to use

## Execution Steps
1) Python script on the Raspberry Pi executes `os.system("rpi-jpeg --output photo.jpeg --height 512 --width 512")`. We denote the photo taken as $\text{I}$.
2) The same python script on the Raspberry Pi executes $$\sigma = \text{Sign}_{\text{SK}}(\text{Poseidon}(\text{I}))$$
3) Laptop uses `scp` to retrieve $\text{I}$ off the Raspberry Pi for editing
4) Python script on laptop, acting as the editor, computes $$\text{I}' = \text{Greyscale}(\text{I})$$
5) The same python script acting as the editor runs a Rust binary supplying $\text{I}, \text{I}', \sigma$ (via cli arguments) to a Halo2 circuit. The private witness is $\text{I}$ and the public output is $\text{I}'$ and $\sigma$
6) The circuit arithmetizes the following statement without revealing $\text{I}$: $$\text{Greyscale}(\text{I}) = \text{I}' \wedge \text{Verify}_{\text{PK}}(\sigma, \text{I}) = 1$$

## Threat Analysis
1) Since the photo is signed automatically as soon as it is captured, and only the camera has $\text{SK}$, the editor cannot use $\text{SK}$ to edit the metadata or contents in $\text{I}$ and then re-sign it.
2) If the editor supplies $\text{I}'' \neq \text{I}'$ to the verifier, the proof will fail since the proof only works with $\text{Greyscale}(I) = \text{I}'$. A separate proof would need to be computed for $\text{I}''$
3) If the editor swaps out $\sigma = \text{Sign}_{\text{SK}}(\text{Poseidon}(I))$ for $\sigma' = \text{Sign}_{\text{SK}}(\text{Poseidon}(I'))$ but provides $\text{I}' = \text{Greyscale}(I)$ to the verifier, the proof will fail since there is no collision such that $\text{Poseidon}(I) = \text{Poseidon}(I')$
4) If the editor uses $\text{SK}' \neq \text{SK}$ to compute
 a new signature for $\text{I}$ the proof will fail since the verifier uses the corresponding $\text{PK}$ from the camera only. 

 ## Approach Using Commitments 