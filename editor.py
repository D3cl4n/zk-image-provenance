import csv
from PIL import Image


# write the pixel values to a output csv for verification against prover
def write_pixels_to_csv(pixel_arr):
    output_csv = "output/pixels_python.csv"
    with open(output_csv, "w") as f:
        writer = csv.writer(f)
        writer.writerows(pixel_arr)

    
# read the bytes of the png into an array
def edit_img(image):
    img = Image.open(image).convert("RGB")
    width, height = img.size
    pixels = img.load()
    grey_img = Image.new("L", (width, height)) # create a new image to write to - greyscale
    grey_pixels = grey_img.load()

    # lists of pixel values
    pixel_arr = []
    
    # iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y]
            pixel_arr.append([r, g, b])
            
            grey_pixel = (30*r + 58*g + 11*b) // 100
            grey_pixels[x, y] = grey_pixel

    grey_img.save("greyscale.png")
    write_pixels_to_csv(pixel_arr)


# main function
def main():
    image = "image.png" # relative path
    edit_img(image)


if __name__ == '__main__':
    main()