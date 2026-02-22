import csv
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

    grey_img.save("greyscale.png")
    write_pixels_to_csv(original_pixel_arr)

    return greyscale_pixel_arr


# main function
def main():
    image = "image.png" # relative path
    greyscale_values = edit_img(image)

    # write the greyscale values to list as public input for ZKP generation
    greyscale_output_csv = "output/greyscale.txt"
    with open(greyscale_output_csv, "w") as f:
        for y in greyscale_values:
            f.write(str(y) + "\n")


if __name__ == '__main__':
    main()