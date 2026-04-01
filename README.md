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
