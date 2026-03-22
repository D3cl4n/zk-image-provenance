import csv
import poseidon
from PIL import Image


# write the pixel values to a output csv for verification against prover
def write_pixels_to_csv(pixel_arr):
    output_csv = "output/pixels_python.csv"
    with open(output_csv, "w") as f:
        writer = csv.writer(f)
        writer.writerows(pixel_arr)

    f.close()

    
# read the bytes of the png into an array
def edit_img(image):
    img = Image.open(image).convert("RGB")
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
    write_pixels_to_csv(original_pixel_arr)

    return greyscale_pixel_arr


# implement the padding scheme to match rust code - will shift to Pi
def pad(preimage, rate):
    rem = len(preimage) % rate
    if rem != 0:
        preimage.extend([0] * rate - rem)

    return [preimage[i:i+3] for i in range(0, len(preimage), 3)]


# implement the sponge functionality - will be done on the Pi once prototype works
def absorb():
    pass


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

    # pad input to poseidon
    preimage_padded = pad(preimage, rate)

    # initialize a poseidon instance
    poseidon_instance = poseidon.Poseidon(p, security_level, alpha, rate, t, full_rounds, partial_rounds, mds_matrix, rc_list, p_bits)
    H = poseidon_instance.run_hash(preimage)


# sign - this will all be done on the Pi once prototype works
def sign(image):
    img = Image.open(image).convert("RGB") # open again since this will be moved to script on Pi
    pixels = img.load()
    width, height = img.size
    r_vec = []
    g_vec = []
    b_vec = []
    exif_vec = img.getexif().tobytes()[6:] # TODO: this will need to match rust exifdata, rust has 2 trailing \x00

    print(f"[+] Exifdata: {exif_vec}")
    print(f"[+] Exifdata length: {len(exif_vec)}")

    # iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y]
            r_vec.append(r)
            g_vec.append(g)
            b_vec.append(b)

    # compute Poseidon(r||g||b||exif)
    preimage = r_vec + g_vec + b_vec + list(exif_vec)
    hash_img_details(preimage)


# main function
def main():
    image = "original.jpg" # relative path
    # all camera / signing functionality below
    sign(image)

    # all editor functionality below
    greyscale_values = edit_img(image)

    # write the greyscale values to list as public input for ZKP generation
    greyscale_output_csv = "output/greyscale.txt"
    with open(greyscale_output_csv, "w") as f:
        for y in greyscale_values:
            f.write(str(y) + "\n")


if __name__ == '__main__':
    main()