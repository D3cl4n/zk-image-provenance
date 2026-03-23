import ecdsa
import poseidon
import piexif
import hashlib
from PIL import Image

    
# read the bytes of the png into an array
def edit_img(image_path):
    img = Image.open(image_path).convert("RGB")
    width, height = img.size
    pixels = img.load()
    grey_img = Image.new("L", (width, height)) # create a new image to write to - greyscale
    grey_pixels = grey_img.load()

    # lists of pixel values
    original_pixel_arr = []
    greyscale_pixel_arr = []
    
    # iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y]
            original_pixel_arr.append([r, g, b])
            
            grey_pixel = (30*r + 58*g + 11*b) // 100
            greyscale_pixel_arr.append(grey_pixel)
            grey_pixels[x, y] = grey_pixel

    grey_img.save("greyscale.jpg")


# implement the padding scheme to match rust code - will shift to Pi
def pad(preimage, rate):
    rem = len(preimage) % rate
    if rem != 0:
        preimage.extend([0] * (rate - rem))

    return [preimage[i:i+3] for i in range(0, len(preimage), 3)]


# compute Poseidon(r||g||b||exif) - this will be done on the Pi once prototype works
def hash_img_details(preimage):
    security_level = 128
    rate = 3
    t = 4
    full_rounds = 8
    partial_rounds = 56
    alpha = 5
    p = poseidon.parameters.prime_255 # BLS12-381 scalar field
    rc_list = poseidon.parameters.round_constants_neptune
    mds_matrix = poseidon.parameters.matrix_neptune
    p_bits = 255

    # initialize a poseidon instance and hash
    poseidon_instance = poseidon.Poseidon(p, security_level, alpha, rate, t, full_rounds, partial_rounds, mds_matrix, rc_list, p_bits)
    # pad input to poseidon and process first block
    blocks = pad(preimage, rate)
    field = poseidon_instance.field_p
    poseidon_instance.state = field([blocks[0][0], blocks[0][1], blocks[0][2], 0]) # absorb first block
    poseidon_instance.rc_counter = 0
    poseidon_instance.full_rounds() # inherently executes RF / 2 rounds
    poseidon_instance.partial_rounds()
    poseidon_instance.full_rounds()

    # permute over the remaining blocks, resetting constant counter and carrying over capacity element
    for block in blocks[1:]:
        poseidon_instance.state[0] += field(block[0])
        poseidon_instance.state[1] += field(block[1])
        poseidon_instance.state[2] += field(block[2])
        poseidon_instance.rc_counter = 0
        poseidon_instance.full_rounds()
        poseidon_instance.partial_rounds()
        poseidon_instance.full_rounds()

    H = poseidon_instance.state[1]

    return int(H)


# embed the signature into the jpg, in the Makernote https://exiv2.org/makernote.html
def embed_signature(image_path, signature_bytes):
    exif_dict = piexif.load(image_path)
    value = b"zkp-sig\x00" + signature_bytes
    exif_dict["Exif"][piexif.ExifIFD.MakerNote] = value

    exif_bytes = piexif.dump(exif_dict)
    piexif.insert(exif_bytes, image_path)


# sign - this will all be done on the Pi once prototype works
def sign(image_path, sk):
    img = Image.open(image_path).convert("RGB") # open again since this will be moved to script on Pi
    pixels = img.load()
    width, height = img.size
    r_vec = []
    g_vec = []
    b_vec = []
    exif_vec = img.getexif().tobytes()[6:] + b"\x00\x00" # add two trailing null bytes to match Rust

    # iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y]
            r_vec.append(r)
            g_vec.append(g)
            b_vec.append(b)

    # compute Poseidon(r||g||b||exif)
    preimage = r_vec + g_vec + b_vec + list(exif_vec)
    H = hash_img_details(preimage)
    H_bytes = H.to_bytes(32, "little")
    signature = sk.sign(H_bytes, hashfunc=hashlib.sha256)
    embed_signature(image_path, signature)

    H_bytes, signature


# main function
def main():
    image_path = "original.jpg" # relative path
    # all camera / signing functionality below
    sk = ecdsa.SigningKey.generate(curve=ecdsa.SECP256k1, hashfunc=hashlib.sha256)
    vk = sk.get_verifying_key()
    hash, signature = sign(image_path, sk)

    # all editor functionality below
    edit_img(image_path)


# TODO: clean up the whole script - pack multiple bytes / pixel values into one field element
# 255 bits in BLS12-381 scalar field Fr lets us hold 31 bytes per field element, pack in both python and rust
if __name__ == '__main__':
    main()