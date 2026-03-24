import ecdsa
import poseidon
import piexif
import math
import hashlib
import os
from PIL import Image

'''
- This script aims to demonstrate camera and editor functionality
- This script uses a preset 512x512 JPEG with exifdata, but will use images from the Raspberry Pi 4 Camera Module
- The camera functionality in this script will be moved to a separate python script on the Raspberry Pi once working
- The ECDSA keys should be stored in a hardware module on the Raspberry Pi eventually 
- For computing the Poseidon hash of the pixels, 31 bytes are packed into a single field element, which the circuit will also do
'''

# class for all functionality related to the editor
class Editor:
    def __init__(self, image_path):
        self.image_path = image_path
        self.r_coeff = 30
        self.g_coeff = 58
        self.b_coeff = 11
        self.r = []
        self.g = []
        self.b = []
        self.greyscale_pixels = []
        self.target = "greyscale.jpg"

    # greyscale the image and save as a new jpg
    def greyscale(self):
        print(f"[*] Greyscaling image: {self.image_path}")
        image = Image.open(self.image_path).convert("RGB")
        width, height = image.size
        pixels = image.load()

        # greyscale image
        grey_img = Image.new("L", (width, height)) # create a new image to write to - greyscale
        grey_pixels = grey_img.load()

        # loop over all pixels - going along and then up the rows
        for y in range(height):
            for x in range(width):
                r_val, g_val, b_val = pixels[x, y]
                self.r.append(r_val)
                self.g.append(g_val)
                self.b.append(b_val)

                # greyscale
                grey_val = (self.r_coeff * r_val + self.g_coeff * g_val + self.b_coeff * b_val) // 100
                self.greyscale_pixels.append(grey_val)
                grey_pixels[x, y] = grey_val

        print(f"[*] Saving greyscaled image as: {self.target}")
        grey_img.save(self.target)

        
# class for all functionality related to the camera (move to Raspberry Pi once prototype works)
class Camera:
    def __init__(self, image_path):
        self.sk = None
        self.vk = None
        self.image_path = image_path
        self.poseidon_instance = poseidon.Poseidon(
            poseidon.parameters.prime_255, 
            128, # security level in bits
            5, # alpha for SB
            3, # rate
            4, # t (total state size)
            8, # RF
            56, # RP
            poseidon.parameters.matrix_neptune, # MDS matrix neptune
            poseidon.parameters.round_constants_neptune, # round constants neptune
            255 # modulus bit size
        )


    # generate ECDSA keys - only call if keys are not already generated and securely stored
    def keygen(self):
        self.sk = ecdsa.SigningKey.generate(curve=ecdsa.SECP256k1, hashfunc=hashlib.sha256)
        self.vk = self.sk.get_verifying_key()


    # implement the padding scheme to match rust code - will shift to Pi
    def pad(self, preimage_packed, rate):
        rem = len(preimage_packed) % rate
        if rem != 0:
            preimage_packed.extend([self.poseidon_instance.field_p(0)] * (rate - rem))

        return [preimage_packed[i:i+rate] for i in range(0, len(preimage_packed), rate)]
    
    
    # compute Poseidon(preimage); preimage = pad(pack(r||g||b||exif))
    def hash(self, preimage_blocks):
        field = self.poseidon_instance.field_p
        self.poseidon_instance.state = field([preimage_blocks[0][0], preimage_blocks[0][1], preimage_blocks[0][2], field(0)]) # absorb first block
        self.poseidon_instance.rc_counter = 0
        self.poseidon_instance.full_rounds() # inherently executes RF / 2 rounds
        self.poseidon_instance.partial_rounds()
        self.poseidon_instance.full_rounds()

        # permute over the remaining blocks, resetting constant counter and carrying over capacity element
        for block in preimage_blocks[1:]:
            self.poseidon_instance.state[0] += field(block[0])
            self.poseidon_instance.state[1] += field(block[1])
            self.poseidon_instance.state[2] += field(block[2])
            self.poseidon_instance.rc_counter = 0
            self.poseidon_instance.full_rounds()
            self.poseidon_instance.partial_rounds()
            self.poseidon_instance.full_rounds()

        H = self.poseidon_instance.state[1]

        return int(H)


    # given a vector representing all the bytes to be hashed, pack into field elements (31 bytes per field element)
    def pack(self, raw_preimage):
        preimage_elements = [] # list storing all the field elements, each packed with 31 bytes from preimage
        bytes_per_element = 31
        blocks = [raw_preimage[i:i+bytes_per_element] for i in range(0, len(raw_preimage), bytes_per_element)] # 31 byte blocks
        
        for i in range(len(blocks)):
            element = self.poseidon_instance.field_p(0)
            for j in range(len(blocks[i])): # if last block is less then 31 bytes we will naturally stop
                element += self.poseidon_instance.field_p(blocks[i][j]) * self.poseidon_instance.field_p(256**j)

            preimage_elements.append(element)

        return preimage_elements


    # compute the digital signature of the original image
    def sign(self):
        image = Image.open(self.image_path).convert("RGB") 
        pixels = image.load()
        width, height = image.size
        r_vec = []
        g_vec = []
        b_vec = []

        # extract pixels and exifdata from captured image
        exif_vec = image.getexif().tobytes()[6:] + b"\x00\x00" # add two trailing null bytes to match Rust
        for y in range(height):
            for x in range(width):
                r, g, b = pixels[x, y]
                r_vec.append(r)
                g_vec.append(g)
                b_vec.append(b)

        raw_preimage = r_vec + g_vec + b_vec + list(exif_vec)
        packed_preimage = self.pack(raw_preimage)
        padded_preimage = self.pad(packed_preimage, 3)
        H = self.hash(padded_preimage)
        print(f"[*] Hash of original image: {hex(H)}")


    # capture a photo (do not call this function unless on the pi)
    def capture(self):
        pass # TODO: replace with os.system invocation of rpi-cam


# main function
def main():
    # camera functionality
    camera = Camera("original.jpg")
    camera.keygen()
    camera.sign()

    # editor functionality
    editor = Editor("original.jpg")
    editor.greyscale()


if __name__ == '__main__':
    main()