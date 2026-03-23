import ecdsa
import poseidon
import piexif
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
    def __init__(self):
        self.sk = None
        self.vk = None
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
    # TODO: make sure this is done, add in packing functionality
    def pad(self, preimage, rate):
        rem = len(preimage) % rate
        if rem != 0:
            preimage.extend([0] * (rate - rem))

        return [preimage[i:i+3] for i in range(0, len(preimage), 3)]

    # compute the digital signature of the original image
    def sign(self):
        pass

    # capture a photo (do not call this function unless on the pi)
    def capture(self):
        pass # TODO: replace with os.system invocation of rpi-cam


# main function
def main():
    # camera functionality
    camera = Camera()
    camera.keygen()

    # editor functionality
    editor = Editor("original.jpg")
    editor.greyscale()


if __name__ == '__main__':
    main()